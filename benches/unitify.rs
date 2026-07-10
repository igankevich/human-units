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

macro_rules! define_unitify {
    ($uint: ident, $func: ident $(, $prefix: ident)+ => $max_prefix: ident) => {
        paste! {
            pub(crate) fn $func(mut value: $uint) -> ($uint, usize) {
                if value == 0 {
                    return (0, Prefix::None as usize);
                }
                $(
                    if ![<$uint _is_multiple_of>](value, 1000) {
                        return (value, Prefix::$prefix as usize);
                    }
                    value /= 1000;
                )+
                (value, Prefix::$max_prefix as usize)
            }
        }
    };
}

define_unitify!(u128, unitify_u128, Nano, Micro, Milli, None, Kilo, Mega, Giga, Tera, Peta, Exa, Zetta, Yotta, Ronna => Quetta);
define_unitify!(u64, unitify_u64, Nano, Micro, Milli, None, Kilo, Mega => Giga);
define_unitify!(u32, unitify_u32, Nano, Micro, Milli => None);
define_unitify!(u16, unitify_u16, Nano => Micro);

const MAX_POW_OF_10_U128: u128 = 100_000_000_000_000_000_000_000_000_000_000_000_000;
const MAX_POW_OF_10_U64: u64 = 10_000_000_000_000_000_000;
const MAX_POW_OF_10_U32: u32 = 1_000_000_000;
const MAX_POW_OF_10_U16: u16 = 10_000;

macro_rules! define_unitify_naive {
    ($uint: ident, $func: ident, $min_prefix: ident, $max_prefix: ident) => {
        paste! {
            pub(crate) fn $func(mut value: $uint) -> ($uint, usize) {
                if value == 0 {
                    return (0, Prefix::None as usize);
                }
                for prefix in Prefix::$min_prefix as usize..Prefix::$max_prefix as usize {
                    if ![<$uint _is_multiple_of>](value, 1000) {
                        return (value, prefix);
                    }
                    value /= 1000;
                }
                (value, Prefix::$max_prefix as usize)
            }
        }
    };
}

define_unitify_naive!(u128, unitify_naive_u128, Nano, Quetta);
define_unitify_naive!(u64, unitify_naive_u64, Nano, Giga);
define_unitify_naive!(u32, unitify_naive_u32, Nano, None);
define_unitify_naive!(u16, unitify_naive_u16, Nano, Micro);

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Prefix {
    Nano = 7,
    Micro = 8,
    Milli = 9,
    #[default]
    None = 10,
    Kilo = 11,
    Mega = 12,
    Giga = 13,
    Tera = 14,
    Peta = 15,
    Exa = 16,
    Zetta = 17,
    Yotta = 18,
    Ronna = 19,
    Quetta = 20,
}

fn criterion_benchmark(c: &mut Criterion) {
    macro_rules! bench {
        ($func: ident, $value: ident) => {
            c.bench_function(stringify!($func), |b| b.iter(|| $func(black_box($value))));
        };
    }
    bench!(unitify_u128, MAX_POW_OF_10_U128);
    bench!(unitify_naive_u128, MAX_POW_OF_10_U128);
    bench!(unitify_u64, MAX_POW_OF_10_U64);
    bench!(unitify_naive_u64, MAX_POW_OF_10_U64);
    bench!(unitify_u32, MAX_POW_OF_10_U32);
    bench!(unitify_naive_u32, MAX_POW_OF_10_U32);
    bench!(unitify_u16, MAX_POW_OF_10_U16);
    bench!(unitify_naive_u16, MAX_POW_OF_10_U16);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
