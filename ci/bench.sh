#!/bin/sh

. ./ci/preamble.sh

cargo +nightly bench
