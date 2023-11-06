use crate::bitread::ReadUntil;
use crate::error::{Error, Result};
use std::io::Read;

const ID1: u8 = 0x1f;
const ID2: u8 = 0x8b;
const DEFLATE: u8 = 8;
const FTEXT: u8 = 1;
const FHCRC: u8 = 2;
const FEXTRA: u8 = 4;
const FNAME: u8 = 8;
const FCOMMENT: u8 = 16;

pub struct Header {
    pub header: [u8; 10],
    pub extra_field: Option<Vec<u8>>,
    pub name: Option<Vec<u8>>,
    pub comment: Option<Vec<u8>>,
    pub crc16: Option<u16>,
    pub size: usize, // header size
}

impl Header {
    pub fn read(mut reader: impl Read + ReadUntil) -> Result<Self> {
        let mut buf = [0; 10];
        reader.read_exact(&mut buf)?;
        if buf[0..3] != [ID1, ID2, DEFLATE] || buf[3] & 0b11100000 != 0 {
            return Err(Error::InvalidGzHeader);
        }

        let mut header = Self {
            header: buf,
            extra_field: None,
            name: None,
            comment: None,
            crc16: None,
            size: buf.len(),
        };

        if header.get_flg() & FEXTRA != 0 {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf)?;
            header.size += buf.len();
            let n = u16::from_le_bytes(buf);
            let mut buf = vec![0u8; n as usize];
            reader.read_exact(&mut buf)?;
            header.size += buf.len();
            header.extra_field = Some(buf);
        }
        if header.get_flg() & FNAME != 0 {
            let mut buf = Vec::new();
            header.size += reader.read_until(0, &mut buf)?;
            header.name = Some(buf);
        }
        if header.get_flg() & FCOMMENT != 0 {
            let mut buf = Vec::new();
            header.size += reader.read_until(0, &mut buf)?;
            header.comment = Some(buf);
        }
        if header.get_flg() & FHCRC != 0 {
            let mut buf = [0u8, 2];
            reader.read_exact(&mut buf)?;
            header.size += buf.len();
            header.crc16 = Some(u16::from_le_bytes(buf));
        }

        Ok(header)
    }

    fn get_flg(&self) -> u8 {
        self.header[3]
    }
}
