#!/bin/sh

. ./ci/preamble.sh

git config --global --add safe.directory "$PWD"
cargo clippy --quiet --all-targets --workspace --features si-units,std,derive,serde -- -D warnings
for file in ci/*.sh; do
    shellcheck --external-sources "$file"
done
