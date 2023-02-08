#!/usr/bin/env bash

set -euo pipefail

# Check if an architectureargument is provided.
if [ "$#" -ne 1 ]; then
  echo "Error: Architecture argument missing. Usage: $0 <architecture>" >&2
  exit 1
fi

# Determine the Rust target triple based on the Debian package architecture.
architecture=$1
rust_target=""
if [ "$architecture" == "amd64" ]; then
  rust_target="x86_64-unknown-linux-musl"
elif [ "$architecture" == "arm64" ]; then
  rust_target="aarch64-unknown-linux-musl"
elif [ "$architecture" == "armhf" ]; then
  rust_target="armv7-unknown-linux-musleabihf"
else
  echo "Error: Unsupported architecture: $architecture" >&2
  exit 1
fi

binary_archive="assets/rpi-derive-key_${rust_target}.tar.gz"

cargo build --release --target=$rust_target

# prepare the Debian package
mkdir -p debian/usr/bin
cp target/$rust_target/release/your-rust-cli debian/usr/bin/
version=$(grep version Cargo.toml | head -1 | awk '{print $3}' | tr -d '"')
echo "Package: your-rust-cli
Version: $version
Section: utils
Priority: optional
Architecture: $architecture
Depends: libc6
Maintainer: Your Name <your.email@example.com>
Description: Your Rust CLI - A command-line interface for doing something amazing." > debian/DEBIAN/control

# build the Debian package
dpkg-deb --build debian
mv debian.deb your-rust-cli_$version_$architecture.deb