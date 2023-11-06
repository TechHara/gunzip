use crate::bitread::BitRead;
use crate::error::{Error, Result};
use crate::huffman_decoder::HuffmanDecoder;
use std::cmp::Ordering::*;

pub const END_OF_BLOCK: u32 = 256;
pub const MAX_DISTANCE: u16 = 1 << 15; // 32kB
pub const MAX_LENGTH: u16 = 258;

#[derive(PartialEq, Eq, Debug)]
pub enum Code {
    Literal(u8),                               // < 256
    EndOfBlock,                                // == 256
    Dictionary { distance: u16, length: u16 }, // 257..285
}

pub enum DecodeResult {
    Done(usize),         // all symbols are exhausted and written
    WindowIsFull(usize), // cannot proceed further because the window is full
    Error(Error),
}

pub fn decode<B: BitRead>(
    window: &mut [u8],
    boundary: usize,
    reader: &mut B,
    ll_decoder: &HuffmanDecoder,
    dist_decoder: &HuffmanDecoder,
) -> Result<DecodeResult> {
    let mut idx = boundary; // position to write to
    if idx + MAX_LENGTH as usize >= window.len() {
        return Ok(DecodeResult::WindowIsFull(idx - boundary));
    }

    loop {
        let code = read_next_code(reader, ll_decoder, dist_decoder)?;
        match code {
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
}

#[inline(always)]
fn read_next_code<B: BitRead>(
    reader: &mut B,
    ll_decoder: &HuffmanDecoder,
    dist_decoder: &HuffmanDecoder,
) -> Result<Code> {
    let bitcode = reader.peek_bits()?;
    let (symbol, len) = ll_decoder.decode(bitcode)?;
    reader.consume(len);
    match symbol.cmp(&END_OF_BLOCK) {
        Less => Ok(Code::Literal(symbol as u8)),
        Equal => Ok(Code::EndOfBlock),
        Greater => {
            let (bits, mut length) = SYMBOL2BITS_LENGTH[(symbol & 0xFF) as usize];
            length += reader.read_bits(bits)?;
            let bitcode = reader.peek_bits()?;
            let (symbol, len) = dist_decoder.decode(bitcode)?;
            reader.consume(len);
            let (bits, mut distance) = SYMBOL2BITS_DISTANCE[symbol as usize];
            distance += reader.read_bits(bits)?;
            Ok(Code::Dictionary {
                distance: distance as u16,
                length: length as u16,
            })
        }
    }
}

pub const SYMBOL2BITS_LENGTH: [(u32, u32); 30] = [
    (0, 0),
    (0, 3),
    (0, 4),
    (0, 5),
    (0, 6),
    (0, 7),
    (0, 8),
    (0, 9),
    (0, 10),
    (1, 11),
    (1, 13),
    (1, 15),
    (1, 17),
    (2, 19),
    (2, 23),
    (2, 27),
    (2, 31),
    (3, 35),
    (3, 43),
    (3, 51),
    (3, 59),
    (4, 67),
    (4, 83),
    (4, 99),
    (4, 115),
    (5, 131),
    (5, 163),
    (5, 195),
    (5, 227),
    (0, 258),
];

pub const SYMBOL2BITS_DISTANCE: [(u32, u32); 30] = [
    (0, 1),
    (0, 2),
    (0, 3),
    (0, 4),
    (1, 5),
    (1, 7),
    (2, 9),
    (2, 13),
    (3, 17),
    (3, 25),
    (4, 33),
    (4, 49),
    (5, 65),
    (5, 97),
    (6, 129),
    (6, 193),
    (7, 257),
    (7, 385),
    (8, 513),
    (8, 769),
    (9, 1025),
    (9, 1537),
    (10, 2049),
    (10, 3073),
    (11, 4097),
    (11, 6145),
    (12, 8193),
    (12, 12289),
    (13, 16385),
    (13, 24577),
];
