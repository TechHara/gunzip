use crate::lz77::MAX_DISTANCE;

pub const WINDOW_SIZE: usize = MAX_DISTANCE as usize * 2;
pub const WINDOW_MASK: usize = WINDOW_SIZE - 1;

pub struct SlidingWindow {
    data: Box<[u8; WINDOW_SIZE]>,
    cur: usize, // where next data is to be written
}

#[allow(clippy::new_without_default)]
impl SlidingWindow {
    pub fn new() -> Self {
        Self {
            data: vec![0; WINDOW_SIZE].into_boxed_slice().try_into().unwrap(),
            cur: 0,
        }
    }

    /// entire window
    pub fn buffer(&mut self) -> &mut [u8; WINDOW_SIZE] {
        self.data.as_mut()
    }

    /// Write buffer size is guaranteed to be at least 32kB after `slide()`
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
