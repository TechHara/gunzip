pub mod error;

use crate::error::Result;
use std::io::{Read, Write};

pub fn gunzip(read: impl Read, write: impl Write) -> Result<()> {
    todo!()
}
