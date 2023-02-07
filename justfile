# List the available reciples.
default:
    just --list

# Build `rpi-derive-key` for the specified TARGET.
build TARGET="aarch64-unknown-linux-musl":
    #!/bin/bash

    set -euo pipefail

    echo "Building for {{TARGET}}."

    if [ "{{os()}}" = "macos" ]; then
        export CC_armv7_unknown_linux_musleabihf=armv7-unknown-linux-musleabihf-gcc
        export CXX_armv7_unknown_linux_musleabihf=armv7-unknown-linux-musleabihf-g++
        export AR_armv7_unknown_linux_musleabihf=armv7-unknown-linux-musleabihf-ar
        export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=armv7-unknown-linux-musleabihf-gcc
        export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_STRIP=armv7-unknown-linux-musleabihf-strip

        export CC_aarch64_unknown_linux_musl=aarch64-unknown-linux-musl-gcc
        export CXX_aarch64_unknown_linux_musl=aarch64-unknown-linux-musl-g++
        export AR_aarch64_unknown_linux_musl=aarch64-unknown-linux-musl-ar
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-unknown-linux-musl-gcc
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_STRIP=aarch64-unknown-linux-musl-strip
    fi

    cargo build --bin rpi-derive-key --release --target {{TARGET}}

# Run `rpi-derive-key` with a fake device-specific secret.
run *ARGS:
    #!/bin/bash

    set -euo pipefail

    export FAKE_RPI_DERIVE_KEY_SECRET=debug

    cargo run -- {{ARGS}}