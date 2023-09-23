// https://stackoverflow.com/questions/2602823/in-c-c-whats-the-simplest-way-to-reverse-the-order-of-bits-in-a-byte
fn reverse_bits(mut bits: u32) -> u32 {
    bits = (bits & 0xFF00) >> 8 | (bits & 0x00FF) << 8;
    bits = (bits & 0xF0F0) >> 4 | (bits & 0x0F0F) << 4;
    bits = (bits & 0xCCCC) >> 2 | (bits & 0x3333) << 2;
    bits = (bits & 0xAAAA) >> 1 | (bits & 0x5555) << 1;
    bits
}

use crate::error::{Error, Result};
use crate::codebook::CodeBook;

const NUM_BITS_FIRST_LOOKUP: u32 = 9;

pub struct HuffmanDecoder {
    /// lookup[bitcode] where bitcode is the bit-order reversed huffman code, right-aligned
    /// this is so that we can feed in multiple bits at once, from right to left
    lookup: Vec<(u32, u32)>, // symbol, length
    primary_mask: u32, // mask NUM_BITS_FIRST_LOOKUP bits
    secondary_mask: u32, // mask the rest of the bits
}

impl HuffmanDecoder {
    pub fn new(codebook: CodeBook) -> Self {
        let mut lookup = Vec::new();
        let max_nbits = codebook.max_length();
        let (nbits, secondary_mask) = if max_nbits > NUM_BITS_FIRST_LOOKUP {
            (
                NUM_BITS_FIRST_LOOKUP,
                (1 << (max_nbits - NUM_BITS_FIRST_LOOKUP)) - 1,
            )
        } else {
            // all codes can be decoded in a single step
            // secondary mask not needed
            (max_nbits, 0)
        };
        let primary_mask = (1 << nbits) - 1;

        lookup.resize(1 << nbits, (0, 0));
        for (symbol, (mut bitcode, len)) in codebook.into_iter().enumerate() {
            if len == 0 {
                continue;
            }

            // reverse bit-order of huffman
            bitcode = reverse_bits(bitcode);
            bitcode >>= 16 - len; // right-align
            if len <= nbits {
                // single-step lookup
                // populate table
                let delta = nbits - len;
                for idx in 0..(1 << delta) {
                    lookup[(bitcode | (idx << len)) as usize] = (symbol as u32, len);
                }
            } else {
                // two-step lookup
                // symbol will point to the base index for the second lookup step
                let base = (bitcode & primary_mask) as usize;
                let offset = if lookup[base].0 == 0 {
                    // first time the base has been seen
                    // will append the second-step lookup table
                    let offset = lookup.len() as u32;
                    // set up the index
                    lookup[base] = (offset, len);
                    // make room for the second-step lookup table
                    let new_len = lookup.len() + (1 << (max_nbits - nbits));
                    lookup.resize(new_len, (0, 0));
                    offset
                } else {
                    // second-step lookup already created
                    lookup[base].0
                };
                let secondary_len = len - nbits;
                let base = offset + ((bitcode >> nbits) & secondary_mask);
                // populate second-step lookup table
                for idx in 0..(1 << (max_nbits - len)) {
                    lookup[(base + (idx << secondary_len)) as usize] = (symbol as u32, len);
                }
            }
        }

        Self {
            lookup,
            primary_mask,
            secondary_mask
        }
    }

    /// Look up the code given at least max_length bits
    /// Returns symbol and its length upon match
    pub fn decode(&self, bits: u32) -> Result<(u32, u32)> {
        let (symbol, len) = self.lookup[(bits & self.primary_mask) as usize];
        if len == 0 {
            Err(Error::HuffmanDecoderCodeNotFound)
        } else if len <= NUM_BITS_FIRST_LOOKUP {
            Ok((symbol, len))
        } else {
            let base = symbol as usize;
            let idx = (bits >> NUM_BITS_FIRST_LOOKUP) & self.secondary_mask;
            Ok(self.lookup[base + idx as usize])
        }
    }
}