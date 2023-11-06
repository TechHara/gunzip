use crc32fast::Hasher;

pub trait Checksum {
    // update the checksum
    fn update(&mut self, xs: &[u8]);

    // returns the checksum and resets it
    fn checksum(&mut self) -> u32;

    // total number of bytes updated so far
    fn len(&self) -> usize;

    // reset the length counter
    fn reset_len(&mut self);
}

pub struct Crc32Checksum {
    hasher: Hasher,
    n: usize,
}

impl Crc32Checksum {
    pub fn new() -> Self {
        Self {
            hasher: Hasher::new(),
            n: 0,
        }
    }
}

impl Checksum for Crc32Checksum {
    fn update(&mut self, xs: &[u8]) {
        self.hasher.update(xs);
        self.n += xs.len();
    }

    fn checksum(&mut self) -> u32 {
        let hasher = std::mem::replace(&mut self.hasher, Hasher::new());
        hasher.finalize()
    }

    fn len(&self) -> usize {
        self.n
    }

    fn reset_len(&mut self) {
        self.n = 0;
    }
}
