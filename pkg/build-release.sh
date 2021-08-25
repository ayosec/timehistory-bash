#!/bin/bash
#
# This script build a release asset. It is expected to be invoked in a GitHub
# Actions runner.

set -xeuo pipefail

mkdir -p ASSETS

# Install the Rust compiler.

if command -v apt-get
then
  apt-get update
  apt-get install -y \
    build-essential  \
    curl             \
    debhelper        \
    git

  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal -y
  source ~/.cargo/env

elif command -v yum
then
  yum install -y \
    cargo        \
    gettext      \
    git          \
    rpm-build

elif command -v zypper
then
  zypper install -y \
    cargo           \
    gettext-runtime \
    git             \
    rpm-build

else
  echo "Unsupported system"
  exit 1
fi

# Test the builtin in this system.
cargo test

# If MAKE_TARBALL is 1, create two packages (with and without debug info).
#
# Otherwise, build and test a package for the current OS.
if [ "${MAKE_TARBALL:-0}" -eq 1 ]
then
  source <(pkg/common/metadata)

  # With debug.
  RUSTFLAGS="-g" cargo build --release

  tar \
    -czf "ASSETS/$PACKAGE_NAME-$PACKAGE_VERSION-debug.tar.gz" \
    -C target/release \
    libtimehistory_bash.so

  # Without debug.
  cargo clean
  cargo build --release
  strip target/release/libtimehistory_bash.so

  tar \
    -czf "ASSETS/$PACKAGE_NAME-$PACKAGE_VERSION.tar.gz" \
    -C target/release \
    libtimehistory_bash.so

  exit 0

elif command -v apt-get
then
  ./pkg/debian/build.sh

  rm -f target/packages/*-dbgsym*deb
  dpkg -i target/packages/*.deb

else
  ./pkg/rpm/build.sh

  rpm -i target/packages/*.rpm

fi

enable -f /usr/lib/bash/libtimehistory_bash.so timehistory
trap timehistory EXIT

source /etc/os-release
for PKG in target/packages/*
do
  mv -vn \
    "$PKG" \
    "ASSETS/$ID-$VERSION_ID-$(basename "$PKG")"
done
