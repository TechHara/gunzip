use std::fmt::Display;
use std::io::ErrorKind;

#[derive(Debug)]
pub enum Error {
    StdIoError(ErrorKind),
    EmptyInput,
    InvalidGzHeader,
    InvalidBlockType,
    BlockType0LenMismatch,
    InvalidCodeLengths,
    HuffmanDecoderCodeNotFound,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::StdIoError(e.kind())
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type Result<R> = std::result::Result<R, Error>;
