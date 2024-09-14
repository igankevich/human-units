#!/bin/sh

. ./ci/preamble.sh

cargo +nightly --quiet miri setup
env MIRIFLAGS=-Zmiri-disable-isolation cargo +nightly miri test
