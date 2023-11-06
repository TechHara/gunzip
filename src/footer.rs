use crate::error::Result;
use std::io::Read;

pub struct Footer {
    pub crc32: u32,
    pub size: u32,
}

impl Footer {
    pub fn read(mut read: impl Read) -> Result<Self> {
        let mut buf = [0u8; 8];
        read.read_exact(&mut buf)?;
        let (a, b) = buf.split_at(4);
        let crc32 = u32::from_le_bytes(a.try_into().unwrap());
        let size = u32::from_le_bytes(b.try_into().unwrap());
        Ok(Self { crc32, size })
    }
}
