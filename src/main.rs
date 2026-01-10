use anyhow::{anyhow, Context, Result};
use serenity::async_trait;
use serenity::all::{Client, EventHandler, GatewayIntents};
use serenity::client::Context as SerenityContext;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::TypeMapKey;
use songbird::input::File as SbFile;
use songbird::SerenityInit;
use rand::seq::IteratorRandom;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};
use walkdir::WalkDir;

struct Handler;

#[derive(Clone, Debug)]
struct TrackInfo {
    stem: String,
    norm: String,
    path: PathBuf,
}

struct OpusIndex;

impl TypeMapKey for OpusIndex {
    type Value = Arc<Vec<TrackInfo>>;
}

#[derive(Default)]
struct GuildPlaybackState {
    loop_enabled: bool,
    current: Option<songbird::tracks::TrackHandle>,
}

struct PlaybackState;

impl TypeMapKey for PlaybackState {
    type Value = Arc<Mutex<HashMap<GuildId, GuildPlaybackState>>>;
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: SerenityContext, ready: Ready) {
        info!("Logged in as {}", ready.user.name);

        let root = opus_cache_root();
        match index_opus_files(&root) {
            Ok(index) => {
                let mut data = ctx.data.write().await;
                data.insert::<OpusIndex>(Arc::new(index));
                data.insert::<PlaybackState>(Arc::new(Mutex::new(HashMap::new())));
                info!("Indexed OPUS_CACHE at {:?}", root);
            }
            Err(e) => {
                error!("Failed to index OPUS_CACHE at {:?}: {:#}", root, e);
            }
        }
    }

    async fn message(&self, ctx: SerenityContext, msg: Message) {
        if msg.author.bot {
            return;
        }

        let content = msg.content.trim();
        if !content.starts_with('!') {
            return;
        }

        let mut parts = content.split_whitespace();
        let cmd = parts.next().unwrap_or("");

        let result = match cmd {
            "!join" => cmd_join(&ctx, &msg).await,
            "!leave" => cmd_leave(&ctx, &msg).await,
            "!play" => {
                let q = parts.collect::<Vec<_>>().join(" ");
                cmd_play(&ctx, &msg, q).await
            }
            "!random" => cmd_random(&ctx, &msg).await,
            "!skip" => cmd_skip(&ctx, &msg).await,
            "!loop" => cmd_loop(&ctx, &msg).await,
            "!list" => {
                let q = parts.collect::<Vec<_>>().join(" ");
                cmd_list(&ctx, &msg, q).await
            }
            "!reindex" => cmd_reindex(&ctx, &msg).await,
            "!stop" => cmd_stop(&ctx, &msg).await,
            "!ping" => {
                let _ = msg.channel_id.say(&ctx.http, "pong").await;
                Ok(())
            }
            "!help" => cmd_help(&ctx, &msg).await,
            _ => Ok(()),
        };

        if let Err(e) = result {
            error!("command error: {:#}", e);
            let _ = msg
                .channel_id
                .say(&ctx.http, format!("❌ {e:#}"))
                .await;
        }
    }
}

fn opus_cache_root() -> PathBuf {
    env::var("OPUS_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./music_opus"))
}

fn normalize_name(s: &str) -> String {
    s.trim().to_lowercase()
}

fn index_opus_files(root: &Path) -> Result<Vec<TrackInfo>> {
    let mut out = Vec::new();
    if !root.exists() {
        return Ok(out);
    }

    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("opus")) != Some(true)
        {
            continue;
        }

        let stem = p
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let norm = normalize_name(&stem);
        out.push(TrackInfo {
            stem,
            norm,
            path: p.to_path_buf(),
        });
    }

    Ok(out)
}

async fn guild_and_author_channel_with_ctx(
    ctx: &SerenityContext,
    msg: &Message,
) -> Result<(GuildId, serenity::model::id::ChannelId)> {
    let guild_id = msg.guild_id.ok_or_else(|| anyhow!("This command only works in a guild."))?;
    let guild = ctx
        .cache
        .guild(guild_id)
        .ok_or_else(|| anyhow!("Guild not in cache yet; try again in a moment."))?;
    let vs = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|vs| vs.channel_id)
        .ok_or_else(|| anyhow!("Join a voice channel first."))?;
    Ok((guild_id, vs))
}

