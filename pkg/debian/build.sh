#!/bin/bash
#
# This script generates a Debian package with the .so file built from
# cargo build --release.
#
# The .deb files are copied to '$ROOT/target/packages'.

set -euo pipefail

PACKAGE_NAME=''
SOURCE=$(git rev-parse --show-toplevel)

cd "$(dirname "$0")"

# shellcheck source=/dev/null
source <(../common/metadata)


# Build the shared library.

cd "$SOURCE"
cargo build --release


# Copy the generated files to the destination.
#
# .md files in the root are copied as .txt files to the
# /usr/share/doc/ directory of the package.

DEST=$(mktemp -d)

DEST_BIN="$DEST/debian/$PACKAGE_NAME/usr/lib/bash"
DEST_DOC="$DEST/debian/$PACKAGE_NAME/usr/share/doc/$PACKAGE_NAME"

DEST_DEB="$SOURCE/target/packages"

mkdir -p "$DEST_BIN"
find target/release -maxdepth 1 -name '*.so' -exec cp -a {} "$DEST_BIN" \;

mkdir -p "$DEST_DOC"
cp -a ./*.md "$DEST_DOC"

mkdir -p "$DEST_DEB"


# Generate files for the Debian scripts.

PACKAGE_AUTHOR=$(git log --pretty='%an <%ae>' -1 pkg/debian)
PACKAGE_DATE=$(date --rfc-email)
export PACKAGE_AUTHOR PACKAGE_DATE

echo 11 > "$DEST/debian/compat"
envsubst < pkg/debian/control > "$DEST/debian/control"
envsubst < pkg/debian/changelog > "$DEST/debian/changelog"


# Build the package.

set -x

cd "$DEST"

dh_fixperms
dh_strip
dh_shlibdeps
dh_gencontrol
dh_md5sums
dh_builddeb --destdir="$DEST_DEB"
