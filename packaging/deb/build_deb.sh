#!/usr/bin/env bash
set -euo pipefail

# Build a minimal .deb from release binaries.
# Requires: dpkg-deb

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
PKGDIR="$ROOT_DIR/packaging/deb_pkg"

rm -rf "$PKGDIR"
mkdir -p "$PKGDIR/DEBIAN" "$PKGDIR/usr/bin" "$PKGDIR/usr/lib/systemd/system" "$PKGDIR/etc"

cp "$ROOT_DIR/packaging/deb/DEBIAN/control" "$PKGDIR/DEBIAN/control"
cp "$ROOT_DIR/packaging/deb/DEBIAN/postinst" "$PKGDIR/DEBIAN/postinst"
cp "$ROOT_DIR/packaging/deb/DEBIAN/prerm" "$PKGDIR/DEBIAN/prerm"
chmod 755 "$PKGDIR/DEBIAN/postinst" "$PKGDIR/DEBIAN/prerm"

cp "$ROOT_DIR/packaging/systemd/kura.service" "$PKGDIR/usr/lib/systemd/system/kura.service"

# env file template
umask 077
cat > "$PKGDIR/etc/kura.env" <<'EOF'
# KurA environment
# DISCORD_TOKEN=YOUR_TOKEN_HERE
OPUS_CACHE=/var/lib/kura/music_opus
RUST_LOG=warn
EOF
chmod 600 "$PKGDIR/etc/kura.env"

# binaries
if [ ! -f "$ROOT_DIR/target/release/kura" ]; then
  echo "Build the binary first: cargo build --release" >&2
  exit 1
fi
cp "$ROOT_DIR/target/release/kura" "$PKGDIR/usr/bin/kura"
cp "$ROOT_DIR/target/release/kurac" "$PKGDIR/usr/bin/kurac"
chmod 755 "$PKGDIR/usr/bin/kura" "$PKGDIR/usr/bin/kurac"

dpkg-deb --root-owner-group --build "$PKGDIR" "$ROOT_DIR/kura-voice_0.1.5_amd64.deb"
