`human-units` is a library with `Size` and `Duration` types specifically designed to be used in configuration files and as command line arguments.
These types serialize sizes and durations in _exact_ but human-readable form.

The library also provides [`FormatSize`] and [`FormatDuration`] traits
to print _approximate_ sizes and durations in a short human-readable form.

- No floating point operations.
- No dependencies by default.
- Supports [Serde](https://serde.rs/).
- Supports [`no_std`](https://docs.rust-embedded.org/book/intro/no-std.html).
- Tested with [Miri](https://github.com/rust-lang/miri).

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
