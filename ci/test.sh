#!/bin/sh

. ./ci/preamble.sh

main() {
    clean
    export ARBTEST_BUDGET_MS=10000
    #test_coverage_preamble
    test_all --workspace --no-default-features --features derive,si-units,iec-units,serde,std --lib
    test_all --workspace --no-default-features --features derive,si-units,iec-units,serde --lib
    export ARBTEST_BUDGET_MS=200
    test_all --package human-units-tests
    #test_coverage_postamble
    unset ARBTEST_BUDGET_MS
    test_miri
}

test_miri() {
    cargo +nightly miri setup --quiet
    do_test_miri --quiet --no-run
    do_test_miri
}

do_test_miri() {
    env MIRIFLAGS=-Zmiri-disable-isolation cargo +nightly \
        miri test --features derive,si-units,iec-units,serde --lib "$@"
}

clean() {
    find target -type f -name '*.profraw' -delete 2>/dev/null || true
    find target -type f -name '*.gcda' -delete 2>/dev/null || true
}

test_all() {
    cargo test --quiet --no-run "$@"
    cargo test --no-fail-fast "$@"
}

test_coverage_preamble() {
    export CARGO_INCREMENTAL=0
    export RUSTFLAGS='-Zprofile -Ccodegen-units=1 -Cllvm-args=--inline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort -Zpanic_abort_tests'
    export LLVM_PROFILE_FILE="human-units-%p-%m.profraw"
}

test_coverage_postamble() {
    grcov \
        . \
        --binary-path target/debug/ \
        -s . \
        -t lcov \
        --branch --ignore-not-existing --ignore "*index.crates.io*" \
        --excl-start '.*cfg\(.*test.*' \
        --excl-br-start '.*cfg\(.*test.*' \
        --excl-br-line '.*cfg.*test.*' \
        -o target/debug/lcov.info
    lcov --summary target/debug/lcov.info
    genhtml -o target/debug/coverage/ \
        --branch-coverage \
        --highlight \
        --ignore-errors source \
        --legend \
        target/debug/lcov.info
}

main "$@"
