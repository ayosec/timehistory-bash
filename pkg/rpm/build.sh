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
rpmbuild -bb --build-in-place             \
  --define "_rpmdir $PWD/target/packages" \
  "$DEST/timehistory.spec"

cd target/packages
mv */*.rpm .
