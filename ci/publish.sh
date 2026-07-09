#!/bin/sh

. ./ci/preamble.sh

cargo publish --package human-units-derive --features serde --quiet
cargo publish --package human-units --features serde,si-units,iec-units,derive,std --quiet
