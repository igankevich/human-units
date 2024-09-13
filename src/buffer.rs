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

    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.data[..self.position]
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
