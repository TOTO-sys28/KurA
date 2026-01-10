Name:           kura-voice
Version:        0.1.0
Release:        1%{?dist}
Summary:        KurA low-CPU Discord voice music bot
License:        MIT
URL:            https://github.com/TOTO-sys28/KurA

BuildRequires:  cargo

%description
KurA streams cached .opus files for low CPU usage.

%prep

%build
cargo build --release --manifest-path %{_sourcedir}/Cargo.toml

%install
mkdir -p %{buildroot}/usr/bin
install -m 0755 %{_sourcedir}/target/release/kura_voice %{buildroot}/usr/bin/kura_voice

mkdir -p %{buildroot}/usr/lib/systemd/system
install -m 0644 %{_sourcedir}/packaging/systemd/kura_voice.service %{buildroot}/usr/lib/systemd/system/kura_voice.service

mkdir -p %{buildroot}/etc
if [ ! -f %{buildroot}/etc/kura_voice.env ]; then
  cat > %{buildroot}/etc/kura_voice.env <<'EOF'
# KurA environment
# DISCORD_TOKEN=YOUR_TOKEN_HERE
OPUS_CACHE=/var/lib/kura_voice/music_opus
RUST_LOG=warn
EOF
  chmod 600 %{buildroot}/etc/kura_voice.env
fi

%files
/usr/bin/kura_voice
/usr/lib/systemd/system/kura_voice.service
%config(noreplace) %attr(600,root,root) /etc/kura_voice.env

%post
%systemd_post kura_voice.service

echo "KurA installed. Edit /etc/kura_voice.env then run: sudo systemctl enable --now kura_voice"

%preun
%systemd_preun kura_voice.service

%postun
%systemd_postun_with_restart kura_voice.service

%changelog
* Sat Jan 10 2026 TOTO-sys28 - 0.1.0-1
- Initial package
