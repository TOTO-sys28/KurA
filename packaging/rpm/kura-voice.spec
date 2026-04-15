Name:           kura
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
cargo build --release -j 2 --manifest-path %{_sourcedir}/Cargo.toml

%install
mkdir -p %{buildroot}/usr/bin
install -m 0755 %{_sourcedir}/target/release/kura %{buildroot}/usr/bin/kura
install -m 0755 %{_sourcedir}/target/release/kurac %{buildroot}/usr/bin/kurac

mkdir -p %{buildroot}/usr/lib/systemd/system
install -m 0644 %{_sourcedir}/packaging/systemd/kura.service %{buildroot}/usr/lib/systemd/system/kura.service

mkdir -p %{buildroot}/etc
if [ ! -f %{buildroot}/etc/kura.env ]; then
  cat > %{buildroot}/etc/kura.env <<'EOF'
# KurA environment
# DISCORD_TOKEN=YOUR_TOKEN_HERE
OPUS_CACHE=/var/lib/kura/music_opus
RUST_LOG=warn
EOF
  chmod 600 %{buildroot}/etc/kura.env
fi

%files
/usr/bin/kura
/usr/bin/kurac
/usr/lib/systemd/system/kura.service
%config(noreplace) %attr(600,root,root) /etc/kura.env

%post
%systemd_post kura.service

echo "KurA installed. Edit /etc/kura.env then run: sudo systemctl enable --now kura"

%preun
%systemd_preun kura.service

%postun
%systemd_postun_with_restart kura.service

%changelog
* Thu Apr 16 2026 TOTO-sys28 - 0.1.0-1
- Initial package with kura/kurac binaries
