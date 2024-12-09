// https://stackoverflow.com/questions/2602823/in-c-c-whats-the-simplest-way-to-reverse-the-order-of-bits-in-a-byte
fn reverse_bits(mut bits: u32) -> u32 {
    bits = (bits & 0xFF00) >> 8 | (bits & 0x00FF) << 8;
    bits = (bits & 0xF0F0) >> 4 | (bits & 0x0F0F) << 4;
    bits = (bits & 0xCCCC) >> 2 | (bits & 0x3333) << 2;
    bits = (bits & 0xAAAA) >> 1 | (bits & 0x5555) << 1;
    bits
}

use crate::codebook::CodeBook;
use crate::error::{Error, Result};

const NUM_BITS_FIRST_LOOKUP: u32 = 10;

/// | extra_bits: 4-bits | value: 15-bits | codelength: 4-bits | symbol: 9-bits |
#[derive(Clone, Copy, Default)]
pub struct PackedData(u32);

impl PackedData {
    pub fn new(symbol: u32, codelength: u32, value: u32, extra_bits: u32) -> Self {
        Self((extra_bits << 28) | (value << 13) | (codelength << 9) | symbol)
    }

    pub fn symbol(&self) -> u32 {
        self.0 & ((1 << 9) - 1)
    }

    pub fn codelength(&self) -> u32 {
        (self.0 >> 9) & ((1 << 4) - 1)
    }

    pub fn value(&self) -> u32 {
        (self.0 >> 13) & ((1 << 15) - 1)
    }

    pub fn extra_bits(&self) -> u32 {
        (self.0 >> 28) & ((1 << 4) - 1)
    }
}

pub struct HuffmanDecoder {
    /// lookup[bitcode] where bitcode is the bit-order reversed huffman code, right-aligned
    /// this is so that we can feed in multiple bits at once, from right to left
    lookup: Vec<PackedData>,
    primary_mask: u32,   // mask NUM_BITS_FIRST_LOOKUP bits
    secondary_mask: u32, // mask the rest of the bits
}

impl HuffmanDecoder {
    pub fn uninitialized() -> Self {
        Self {
            lookup: Vec::new(),
            primary_mask: 0,
            secondary_mask: 0,
        }
    }

    pub fn set(&mut self, codebook: CodeBook, extra: &[(u32, u32); 32]) {
        self.lookup.truncate(0);
        let max_nbits = codebook.max_length();
        let nbits: u32;
        let secondary_mask: u32;
        if max_nbits > NUM_BITS_FIRST_LOOKUP {
            nbits = NUM_BITS_FIRST_LOOKUP;
            secondary_mask = ((1 << (max_nbits - NUM_BITS_FIRST_LOOKUP)) - 1) as u32;
            let n_two_steps = codebook
                .tree
                .iter()
                .filter(|(_, l)| *l > NUM_BITS_FIRST_LOOKUP)
                .count();
            self.lookup.resize(
                (1 << nbits) + (n_two_steps << (max_nbits - nbits)),
                PackedData::default(),
            );
        } else {
            // all codes can be decoded in a single step
            // secondary mask not needed
            nbits = max_nbits;
            secondary_mask = 0;
            self.lookup.resize(1 << nbits, PackedData::default());
        };
        let primary_mask = (1 << nbits) as u32 - 1;
        let mut last_offset: usize = 1 << nbits;

        for (symbol, (mut bitcode, len)) in codebook.tree.into_iter().enumerate() {
            if len == 0 {
                continue;
            }

            // reverse bit-order of huffman
            bitcode = reverse_bits(bitcode);
            bitcode >>= 16 - len; // right-align

            let (extra, value) = extra[symbol & 0x1F];
            if len <= nbits {
                // single-step lookup
                // populate table
                let delta = nbits - len;

                let x = PackedData::new(symbol as u32, len, value, extra);
                for idx in 0..(1 << delta) {
                    self.lookup[(bitcode | (idx << len)) as usize] = x;
                }
            } else {
                // two-step lookup
                // symbol will point to the base index for the second lookup step
                let base = (bitcode & primary_mask) as usize;
                let offset = if self.lookup[base].0 == 0 {
                    // first time the base has been seen
                    // will append the second-step lookup table
                    let offset = last_offset;
                    last_offset += 1 << (max_nbits - nbits);
                    // set up the index
                    self.lookup[base] = PackedData::new(0, len, offset as u32, 0);
                    offset
                } else {
                    // second-step lookup already created
                    self.lookup[base].value() as usize
                };
                let secondary_len = len - nbits;
                let base = offset + ((bitcode >> nbits) & secondary_mask) as usize;
                // populate second-step lookup table
                let x = PackedData::new(symbol as u32, len, value, extra);
                for idx in 0..(1 << (max_nbits - len)) {
                    self.lookup[base + (idx << secondary_len)] = x;
                }
            }
        }

        self.lookup.truncate(last_offset);
        self.primary_mask = primary_mask;
        self.secondary_mask = secondary_mask;
    }

    /// Look up the code given at least max_length bits
    /// Returns symbol and its length upon match
    #[inline(always)]
    pub fn decode(&self, bits: u32) -> Result<PackedData> {
        let result = self.lookup[(bits & self.primary_mask) as usize];
        match result.codelength() {
            0 => Err(Error::HuffmanDecoderCodeNotFound),
            1..=NUM_BITS_FIRST_LOOKUP => Ok(result),
            _ => {
                let base = result.value() as usize;
                let idx = (bits >> NUM_BITS_FIRST_LOOKUP) & self.secondary_mask;
                Ok(self.lookup[base + idx as usize])
            }
        }
    }
}
