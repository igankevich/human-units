#!/bin/sh

apt_get() {
    env DEBIAN_FRONTEND=noninteractive sudo -n apt-get -qq "$@"
}

. ./ci/preamble.sh

apt_get update
apt_get install --no-install-recommends \
    libclang-dev \
    shellcheck \
    clang \
    curl \
    lcov

rustup toolchain add nightly \
    --target x86_64-unknown-linux-gnu \
    --component rustfmt,miri,llvm-tools-preview
rustup toolchain add stable \
    --target x86_64-unknown-linux-gnu \
    --component clippy,rustfmt,llvm-tools-preview
rustup default stable
cargo install --quiet cargo-msrv
