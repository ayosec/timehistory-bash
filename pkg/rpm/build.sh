#!/bin/bash
#
# This script generates a RPM package with the .so file built from
# cargo build --release.

set -euo pipefail

cd "$(dirname "$0")"

# shellcheck source=/dev/null
source <(../common/metadata)

DEST=$(mktemp -d)

envsubst < timehistory.spec > "$DEST/timehistory.spec"

cd "$(git rev-parse --show-toplevel)"
HOME="$DEST" rpmbuild --build-in-place -bb "$DEST/timehistory.spec"

mkdir -p target/packages
find "$DEST/rpmbuild/RPMS" -type f -name '*.rpm' -exec cp -t target/packages {} +
