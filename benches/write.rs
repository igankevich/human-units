#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use human_units::Buffer;
use paste::paste;

fn criterion_benchmark(c: &mut Criterion) {
    macro_rules! bench {
        ($($uint: ident,)+) => {
            paste! {
                $(
                    for value in [10 as $uint, $uint::MAX / 2, $uint::MAX] {
                        c.bench_with_input(BenchmarkId::new(concat!(
                            "Buffer::",
                            stringify!([<write_ $uint>]),
                        ), value), &value, |b, &value| {
                            b.iter(|| {
                                let mut buf = Buffer::<64>::new();
                                buf.[<write_ $uint>](value)
                            })
                        });
                    }
                )+
            }
        };
    }
    bench! { u128, u64, u32, u16, };
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
