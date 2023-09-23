use crate::lz77::MAX_DISTANCE;

pub const WINDOW_SIZE: usize = MAX_DISTANCE as usize * 3;

pub struct SlidingWindow {
    data: Vec<u8>,
    cur: usize, // where next data is to be written
}

impl SlidingWindow {
    pub fn new() -> Self {
        Self {
            data: vec![0; WINDOW_SIZE],
            cur: 0,
        }
    }

    /// entire window
    pub fn buffer(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Write buffer size is guaranteed to be at least 64kB after `slide()`
    pub fn write_buffer(&mut self) -> &mut [u8] {
        &mut self.data[self.cur..]
    }

    /// Slide the data so that at most 32kB of the most recent history is kept
    pub fn slide(&mut self, n: usize) {
        let end = self.cur + n;
        if end > MAX_DISTANCE as usize {
            let delta = end - MAX_DISTANCE as usize;
            self.data.copy_within(delta..end, 0);
            self.cur = MAX_DISTANCE as usize;
        } else {
            self.cur = end;
        }
    }

    /// index to write buffer
    pub fn boundary(&self) -> usize {
        self.cur
    }
}