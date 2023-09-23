pub mod error;
pub mod bitread;
pub mod header;
pub mod footer;
pub mod inflate;
pub mod codebook;
pub mod huffman_decoder;
pub mod lz77;
pub mod sliding_window;

use inflate::Inflate;
use crate::error::Error;

use bitread::{BitReader, BitRead};
use header::Header;
use footer::Footer;

use crate::error::Result;
use std::io::{Read, Write};

pub fn gunzip(read: impl Read, mut write: impl Write) -> Result<()> {    
    let mut reader = BitReader::new(read);
    let mut member_idx = 0;

    while reader.has_data_left()? {
        Header::read(&mut reader)?;
        member_idx += 1;
        
        // read one or more blocks
        Inflate::new(&mut reader, &mut write).run()?;

        let footer = Footer::read(&mut reader)?;
        // TODO: do something with footer
    }

    if member_idx == 0 {
        Err(Error::EmptyInput)
    } else {
        Ok(())
    }
}