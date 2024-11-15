pub mod bitread;
pub mod checksum;
pub mod codebook;
pub mod error;
pub mod footer;
pub mod header;
pub mod huffman_decoder;
pub mod lz77;
pub mod producer;
pub mod sliding_window;

use crate::{checksum::Checksum, error::Error};

use checksum::Crc32Checksum;
use producer::{Produce, Producer};

use std::io::Read;

pub struct SingleThreadDecompressor<R> {
    inner: Decompressor<Producer<R>>,
}

impl<R: Read> SingleThreadDecompressor<R> {
    pub fn new(read: R) -> Self {
        Self {
            inner: Decompressor::new(Producer::new(read)),
        }
    }
}

impl<R: Read> Read for SingleThreadDecompressor<R> {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

pub struct MultithreadDecompressor {
    inner: Decompressor<std::sync::mpsc::IntoIter<Produce>>,
}

impl MultithreadDecompressor {
    pub fn new<R: Read + Send + 'static>(read: R) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for produce in Producer::new(read) {
                tx.send(produce)
                    .expect("error while transmitting produce over the channel");
            }
        });

        Self {
            inner: Decompressor::new(rx.into_iter()),
        }
    }
}

impl Read for MultithreadDecompressor {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

struct Decompressor<I> {
    iter: I,
    buf: Vec<u8>,
    begin: usize,
    checksum: Crc32Checksum,
}

impl<I> Decompressor<I>
where
    I: Iterator<Item = Produce>,
{
    fn new(iter: I) -> Self {
        Self {
            iter,
            buf: vec![],
            begin: 0,
            checksum: Crc32Checksum::new(),
        }
    }

    fn fill_buf(&mut self) -> std::io::Result<usize> {
        loop {
            match self.iter.next() {
                Some(Produce::Err(e)) => {
                    return Err(e.into());
                }
                Some(Produce::Header(_)) => { /* nothing to do */ }
                Some(Produce::Data(xs)) => {
                    if xs.is_empty() {
                        continue;
                    }
                    self.checksum.update(&xs);
                    self.buf = xs;
                    self.begin = 0;
                    return Ok(self.buf.len());
                }
                Some(Produce::Footer(footer)) => {
                    if self.checksum.checksum() != footer.crc32 {
                        return Err(Error::ChecksumMismatch.into());
                    }

                    if self.checksum.len() & 0xFFFFFFFF != footer.size as usize {
                        return Err(Error::SizeMismatch.into());
                    }

                    self.checksum.reset_len();
                }
                None => return Ok(0),
            }
        }
    }
}

impl<I> Read for Decompressor<I>
where
    I: Iterator<Item = Produce>,
{
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        let mut nbytes = 0;
        loop {
            let n = buf.len().min(self.buf[self.begin..].len());
            buf[..n].copy_from_slice(&self.buf[self.begin..self.begin + n]);
            buf = &mut buf[n..];
            nbytes += n;
            self.begin += n;

            if buf.is_empty() || self.fill_buf()? == 0 {
                break;
            }
        }
        Ok(nbytes)
    }
}