async fn cmd_join(ctx: &SerenityContext, msg: &Message) -> Result<()> {
    let (guild_id, channel_id) = guild_and_author_channel_with_ctx(ctx, msg).await?;

    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| anyhow!("Songbird voice manager not initialized"))
        .map(|m| m.clone())?;

    let _handler = manager.join(guild_id, channel_id).await;
    msg.channel_id.say(&ctx.http, "✅ Joined.").await?;
    Ok(())
}

async fn cmd_leave(ctx: &SerenityContext, msg: &Message) -> Result<()> {
    let guild_id = msg.guild_id.ok_or_else(|| anyhow!("This command only works in a guild."))?;

    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| anyhow!("Songbird voice manager not initialized"))
        .map(|m| m.clone())?;

    manager.remove(guild_id).await?;
    msg.channel_id.say(&ctx.http, "👋 Left.").await?;
    Ok(())
}

async fn cmd_stop(ctx: &SerenityContext, msg: &Message) -> Result<()> {
    let guild_id = msg.guild_id.ok_or_else(|| anyhow!("This command only works in a guild."))?;

    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| anyhow!("Songbird voice manager not initialized"))
        .map(|m| m.clone())?;

    let call = manager
        .get(guild_id)
        .ok_or_else(|| anyhow!("Not in a voice channel."))?;

    stop_current_track(ctx, guild_id).await;

    let handler = call.lock().await;
    handler.queue().stop();

    msg.channel_id.say(&ctx.http, "⏹️ Stopped.").await?;
    Ok(())
}

async fn cmd_reindex(ctx: &SerenityContext, msg: &Message) -> Result<()> {
    let root = opus_cache_root();
    let index = index_opus_files(&root).context("Failed to index OPUS_CACHE")?;
    let mut data = ctx.data.write().await;
    data.insert::<OpusIndex>(Arc::new(index));
    msg.channel_id
        .say(&ctx.http, format!("✅ Re-indexed OPUS_CACHE at `{}`", root.display()))
        .await?;
    Ok(())
}

async fn cmd_help(ctx: &SerenityContext, msg: &Message) -> Result<()> {
    msg.channel_id
        .say(
            &ctx.http,
            "Commands: !join, !leave, !play [prefix], !random, !skip, !loop, !stop, !list [prefix], !reindex, !help",
        )
        .await?;
    Ok(())
}

async fn cmd_list(ctx: &SerenityContext, msg: &Message, query: String) -> Result<()> {
    let index_arc = {
        let data = ctx.data.read().await;
        data.get::<OpusIndex>().cloned()
    };
    let root = opus_cache_root();
    let index = match index_arc {
        Some(idx) => idx,
        None => {
            let idx = Arc::new(index_opus_files(&root).context("Failed to index OPUS_CACHE")?);
            let mut data = ctx.data.write().await;
            data.insert::<OpusIndex>(idx.clone());
            idx
        }
    };

    let q = normalize_name(query.trim());
    let mut items: Vec<String> = if q.is_empty() {
        index.iter().map(|t| t.stem.clone()).collect()
    } else {
        let mut prefix: Vec<String> = index
            .iter()
            .filter(|t| t.norm.starts_with(&q))
            .map(|t| t.stem.clone())
            .collect();
        if prefix.is_empty() {
            prefix = index
                .iter()
                .filter(|t| t.norm.contains(&q))
                .map(|t| t.stem.clone())
                .collect();
        }
        prefix
    };

    items.sort();
    if items.is_empty() {
        msg.channel_id
            .say(&ctx.http, format!("No matches in `{}`", root.display()))
            .await?;
        return Ok(());
    }

    send_lines(ctx, msg.channel_id, items).await?;
    Ok(())
}

async fn get_index(ctx: &SerenityContext) -> Result<(PathBuf, Arc<Vec<TrackInfo>>)> {
    let index_arc = {
        let data = ctx.data.read().await;
        data.get::<OpusIndex>().cloned()
    };
    let root = opus_cache_root();
    let index = match index_arc {
        Some(idx) => idx,
        None => {
            let idx = Arc::new(index_opus_files(&root).context("Failed to index OPUS_CACHE")?);
            let mut data = ctx.data.write().await;
            data.insert::<OpusIndex>(idx.clone());
            idx
        }
    };
    Ok((root, index))
}

