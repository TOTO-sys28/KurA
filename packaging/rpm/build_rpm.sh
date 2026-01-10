#!/usr/bin/env bash
set -euo pipefail

# Quick RPM build helper (requires rpm-build tooling).
# This script builds an RPM from the current source tree.

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

TOPDIR="$ROOT_DIR/packaging/rpm/.rpmbuild"
RPMDB="$ROOT_DIR/packaging/rpm/.rpmdb"
OUTDIR="$ROOT_DIR/packaging/rpm/dist"

mkdir -p "$TOPDIR"/BUILD "$TOPDIR"/RPMS "$TOPDIR"/SOURCES "$TOPDIR"/SPECS "$TOPDIR"/SRPMS
mkdir -p "$RPMDB" "$OUTDIR"

rpmbuild -bb "$ROOT_DIR/packaging/rpm/kura-voice.spec" \
  --nodeps \
  --define "_topdir $TOPDIR" \
  --define "_sourcedir $ROOT_DIR" \
  --define "_rpmdir $OUTDIR" \
  --define "_srcrpmdir $OUTDIR" \
  --define "_dbpath $RPMDB"
