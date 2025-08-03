/// An approximate value that consists of integer and fraction parts, prefix and symbol.
pub struct FormattedUnit<'symbol, T, const N: usize> {
    pub(crate) prefix: &'static str,
    pub(crate) symbol: &'symbol str,
    pub(crate) integer: T,
    pub(crate) fraction: u8,
}

impl<'symbol, T, const N: usize> FormattedUnit<'symbol, T, N> {
    /// Create new instance.
    #[doc(hidden)]
    pub const fn new(prefix: &'static str, symbol: &'symbol str, integer: T, fraction: u8) -> Self {
        Self {
            prefix,
            symbol,
            integer,
            fraction,
        }
    }

    /// Unit prefix.
    pub const fn prefix(&self) -> &'static str {
        self.prefix
    }

    /// Unit symbol.
    pub const fn symbol(&self) -> &'symbol str {
        self.symbol
    }

    /// Integer part.
    pub const fn integer(&self) -> T
    where
        T: Copy,
    {
        self.integer
    }

    /// Fraction part. Max. value is 9.
    pub const fn fraction(&self) -> u8 {
        self.fraction
    }
}
