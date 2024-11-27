use crate::error::{Error, Result};

pub const MAX_CODELENGTH: u32 = 15;
pub const MAX_LL_SYMBOL: u32 = 288;

pub struct CodeBook {
    pub tree: Vec<(u32, u32)>, // bitcode, length
    max_length: u32,
}

impl CodeBook {
    pub fn new(lengths: impl IntoIterator<Item = u32>, len: usize) -> Result<Self> {
        let err = Err(Error::InvalidCodeLengths);

        // follow RFC1951 https://www.rfc-editor.org/rfc/rfc1951#ref-1
        if len == 0 || len > MAX_LL_SYMBOL as usize + 1 {
            return err;
        }

        let mut tree = Vec::with_capacity(len);
        let mut max_len = 0; // max(lengths)

        // step 1
        // # of codes having bitcode length count
        let mut bl_count = [0; MAX_CODELENGTH as usize + 1];
        for l in lengths {
            bl_count[l as usize] += 1;
            tree.push((0, l));
            max_len = max_len.max(l);
        }

        if max_len > MAX_CODELENGTH {
            return err;
        }

        // step 2
        let mut next_code = [0; MAX_CODELENGTH as usize + 1];
        let mut code = 0;
        bl_count[0] = 0; // this is a must!!
        for bits in 1..=max_len as usize {
            code = (code + bl_count[bits - 1]) << 1;
            next_code[bits] = code;
        }

        // step 3
        for pair in &mut tree {
            let len = pair.1 as usize;
            if len != 0 {
                pair.0 = next_code[len];
                next_code[len] += 1;
            }
        }

        Ok(Self {
            tree,
            max_length: max_len,
        })
    }

    /// maximum number of bits within the codebook
    pub fn max_length(&self) -> u32 {
        self.max_length
    }

    pub fn default_ll() -> Self {
        let lengths = [
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 9,
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8,
        ];

        Self::new(lengths.iter().copied(), lengths.len()).unwrap()
    }

    pub fn default_dist() -> Self {
        let lengths = [5; 30];
        Self::new(lengths.iter().copied(), lengths.len()).unwrap()
    }
}
