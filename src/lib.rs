#![cfg_attr(feature = "no_std", no_std)]
/*!
# Introduction

This crate adds [`Size`] and [`Duration`] types specifically designed to be used in configuration files and as command line arguments.
These types serialize sizes and durations in _exact_ but human-readable form.

The library also provides [`FormatSize`] and [`FormatDuration`] traits
to print _approximate_ sizes and durations in a short human-readable form.

- No floating point operations.
- No dependencies by default.
- Supports [Serde](https://serde.rs/).
- Supports [`no_std`](https://docs.rust-embedded.org/book/intro/no-std.html).
- Tested with [Miri](https://github.com/rust-lang/miri).
- **72–85%** faster than similar libraries (see benchmarks below).


# Examples

## Exact human-readable size/duration

```
use human_units::{Duration, Size};
assert_eq!("1k", Size(1024).to_string());
assert_eq!("1025", Size(1025).to_string());
assert_eq!("1m", Duration(core::time::Duration::from_secs(60)).to_string());
assert_eq!("61s", Duration(core::time::Duration::from_secs(61)).to_string());
```

## Inexact short human-readable size/duration

```
use core::time::Duration;
use human_units::{FormatDuration, FormatSize};
assert_eq!("1 KiB", 1024_u64.format_size().to_string());
assert_eq!("1 m", Duration::from_secs(60).format_duration().to_string());
```

## Custom output

```
use colored::Colorize;
use core::time::Duration;
use human_units::{FormatDuration, FormattedDuration};

/// Prints the unit in cyan.
struct ColoredDuration(FormattedDuration);

impl core::fmt::Display for ColoredDuration {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0.integer)?;
        if self.0.fraction != 0 {
            write!(f, ".{}", self.0.fraction)?;
        }
        write!(f, " {}", self.0.unit.cyan())
    }
}

// prints "1 m ago", "m" is printed with cyan color
println!("{} ago", ColoredDuration(Duration::from_secs(60).format_duration()));
```

## Serde integration

```
use human_units::Size;
use serde::Serialize;

#[derive(Serialize, PartialEq, Eq, Debug)]
struct SizeWrapper {
    size: Size,
}

let object = SizeWrapper{ size: Size(1024) };
assert_eq!(r#"{"size":"1k"}"#, serde_json::to_string(&object).unwrap());
```

# Benchmarks

Benchmarks were done with Rust 1.80.1 on a x86\_64 laptop.

## Format size

| Library | Version | Features | Benchmark | Time |
|---------|---------|----------|-----------|------|
| `human_bytes` | 0.4.3 | `fast`       | `format_size_then_to_string` | 88.40 ns ± 5.02 ns|
| `human-repr`  | 1.1.0 | `1024,space` | `format_size_then_to_string` | 161.38 ns ± 13.29 ns|
| `human-units` | 0.1.3 |              | `format_size_then_to_string` | **24.24 ns ± 1.23 ns** |

## Format duration

| Library | Version | Features | Benchmark | Time |
|---------|---------|----------|-----------|------|
| `human-repr`  | 1.1.0 | `1024,space` | `format_duration_then_to_string` | 229.47 ns ± 11.90 ns|
| `human-units` | 0.1.3 |              | `format_duration_then_to_string` | **41.55 ns ± 2.77 ns** |

*/

// #[cfg(feature = "serde")]
mod buffer;
mod duration;
mod duration_format;
#[cfg(feature = "serde")]
mod duration_serde;
mod size;
mod size_format;
#[cfg(feature = "serde")]
mod size_serde;

//#[cfg(feature = "serde")]
pub(crate) use self::buffer::*;
pub use self::duration::*;
pub use self::duration_format::*;
pub use self::size::*;
pub use self::size_format::*;