async fn send_lines(ctx: &SerenityContext, channel_id: serenity::model::id::ChannelId, mut lines: Vec<String>) -> Result<()> {
    const MAX: usize = 1900;
    if lines.is_empty() {
        return Ok(());
    }

    let mut buf = String::new();
    let mut first = true;
    while let Some(line) = lines.first().cloned() {
        lines.remove(0);
        let piece = if first { line } else { format!("\n{}", line) };
        if buf.len() + piece.len() > MAX {
            channel_id.say(&ctx.http, buf.clone()).await?;
            buf.clear();
            first = true;
            continue;
        }
        buf.push_str(&piece);
        first = false;
    }
    if !buf.is_empty() {
        channel_id.say(&ctx.http, buf).await?;
    }
    Ok(())
}

async fn get_playback_state(ctx: &SerenityContext) -> Arc<Mutex<HashMap<GuildId, GuildPlaybackState>>> {
    let existing = {
        let data = ctx.data.read().await;
        data.get::<PlaybackState>().cloned()
    };
    if let Some(v) = existing {
        return v;
    }

    let mut data = ctx.data.write().await;
    let v = Arc::new(Mutex::new(HashMap::new()));
    data.insert::<PlaybackState>(v.clone());
    v
}

async fn apply_loop_setting(
    ctx: &SerenityContext,
    guild_id: GuildId,
    track: &songbird::tracks::TrackHandle,
) -> Result<()> {
    let state = get_playback_state(ctx).await;
    let map = state.lock().await;
    let loop_enabled = map.get(&guild_id).map(|s| s.loop_enabled).unwrap_or(false);
    drop(map);

    if loop_enabled {
        track.enable_loop()?;
    } else {
        track.disable_loop()?;
    }
    Ok(())
}

async fn set_current_track(ctx: &SerenityContext, guild_id: GuildId, track: songbird::tracks::TrackHandle) {
    let state = get_playback_state(ctx).await;
    let mut map = state.lock().await;
    let entry = map.entry(guild_id).or_default();
    entry.current = Some(track);
}

async fn take_current_track(ctx: &SerenityContext, guild_id: GuildId) -> Option<songbird::tracks::TrackHandle> {
    let state = get_playback_state(ctx).await;
    let mut map = state.lock().await;
    map.get_mut(&guild_id).and_then(|e| e.current.take())
}

async fn stop_current_track(ctx: &SerenityContext, guild_id: GuildId) {
    if let Some(track) = take_current_track(ctx, guild_id).await {
        let _ = track.stop();
    }
}

async fn cmd_loop(ctx: &SerenityContext, msg: &Message) -> Result<()> {
    let guild_id = msg.guild_id.ok_or_else(|| anyhow!("This command only works in a guild."))?;
    let state = get_playback_state(ctx).await;

    let (enabled, current) = {
        let mut map = state.lock().await;
        let entry = map.entry(guild_id).or_default();
        entry.loop_enabled = !entry.loop_enabled;
        (entry.loop_enabled, entry.current.clone())
    };

    if let Some(track) = current {
        if enabled {
            track.enable_loop()?;
        } else {
            track.disable_loop()?;
        }
    }

    msg.channel_id
        .say(&ctx.http, if enabled { "🔁 Loop: ON" } else { "Loop: OFF" })
        .await?;
    Ok(())
}

async fn cmd_skip(ctx: &SerenityContext, msg: &Message) -> Result<()> {
    let guild_id = msg.guild_id.ok_or_else(|| anyhow!("This command only works in a guild."))?;

    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| anyhow!("Songbird voice manager not initialized"))
        .map(|m| m.clone())?;
    let call = manager
        .get(guild_id)
        .ok_or_else(|| anyhow!("Not in a voice channel. Use !join first."))?;

    let (root, index) = get_index(ctx).await?;
    if index.is_empty() {
        msg.channel_id
            .say(&ctx.http, format!("No cached .opus files found in `{}`", root.display()))
            .await?;
        return Ok(());
    }

    let selected = {
        let mut rng = rand::thread_rng();
        index
            .iter()
            .choose(&mut rng)
            .map(|t| t.path.clone())
            .ok_or_else(|| anyhow!("No cached .opus files found in `{}`", root.display()))?
    };

    stop_current_track(ctx, guild_id).await;

    let mut handler = call.lock().await;
    handler.queue().stop();
    let input = SbFile::new(selected.clone());
    let track = handler.play_input(input.into());
    track.set_volume(1.0)?;
    apply_loop_setting(ctx, guild_id, &track).await?;
    set_current_track(ctx, guild_id, track.clone()).await;

    msg.channel_id
        .say(&ctx.http, format!("⏭️ Skipped to `{}`", selected.display()))
        .await?;
    Ok(())
}

