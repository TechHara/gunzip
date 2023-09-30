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
    DistanceTooMuch,
    EndOfBlockNotFound,
    ReadDynamicCodebook,
    ChecksumMismatch,
    SizeMismatch
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::StdIoError(e.kind())
    }
}

impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::StdIoError(e) => Self::from(e),
            e => Self::new(std::io::ErrorKind::Other, e),
        }
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type Result<R> = std::result::Result<R, Error>;
