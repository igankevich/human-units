#!/bin/sh

. ./ci/preamble.sh

cargo_publish() {
    cargo publish --package human-units-derive --features serde --quiet
    cargo publish --package human-units --features serde,si-units,iec-units,derive,std --quiet
}

if test "$GITHUB_ACTIONS" = "true" && test "$GITHUB_REF_TYPE" != "tag"; then
    exit 0
fi
cargo_publish
