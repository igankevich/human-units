#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use paste::paste;
use std::hint::black_box;

macro_rules! bench {
    ($c: ident, $func: ident, $value: expr) => {
        $c.bench_function(stringify!($func), |b| b.iter(|| $func(black_box($value))));
    };
}

macro_rules! parameterize {
    ($($uint: ident $(, $ilog: literal)+ ;)+) => {
        paste! {
            $(
                fn [<format_unit_ $uint>](value: $uint) -> (u16, u8, u32) {
                    let mut i: u32 = 0;
                    let mut scale: $uint = 1;
                    let mut n = value;
                    while n >= 1000 {
                        scale *= 1000;
                        n /= 1000;
                        i += 1;
                    }
                    let mut b = value % scale;
                    if b != 0 {
                        // Compute the first digit of the fractional part.
                        b /= (scale / 10);
                    }
                    let integer = n;
                    let fraction = b;
                    (integer as u16, fraction as u8, i)
                }

                fn [<format_unit_ $uint _v2>](value: $uint) -> (u16, u8, u32) {
                    let i: u32 = value.ilog(1000);
                    let scale: $uint = (1000 as $uint).pow(i);
                    let integer = value / scale;
                    let mut fraction = value % scale;
                    if fraction != 0 {
                        // Compute the first digit of the fractional part.
                        fraction /= (scale / 10);
                    }
                    (integer as u16, fraction as u8, i)
                }

                fn [<format_unit_ $uint _v3>](value: $uint) -> (u16, u8, u32) {
                    let (scale, i) = [<scale_ $uint>](value);
                    let integer = value / scale;
                    let mut fraction = value % scale;
                    if fraction != 0 {
                        // Compute the first digit of the fractional part.
                        fraction /= (scale / 10);
                    }
                    (integer as u16, fraction as u8, i)
                }

                fn [<format_unit_ $uint _v4>](value: $uint) -> (u16, u8, u32) {
                    $(
                        {
                            const SCALE: $uint = (1000 as $uint).pow($ilog);
                            if value >= SCALE {
                                let integer = value / SCALE;
                                let mut fraction = value % SCALE;
                                if fraction != 0 {
                                    // Compute the first digit of the fractional part.
                                    fraction /= (SCALE / 10);
                                }
                                return (integer as u16, fraction as u8, $ilog)
                            }
                        }
                    )+
                    (value as u16, 0, 0)
                }

                #[inline]
                fn [<scale_ $uint>](value: $uint) -> ($uint, u32) {
                    $(
                        {
                            const SCALE: $uint = (1000 as $uint).pow($ilog);
                            if value >= SCALE {
                                return (SCALE, $ilog);
                            }
                        }
                    )+
                    (1, 0)
                }
            )+

            fn criterion_benchmark(c: &mut Criterion) {
                $(
                    {
                        const MAX_POW_OF_10: $uint = (10 as $uint).pow($uint::MAX.ilog(10));
                        let mut group = c.benchmark_group(stringify!($uint));
                        bench!(group, [<format_unit_ $uint>], MAX_POW_OF_10);
                        bench!(group, [<format_unit_ $uint _v2>], MAX_POW_OF_10);
                        bench!(group, [<format_unit_ $uint _v3>], 10);
                        bench!(group, [<format_unit_ $uint _v4>], 10);
                        group.finish();
                    }
                )+
            }
        }
    };
}

parameterize! {
    u128, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1;
    u64, 6, 5, 4, 3, 2, 1;
    u32, 3, 2, 1;
    u16, 1;
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
