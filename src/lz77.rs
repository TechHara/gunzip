use crate::bitread::BitRead;
use crate::error::{Error, Result};
use crate::huffman_decoder::HuffmanDecoder;
use crate::sliding_window::{WINDOW_MASK, WINDOW_SIZE};
use std::cmp::Ordering::*;

pub const END_OF_BLOCK: u32 = 256;
pub const MAX_DISTANCE: u16 = 1 << 15; // 32kB
pub const MAX_LENGTH: u16 = 258;

pub enum DecodeResult {
    Done(usize),         // all symbols are exhausted and written
    WindowIsFull(usize), // cannot proceed further because the window is full
    Error(Error),
}

pub fn decode<B: BitRead>(
    window: &mut [u8; WINDOW_SIZE],
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
        let bitcode = reader.peek_bits()?;
        let (symbol, len) = ll_decoder.decode(bitcode)?;
        reader.consume(len);
        match symbol.cmp(&END_OF_BLOCK) {
            Less => {
                window[idx & WINDOW_MASK] = symbol as u8;
                idx += 1;
            }
            Equal => {
                return Ok(DecodeResult::Done(idx - boundary));
            }
            Greater => {
                let (bits, mut length) = SYMBOL2BITS_LENGTH[(symbol & 0x1F) as usize];
                length += reader.read_bits(bits)?;
                let bitcode = reader.peek_bits()?;
                let (symbol, len) = dist_decoder.decode(bitcode)?;
                reader.consume(len);
                let (bits, mut distance) = SYMBOL2BITS_DISTANCE[(symbol & 0x1F) as usize];
                distance += reader.read_bits(bits)?;

                let distance = distance as usize;
                let length = length as usize;

                if distance > idx {
                    return Err(Error::DistanceTooMuch);
                }

                let begin = idx - distance;
                if distance >= length {
                    let (src, dst) = window.split_at_mut(idx);
                    dst[..length].copy_from_slice(&src[begin..begin + length]);
                    idx += length;
                } else if distance == 1 {
                    let x = window[begin];
                    window[idx..idx + length].fill(x);
                    idx += length;
                } else {
                    for (dst, src) in (idx..).zip(begin..begin + length) {
                        window[dst & WINDOW_MASK] = window[src & WINDOW_MASK];
                    }
                    idx += length;
                }
            }
        }

        if idx + MAX_LENGTH as usize >= window.len() {
            return Ok(DecodeResult::WindowIsFull(idx - boundary));
        }
    }
}

pub const SYMBOL2BITS_LENGTH: [(u32, u32); 32] = [
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
    (0, 0), // padding
    (0, 0),
];

pub const SYMBOL2BITS_DISTANCE: [(u32, u32); 32] = [
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
    (0, 0), // padding
    (0, 0),
];
