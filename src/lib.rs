pub mod error;
pub mod bitread;
pub mod header;
pub mod footer;
pub mod codebook;
pub mod huffman_decoder;
pub mod lz77;
pub mod sliding_window;
pub mod checksum_write;
pub mod producer;

use crate::{error::Error, producer::Producer};

use header::Header;
use footer::Footer;

use crate::error::Result;
use std::io::{Read, Write};

use checksum_write::{ChecksumWrite, Crc32Writer};

pub enum Produce {
    Header(Header),
    Footer(Footer),
    Data(Vec<u8>),
    Err(Error),
}

pub fn gunzip(read: impl Read + Send + 'static, write: impl Write, multithread: bool) -> Result<()> {
    // producer: takes Read and produces data block
    // consumer: takes Write and consumes data block

    let iter = if multithread {
        let (tx, rx) = std::sync::mpsc::channel::<Produce>();
        std::thread::spawn(move || {
            for produce in Producer::new(read) {
                tx.send(produce).expect("error while transmitting produce over the channel");
            }
        });

        Box::new(rx.into_iter()) as Box<dyn Iterator<Item = Produce>>
    } else {
        Box::new(Producer::new(read))
    };
    
    let mut writer = Crc32Writer::new(write);
    
    for xs in iter {
        match xs {
            Produce::Header(_) => (),
            Produce::Footer(footer) => {
                let checksum = writer.checksum();
                let size = writer.len();
        
                if footer.crc32 != checksum {
                    return Err(Error::ChecksumMismatch);
                }
        
                if footer.size as usize != size & 0xFFFFFFFF {
                    return Err(Error::SizeMismatch);
                }

                writer.reset_len();
            }
            Produce::Data(xs) => {
                writer.write_all(&xs)?;
            }
            Produce::Err(e) => {
                return Err(e);
            }
        }
    }
    Ok(())
}