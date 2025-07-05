#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

mod human_units {
    use human_units::FormatDuration;
    use human_units::FormatSize;

    use super::black_box;
    use super::max_duration;
    use super::max_size;
    use super::Criterion;

    pub fn format_size(c: &mut Criterion) {
        c.bench_function("human_units::format_size", |b| {
            b.iter(|| black_box(max_size().format_size()))
        });
    }

    pub fn format_size_then_to_string(c: &mut Criterion) {
        c.bench_function("human_units::format_size_then_to_string", |b| {
            b.iter(|| black_box(max_size().format_size().to_string()))
        });
    }

    pub fn format_duration(c: &mut Criterion) {
        c.bench_function("human_units::format_duration", |b| {
            b.iter(|| black_box(max_duration().format_duration()))
        });
    }

    pub fn format_duration_then_to_string(c: &mut Criterion) {
        c.bench_function("human_units::format_duration_then_to_string", |b| {
            b.iter(|| black_box(max_duration().format_duration().to_string()))
        });
    }
}

mod human_bytes {
    use human_bytes::human_bytes;

    use super::black_box;
    use super::max_size;
    use super::Criterion;

    pub fn format_size_then_to_string(c: &mut Criterion) {
        c.bench_function("human_bytes::format_size_then_to_string", |b| {
            b.iter(|| black_box(human_bytes(max_size() as f64)))
        });
    }
}

mod human_repr {
    use human_repr::HumanCount;
    use human_repr::HumanDuration;

    use super::black_box;
    use super::max_duration;
    use super::max_size;
    use super::Criterion;

    pub fn format_duration_then_to_string(c: &mut Criterion) {
        c.bench_function("human_repr::format_duration_then_to_string", |b| {
            b.iter(|| black_box(max_duration().human_duration().to_string()))
        });
    }

    pub fn format_size_then_to_string(c: &mut Criterion) {
        c.bench_function("human_repr::format_size_then_to_string", |b| {
            b.iter(|| black_box(max_size().human_count_bytes().to_string()))
        });
    }
}

fn max_duration() -> core::time::Duration {
    black_box(core::time::Duration::new(u64::MAX, 999_999_999_u32))
}

fn max_size() -> u64 {
    black_box(u64::MAX)
}

criterion_group!(
    benches,
    human_units::format_size,
    human_units::format_size_then_to_string,
    human_units::format_duration,
    human_units::format_duration_then_to_string,
    human_bytes::format_size_then_to_string,
    human_repr::format_duration_then_to_string,
    human_repr::format_size_then_to_string,
);
criterion_main!(benches);
