use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

fn read_input(prompt: &str, default: Option<&str>) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let val = buf.trim();
    if val.is_empty() {
        Ok(default.unwrap_or_default().to_string())
    } else {
        Ok(val.to_string())
    }
}

fn is_audio_file(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(OsStr::to_str) else {
        return false;
    };
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "mp3" | "wav" | "flac" | "m4a" | "ogg" | "aac" | "opus" | "wma" | "aiff" | "alac"
    )
}

fn collect_audio_files(root: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_audio_files(&path, out)?;
        } else if is_audio_file(&path) {
            out.push(path);
        }
    }
    Ok(())
}

fn convert_file(input: &Path, input_root: &Path, output_root: &Path) -> io::Result<()> {
    let relative = input
        .strip_prefix(input_root)
        .unwrap_or(input)
        .with_extension("opus");
    let output = output_root.join(relative);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    let status = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-y")
        .arg("-i")
        .arg(input)
        .arg("-vn")
        .arg("-c:a")
        .arg("libopus")
        .arg("-b:a")
        .arg("128k")
        .arg(output.as_os_str())
        .status()?;

    if !status.success() {
        return Err(io::Error::other(format!(
            "ffmpeg failed for {}",
            input.display()
        )));
    }
    Ok(())
}

fn main() -> io::Result<()> {
    println!("KurA Converter (kurac)");
    println!("Converts audio files to .opus for low-CPU playback.");
    println!();

    let input_dir = read_input("Input music directory [. = here] [.] ", Some("."))?;
    let output_dir = read_input("Output OPUS directory [./music_opus]: ", Some("./music_opus"))?;

    let input_root = PathBuf::from(input_dir);
    let output_root = PathBuf::from(output_dir);

    if !input_root.exists() || !input_root.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Input directory not found: {} (create it or choose another path)",
                input_root.display()
            ),
        ));
    }

    fs::create_dir_all(&output_root)?;

    let mut files = Vec::new();
    collect_audio_files(&input_root, &mut files)?;
    if files.is_empty() {
        println!("No audio files found in {}", input_root.display());
        return Ok(());
    }

    println!("Found {} audio files. Starting conversion...", files.len());
    let mut failed = 0usize;
    for (idx, file) in files.iter().enumerate() {
        print!("[{}/{}] {}\r", idx + 1, files.len(), file.display());
        io::stdout().flush()?;
        if let Err(err) = convert_file(file, &input_root, &output_root) {
            failed += 1;
            eprintln!("\nFailed: {} ({err})", file.display());
        }
    }
    println!();

    if failed == 0 {
        println!("Done. Converted files written to {}", output_root.display());
    } else {
        println!(
            "Done with warnings. {} files failed. Output dir: {}",
            failed,
            output_root.display()
        );
    }
    Ok(())
}
