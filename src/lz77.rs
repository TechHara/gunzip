use crate::error::{Result, Error};

pub const END_OF_BLOCK: u32 = 256;
pub const MAX_DISTANCE: u16 = 1 << 15; // 32kB
pub const MAX_LENGTH: u16 = 258;

#[derive(PartialEq, Eq, Debug)]
pub enum Code {
    Literal(u8), // < 256
    EndOfBlock, // == 256
    Dictionary { distance: u16, length: u16 }, // 257..285
}

pub enum DecodeResult {
    Done(usize),         // all symbols are exhausted and written
    WindowIsFull(usize), // cannot proceed further because the window is full
    Error(Error),
}

pub fn decode(
    window: &mut [u8],
    boundary: usize,
    codes: &mut impl Iterator<Item = Result<Code>>,
) -> Result<DecodeResult> {
    let mut idx = boundary; // position to write to
    if idx + MAX_LENGTH as usize >= window.len() {
        return Ok(DecodeResult::WindowIsFull(idx - boundary));
    }
    for code in codes {
        match code? {
            Code::Literal(x) => {
                window[idx] = x;
                idx += 1;
            }
            Code::Dictionary { distance, length } => {
                let mut distance = distance as usize;
                let mut length = length as usize;

                if distance > idx {
                    return Err(Error::DistanceTooMuch);
                }

                let begin = idx - distance;
                while length > 0 {
                    let n = distance.min(length);
                    window.copy_within(begin..begin + n, idx);
                    idx += n;
                    length -= n;
                    distance += n;
                }
            }
            Code::EndOfBlock => {
                return Ok(DecodeResult::Done(idx - boundary));
            }
        }

        if idx + MAX_LENGTH as usize >= window.len() {
            return Ok(DecodeResult::WindowIsFull(idx - boundary));
        }
    }
    Err(Error::EndOfBlockNotFound)
}