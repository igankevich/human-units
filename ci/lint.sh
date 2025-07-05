#!/bin/sh

. ./ci/preamble.sh

git config --global --add safe.directory "$PWD"
cargo clippy --all-targets --workspace --features si-units,std,derive,serde -- -D warnings
shellcheck --external-sources