async fn cmd_random(ctx: &SerenityContext, msg: &Message) -> Result<()> {
    let guild_id = msg.guild_id.ok_or_else(|| anyhow!("This command only works in a guild."))?;
    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| anyhow!("Songbird voice manager not initialized"))
        .map(|m| m.clone())?;
    let call = manager
        .get(guild_id)
        .ok_or_else(|| anyhow!("Not in a voice channel. Use !join first."))?;

    let (root, index) = get_index(ctx).await?;
    if index.is_empty() {
        msg.channel_id
            .say(&ctx.http, format!("No cached .opus files found in `{}`", root.display()))
            .await?;
        return Ok(());
    }

    let selected = {
        let mut rng = rand::thread_rng();
        index
            .iter()
            .choose(&mut rng)
            .map(|t| t.path.clone())
            .ok_or_else(|| anyhow!("No cached .opus files found in `{}`", root.display()))?
    };

    stop_current_track(ctx, guild_id).await;

    let mut handler = call.lock().await;
    handler.queue().stop();
    let input = SbFile::new(selected.clone());
    let track = handler.play_input(input.into());
    track.set_volume(1.0)?;
    apply_loop_setting(ctx, guild_id, &track).await?;
    set_current_track(ctx, guild_id, track.clone()).await;

    msg.channel_id
        .say(&ctx.http, format!("🎲 Random: `{}`", selected.display()))
        .await?;
    Ok(())
}

async fn cmd_play(ctx: &SerenityContext, msg: &Message, query: String) -> Result<()> {
    if query.trim().is_empty() {
        return cmd_random(ctx, msg).await;
    }

    let guild_id = msg.guild_id.ok_or_else(|| anyhow!("This command only works in a guild."))?;

    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| anyhow!("Songbird voice manager not initialized"))
        .map(|m| m.clone())?;

    let call = manager
        .get(guild_id)
        .ok_or_else(|| anyhow!("Not in a voice channel. Use !join first."))?;

    let (root, index) = get_index(ctx).await?;

    let q = normalize_name(query.trim());
    let mut candidates: Vec<PathBuf> = index
        .iter()
        .filter(|t| t.norm.starts_with(&q))
        .map(|t| t.path.clone())
        .collect();
    if candidates.is_empty() {
        candidates = index
            .iter()
            .filter(|t| t.norm.contains(&q))
            .map(|t| t.path.clone())
            .collect();
    }
    if candidates.is_empty() {
        return Err(anyhow!("No cached .opus match in {:?} for '{}'.", root, query));
    }
    let selected = {
        let mut rng = rand::thread_rng();
        candidates
            .iter()
            .choose(&mut rng)
            .cloned()
            .ok_or_else(|| anyhow!("No cached .opus match in {:?} for '{}'.", root, query))?
    };

    stop_current_track(ctx, guild_id).await;

    let mut handler = call.lock().await;

    handler.queue().stop();

    let input = SbFile::new(selected.clone());
    let track = handler.play_input(input.into());
    track.set_volume(1.0)?;
    apply_loop_setting(ctx, guild_id, &track).await?;
    set_current_track(ctx, guild_id, track.clone()).await;

    msg.channel_id
        .say(&ctx.http, format!("▶️ Playing `{}`", selected.display()))
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let token = match env::var("DISCORD_TOKEN") {
        Ok(v) if !v.trim().is_empty() => v,
        _ => {
            if cfg!(windows) {
                eprintln!("DISCORD_TOKEN is required. Run setup.bat (creates kura.env) then: call kura.env && kura_voice.exe");
            } else {
                eprintln!("DISCORD_TOKEN is required. Run: bash scripts/setup.sh (writes /etc/kura_voice.env or ./.env)");
            }
            return Err(anyhow!("DISCORD_TOKEN is required"));
        }
    };

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .register_songbird()
        .await
        .context("Error creating client")?;

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        shard_manager.shutdown_all().await;
    });

    client.start().await.context("Client error")?;
    Ok(())
}
