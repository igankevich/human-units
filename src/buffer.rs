#![doc(hidden)]

use core::str::from_utf8_unchecked;

pub struct Buffer<const N: usize> {
    data: [u8; N],
    position: usize,
}

impl<const N: usize> Buffer<N> {
    pub const fn new() -> Self {
        Self {
            data: [0_u8; N],
            position: 0,
        }
    }

    pub fn write_byte(&mut self, ch: u8) {
        self.data[self.position] = ch;
        self.position += 1;
    }

    pub fn write_str_infallible(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let n = bytes.len().min(N - self.position);
        self.data[self.position..(self.position + n)].copy_from_slice(&bytes[..n]);
        self.position += n;
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.position]
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

macro_rules! define_write {
    ($uint: ident, $func: ident) => {
        impl<const N: usize> Buffer<N> {
            pub fn $func(&mut self, mut n: $uint, mut p10: $uint) {
                let mut written = false;
                while p10 != 0 && n < p10 {
                    p10 /= 10;
                }
                while p10 != 0 {
                    let d = n / p10;
                    if written || d != 0 {
                        self.write_byte(b'0' + d as u8);
                        written = true;
                    }
                    n -= d * p10;
                    p10 /= 10;
                }
                if !written {
                    self.write_byte(b'0');
                }
            }
        }
    };
}

define_write!(u128, write_u128);
define_write!(u64, write_u64);
define_write!(u32, write_u32);
define_write!(u16, write_u16);

impl<const N: usize> core::fmt::Write for Buffer<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_str_infallible(s);
        Ok(())
    }
}
