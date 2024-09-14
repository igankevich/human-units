use core::str::from_utf8_unchecked;

pub(crate) struct Buffer<const N: usize> {
    data: [u8; N],
    position: usize,
}

impl<const N: usize> Buffer<N> {
    pub(crate) fn new() -> Self {
        Self {
            data: [0_u8; N],
            position: 0,
        }
    }

    pub(crate) fn write_u64(&mut self, mut n: u64, mut p10: u64) {
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

    pub(crate) fn write_byte(&mut self, ch: u8) {
        self.data[self.position] = ch;
        self.position += 1;
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.data[..self.position]
    }

    pub(crate) unsafe fn as_str(&self) -> &str {
        from_utf8_unchecked(self.as_slice())
    }
}

impl<const N: usize> core::fmt::Write for Buffer<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let n = bytes.len().min(N - self.position);
        self.data[self.position..(self.position + n)].copy_from_slice(&bytes[..n]);
        self.position += n;
        Ok(())
    }
}
