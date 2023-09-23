pub mod error;
pub mod bitread;
pub mod header;
pub mod footer;
pub mod inflate;
pub mod codebook;
pub mod huffman_decoder;
pub mod lz77;
pub mod sliding_window;
pub mod checksum_write;

use inflate::Inflate;
use crate::error::Error;

use bitread::{BitReader, BitRead};
use header::Header;
use footer::Footer;

use crate::error::Result;
use std::io::{Read, Write};

use checksum_write::{ChecksumWrite, Crc32Writer};

pub fn gunzip(read: impl Read, write: impl Write) -> Result<()> {
    let mut reader = BitReader::new(read);
    let mut writer = Crc32Writer::new(write);
    let mut member_idx = 0;

    while reader.has_data_left()? {
        Header::read(&mut reader)?;
        member_idx += 1;

        // read one or more blocks
        Inflate::new(&mut reader, &mut writer).run()?;

        let footer = Footer::read(&mut reader)?;
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

    if member_idx == 0 {
        Err(Error::EmptyInput)
    } else {
        Ok(())
    }
}