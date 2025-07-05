#![doc(hidden)]

use core::mem::transmute;
use core::mem::MaybeUninit;
use core::str::from_utf8_unchecked;
use paste::paste;

pub struct Buffer<const N: usize> {
    data: [MaybeUninit<u8>; N],
    position: usize,
}

impl<const N: usize> Buffer<N> {
    pub const fn new() -> Self {
        Self {
            data: [const { MaybeUninit::uninit() }; N],
            position: 0,
        }
    }

    pub fn write_byte(&mut self, ch: u8) {
        self.data[self.position].write(ch);
        self.position += 1;
    }

    pub fn write_str_infallible(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let n = bytes.len().min(N - self.position);
        let src = &mut self.data[self.position..(self.position + n)];
        let uninit_src: &mut [u8] = unsafe { transmute(src) };
        uninit_src.copy_from_slice(&bytes[..n]);
        self.position += n;
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { transmute(&self.data[..self.position]) }
    }

    pub unsafe fn as_str(&self) -> &str {
        from_utf8_unchecked(self.as_slice())
    }
}

impl<const N: usize> Default for Buffer<N> {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! parameterize_width {
    // Linear search.
    (linear_search, $uint: ident, ($($ilog_left: expr,)+), $ilog_midpoint: expr, ($($ilog_right: expr,)+)) => {
        paste! {
            #[inline]
            const fn [<width_ $uint>](value: $uint) -> u8 {
                $(
                    {
                        const POW: $uint = (10 as $uint).pow($ilog_left);
                        if value <= POW {
                            return $ilog_left;
                        }
                    }
                )+
                $(
                    {
                        const POW: $uint = (10 as $uint).pow($ilog_right);
                        if value <= POW {
                            return $ilog_right;
                        }
                    }
                )+
                $uint::MAX.ilog10() as u8 + 1
            }
        }
    };
    // One step of bisection.
    (bisection_1, $uint: ident, ($($ilog_left: expr,)+), $ilog_midpoint: expr, ($($ilog_right: expr,)+)) => {
        paste! {
            #[inline]
            const fn [<width_ $uint>](value: $uint) -> u8 {
                const MIDPOINT: $uint = (10 as $uint).pow($ilog_midpoint).next_power_of_two();
                if value <= MIDPOINT {
                    $(
                        {
                            const POW: $uint = (10 as $uint).pow($ilog_left);
                            if value < POW {
                                return $ilog_left;
                            }
                        }
                    )+
                    $ilog_midpoint as u8 + 1
                } else {
                    $(
                        {
                            const POW: $uint = (10 as $uint).pow($ilog_right);
                            if value < POW {
                                return $ilog_right;
                            }
                        }
                    )+
                    $uint::MAX.ilog10() as u8 + 1
                }
            }
        }
    };
    // Simple ilog10.
    (ilog10, $uint: ident, ($($ilog_left: expr,)+), $ilog_midpoint: expr, ($($ilog_right: expr,)+)) => {
        paste! {
            #[inline]
            const fn [<width_ $uint>](value: $uint) -> u8 {
                value.ilog10() as u8 + 1
            }
        }
    };
}

macro_rules! parameterize {
    ($(
        $uint: ident,
        $algo: ident,
        (($($ilog_left: expr,)+), $ilog_midpoint: expr, ($($ilog_right: expr,)+)),
    )+) => {
        paste! {
            $(
                impl<const N: usize> Buffer<N> {
                    pub fn [<write_ $uint>](&mut self, mut n: $uint) {
                        if n == 0 {
                            self.write_byte(b'0');
                            return;
                        }
                        let width = [<width_ $uint>](n);
                        let old_position = self.position;
                        self.position += width as usize;
                        for i in (old_position..self.position).rev() {
                            let digit = n % 10;
                            n /= 10;
                            self.data[i].write(b'0' + digit as u8);
                        }
                    }
                }

                parameterize_width! {
                    $algo,
                    $uint,
                    ($($ilog_left,)+),
                    $ilog_midpoint,
                    ($($ilog_right,)+)
                }
            )+

            #[cfg(test)]
            mod buffer_tests {
                use super::*;

                use arbtest::arbtest;

                $(
                    #[cfg(feature = "std")]
                    #[test]
                    fn [<test_write_ $uint>]() {
                        arbtest(|u| {
                            let number: $uint = u.arbitrary()?;
                            let expected = format!("{}", number);
                            let mut buf = Buffer::<64>::new();
                            buf.[<write_ $uint>](number);
                            let actual = unsafe { buf.as_str() };
                            assert_eq!(expected, actual, "number = {number}");
                            Ok(())
                        }).seed(0x7cf5f7aa00010000);
                    }
                )+
            }
        }
    };
}

parameterize! {
    u128, bisection_1,
    ((1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,),
    20,
    (20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38,)),
    u64, bisection_1,
    ((1, 2, 3, 4, 5, 6, 7, 8, 9,), 10, (10, 11, 12, 13, 14, 15, 16, 17, 18, 19,)),
    u32, ilog10, ((1, 2, 3, 4,), 5, (5, 6, 7, 8, 9,)),
    u16, bisection_1, ((1, 2,), 3, (3, 4,)),
}

impl<const N: usize> core::fmt::Write for Buffer<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_str_infallible(s);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_power_of_10() {
        macro_rules! check {
            ($uint: ident, $ilog: expr, $ilog_midpoint: expr) => {{
                paste! {
                    const ILOG: u32 = $uint::MAX.ilog(10);
                    assert_eq!(ILOG, $ilog);
                    const MAX_POWER_OF_10: $uint = (10 as $uint).pow(ILOG);
                    assert_eq!(None, MAX_POWER_OF_10.checked_mul(10));
                    const MIDPOINT: $uint = (10 as $uint).pow($ilog_midpoint);
                    assert_eq!($ilog_midpoint, MIDPOINT.next_power_of_two().ilog10());
                }
            }};
        }
        check!(u16, 4, 3);
        check!(u32, 9, 5);
        check!(u64, 19, 10);
        check!(u128, 38, 20);
    }
}
