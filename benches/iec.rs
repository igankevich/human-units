#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use pastey::paste;
use std::hint::black_box;

macro_rules! parameterize {
    ($($uint: ident)+) => {
        paste! {
            $(
                pub const fn [<$uint _is_multiple_of>](a: $uint, b: $uint) -> bool {
                    match b {
                        0 => a == 0,
                        _ => a % b == 0,
                    }
                }
            )+
        }
    };
}

parameterize! {
    u128
    u64
    u32
    u16
}

macro_rules! bench {
    ($c: ident, $func: ident, $value: expr) => {
        $c.bench_function(stringify!($func), |b| b.iter(|| $func(black_box($value))));
    };
}

macro_rules! parameterize {
    ($((
        $uint: ident
        $max_prefix: ident
        ($max_prefix_integer: expr)
        ($max_integer: expr)
        ($($ilog: expr)+)
    ))+) => {
        paste! {
            $(
                fn [<unitify_ $uint>](mut value: $uint) -> ($uint, usize) {
                    if value == 0 {
                        return (0, Prefix::None as usize);
                    }
                    for prefix in Prefix::None as usize..Prefix::$max_prefix as usize {
                        if ![<$uint _is_multiple_of>](value, 1024) {
                            return (value, prefix);
                        }
                        value >>= 10;
                    }
                    (value, Prefix::$max_prefix as usize)
                }

                fn [<unitify_ $uint _v2>](value: $uint) -> ($uint, usize) {
                    if value == 0 {
                        return (0, Prefix::None as usize);
                    }
                    $(
                        {
                            const POW: $uint = (1024 as $uint).pow($ilog);
                            if [<$uint _is_multiple_of>](value, POW) {
                                return (value >> (10 * $ilog), $ilog);
                            }
                        }
                    )+
                    (value, Prefix::$max_prefix as usize)
                }
            )+

            fn criterion_benchmark(c: &mut Criterion) {
                $(
                    {
                        const MAX_POW_OF_2: $uint = (2 as $uint).pow($uint::MAX.ilog2());
                        let mut group = c.benchmark_group(concat!("iec::", stringify!($uint)));
                        bench!(group, [<unitify_ $uint>], MAX_POW_OF_2);
                        bench!(group, [<unitify_ $uint _v2>], 1);
                        group.finish();
                    }
                )+
            }
        }
    };
}

parameterize! {
    (u128 Quebi (1024*1024) (1024*1024*1024 - 1) (10 9 8 7 6 5 4 3 2 1))
    (u64 Exbi (1) (1023) (6 5 4 3 2 1))
    (u32 Gibi (1) (1023) (3 2 1))
    (u16 Kibi (1) (1023) (1))
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(all(test, feature = "std"), derive(arbitrary::Arbitrary))]
#[repr(u8)]
#[allow(dead_code)]
enum Prefix {
    #[default]
    None = 0,
    Kibi = 1,
    Mebi = 2,
    Gibi = 3,
    Tebi = 4,
    Pebi = 5,
    Exbi = 6,
    Zebi = 7,
    Yobi = 8,
    Robi = 9,
    Quebi = 10,
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
