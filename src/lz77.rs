use crate::bitread::BitRead;
use crate::error::{Error, Result};
use crate::huffman_decoder::HuffmanDecoder;
use crate::sliding_window::{WINDOW_MASK, WINDOW_SIZE};
use std::cmp::Ordering::*;

pub const END_OF_BLOCK: u32 = 256;
pub const MAX_DISTANCE: u32 = 1 << 15; // 32kB
pub const MAX_LENGTH: u32 = 258;
const COPY_SIZE: usize = 32;

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
        let packed_data = ll_decoder.decode(bitcode)?;
        let symbol = packed_data.symbol();
        let len = packed_data.codelength();

        match symbol.cmp(&END_OF_BLOCK) {
            Less => {
                reader.consume(len);
                window[idx & WINDOW_MASK] = symbol as u8;
                idx += 1;
            }
            Equal => {
                reader.consume(len);
                return Ok(DecodeResult::Done(idx - boundary));
            }
            Greater => {
                let (bits, mut length) = (packed_data.extra_bits(), packed_data.value());
                length += (bitcode >> len) & ((1 << bits) - 1); // 15 + 5 <= 24
                reader.consume(len + bits);

                let bitcode = reader.peek_bits()?;
                let packed_data = dist_decoder.decode(bitcode)?;
                let len = packed_data.codelength();
                reader.consume(len); // 15 + 13 > 24
                let (bits, mut distance) = (packed_data.extra_bits(), packed_data.value());
                distance += reader.read_bits(bits)?;

                let distance = distance as usize;
                let length = length as usize;

                if distance > idx {
                    return Err(Error::DistanceTooMuch);
                }

                let begin = idx - distance;
                if distance >= length {
                    window.copy_within(begin..begin + COPY_SIZE, idx);
                    if length > COPY_SIZE {
                        let (src, dst) = window.split_at_mut(idx + COPY_SIZE);
                        dst[..length - COPY_SIZE].copy_from_slice(&src[begin + COPY_SIZE..begin + length]);
                    }
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
