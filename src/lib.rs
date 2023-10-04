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

use crate::{checksum::Checksum, error::Error, producer::Producer};

use checksum::Crc32Checksum;
use footer::Footer;
use header::Header;

use std::io::Read;

pub enum Produce {
    Header(Header),
    Footer(Footer),
    Data(Vec<u8>),
    Err(Error),
}

pub struct Decompressor {
    iter: Box<dyn Iterator<Item = Produce>>,
    buf: Vec<u8>,
    begin: usize,
    checksum: Crc32Checksum,
}

impl Decompressor {
    pub fn new<R: Read + Send + 'static>(read: R, multithread: bool) -> Self {
        let iter = if multithread {
            let (tx, rx) = std::sync::mpsc::channel::<Produce>();
            std::thread::spawn(move || {
                for produce in Producer::new(read) {
                    tx.send(produce)
                        .expect("error while transmitting produce over the channel");
                }
            });

            Box::new(rx.into_iter()) as Box<dyn Iterator<Item = Produce>>
        } else {
            Box::new(Producer::new(read))
        };

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

impl Read for Decompressor {
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
