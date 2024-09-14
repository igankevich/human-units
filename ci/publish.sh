#!/bin/sh

. ./ci/preamble.sh

install_token() {
    cat >/dev/shm/credentials.toml <<EOF
[repository]
token = "$CRATES_IO_TOKEN"
EOF
    ln -sfn /dev/shm/credentials.toml ~/.cargo/credentials.toml
}

cargo_publish() {
    cargo publish --all-features --dry-run
}

# TODO
#if test "$GITHUB_ACTIONS" = "true" && test "$GITHUB_REF_TYPE" != "tag"; then
#    exit 0
#fi
install_token
cargo_publish
