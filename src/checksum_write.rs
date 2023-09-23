use std::io::Write;
use crc32fast::Hasher;

pub trait ChecksumWrite: Write {
    /// resets the checksum upon this call
    fn checksum(&mut self) -> u32;

    /// Total # bytes wrote so far
    fn len(&self) -> usize;

    /// Reset length to be 0
    fn reset_len(&mut self);
}

impl<W: ChecksumWrite> ChecksumWrite for &mut W {
    fn checksum(&mut self) -> u32 {
        (**self).checksum()
    }

    fn len(&self) -> usize {
        (**self).len()
    }

    fn reset_len(&mut self) {
        (**self).reset_len()
    }
}

pub struct Crc32Writer<W: Write> {
    hasher: Hasher,
    writer: W,
    n: usize,
}

impl<W: Write> Crc32Writer<W> {
    pub fn new(writer: W) -> Self {
        Self {
            hasher: Hasher::new(),
            writer,
            n: 0,
        }
    }
}

impl<W: Write> ChecksumWrite for Crc32Writer<W> {
    fn checksum(&mut self) -> u32 {
        let hasher = std::mem::replace(&mut self.hasher, Hasher::new());
        hasher.finalize()
    }

    fn len(&self) -> usize {
        self.n
    }

    fn reset_len(&mut self) {
        self.n = 0;
    }
}

impl<W: Write> Write for Crc32Writer<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);
        self.n += n;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}