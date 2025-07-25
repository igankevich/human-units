#!/bin/sh

. ./ci/preamble.sh

git config --global --add safe.directory "$PWD"
cargo clippy --quiet --all-targets --workspace --features si-units,iec-units,std,derive,serde -- -D warnings
cargo msrv --workspace verify
for file in ci/*.sh; do
    shellcheck --external-sources "$file"
done
