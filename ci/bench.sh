#!/bin/sh

. ./ci/preamble.sh

benchmark_performance() {
    cargo +nightly bench
}

benchmark_executable_size() {
    rm -rf "$workdir"/tmp
    mkdir "$workdir"/tmp
    root="$PWD"
    cd "$workdir"/tmp
    cargo init --quiet
    # https://github.com/johnthagen/min-sized-rust
    {
        grep human-repr "$root"/Cargo.toml
        grep human_bytes "$root"/Cargo.toml
    } >>Cargo.toml
    cat >>Cargo.toml <<EOF
human-units = { path = "$root" }

[profile.release]
strip = true
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
EOF
    get_executable_size <<'EOF'
fn main() {}
EOF
    size_empty="$size"
    get_executable_size <<'EOF'
use human_units::FormatSize;
fn main() {
    println!("{}", u64::MAX.format_size());
}
EOF
    size_human_units="$size"
    get_executable_size <<'EOF'
use human_repr::HumanCount;
fn main() {
    println!("{}", u64::MAX.human_count_bytes());
}
EOF
    size_human_repr="$size"
    get_executable_size <<'EOF'
use human_bytes::human_bytes;
fn main() {
    println!("{}", human_bytes(u64::MAX as f64));
}
EOF
    size_human_bytes="$size"
    python3 <<EOF
baseline = $size_empty
size_human_bytes = $size_human_bytes - baseline
size_human_repr = $size_human_repr - baseline
size_human_units = $size_human_units - baseline
print('human-repr {}'.format(size_human_repr))
print('human-units {}'.format(size_human_units))
print('human_bytes {}'.format(size_human_bytes))
EOF
    get_executable_size <<'EOF'
use core::time::Duration;
use human_units::FormatDuration;
fn main() {
    println!("{}", Duration::new(u64::MAX, 999_999_999_u32).format_duration());
}
EOF
    size_human_units="$size"
    get_executable_size <<'EOF'
use core::time::Duration;
use human_repr::HumanDuration;
fn main() {
    println!("{}", Duration::new(u64::MAX, 999_999_999_u32).human_duration());
}
EOF
    size_human_repr="$size"
    python3 <<EOF
baseline = $size_empty
size_human_repr = $size_human_repr - baseline
size_human_units = $size_human_units - baseline
print('human-repr {}'.format(size_human_repr))
print('human-units {}'.format(size_human_units))
EOF
    cd "$root"
}

get_executable_size() {
    cat >src/main.rs
    cargo build --release --quiet
    size="$(stat --format='%s' target/release/tmp)"
}

# TODO
#benchmark_performance
benchmark_executable_size
