use std::io::Read;
use std::mem::size_of;

pub trait BitRead: Read {
    /// peak at least 24-bits without consuming
    /// returns error if less than 24-bits are remaining
    /// the caller is expected to apply the mask
    fn peek_bits(&mut self) -> std::io::Result<u32>;

    /// consume n-bits
    fn consume(&mut self, n: u32);

    /// consume 0 to 7 bits to byte-align
    fn byte_align(&mut self);

    /// read up to 24-bits and consume
    /// return error if less than n-bits are remaining
    fn read_bits(&mut self, n: u32) -> std::io::Result<u32> {
        debug_assert!(n <= 24);
        let bits = self.peek_bits()?;
        self.consume(n);
        Ok(bits & ((1 << n) - 1))
    }

    /// indicate whether there is more data, even a single bit left
    /// it may refill the buffer
    fn has_data_left(&mut self) -> std::io::Result<bool>;
}

const BUFFER_SIZE: usize = 16 << 10;

pub struct BitReader<R: Read> {
    read: R,
    nbits: u32, // # consumed bits within the first byte
    buf: Vec<u8>,
    begin: usize,
    cap: usize,
}

impl<R: Read> BitReader<R> {
    pub fn new(read: R) -> Self {
        Self {
            read,
            nbits: 0,
            buf: vec![0; BUFFER_SIZE],
            begin: 0,
            cap: 0,
        }
    }

    fn buffer(&self) -> &[u8] {
        &self.buf[self.begin..self.cap]
    }

    /// # bits remaining within the buffer
    fn bit_len(&self) -> usize {
        self.buffer().len() * 8 - self.nbits as usize
    }

    /// refill buffer
    /// returns # additional bytes added to buffer
    fn fill_buf(&mut self) -> std::io::Result<usize> {
        self.buf.copy_within(self.begin..self.cap, 0);
        self.cap -= self.begin;
        self.begin = 0;
        let n = self.read.read(&mut self.buf[self.cap..])?;
        self.cap += n;
        Ok(n)
    }
}

impl<R: Read> Read for BitReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.byte_align();
        // read off from the buffer first
        let mut n = buf.len().min(self.buffer().len());
        buf[..n].copy_from_slice(&self.buffer()[..n]);
        self.begin += n;

        // read the rest directly from Read
        n += self.read.read(&mut buf[n..])?;
        Ok(n)
    }
}

impl<R: Read> BitRead for BitReader<R> {
    fn byte_align(&mut self) {
        if self.nbits > 0 {
            self.nbits = 0;
            self.begin += 1;
        }
    }

    fn consume(&mut self, n: u32) {
        debug_assert!(self.bit_len() >= n as usize);
        self.nbits += n;
        self.begin += (self.nbits / 8) as usize;
        self.nbits %= 8;
    }

    fn peek_bits(&mut self) -> std::io::Result<u32> {
        // fill_buf may not return enough bytes at once
        while self.buffer().len() < size_of::<u32>() {
            if self.fill_buf()? == 0 {
                return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
            }
        }
        let bits = u32::from_le_bytes(
            self.buf[self.begin..self.begin + size_of::<u32>()]
                .try_into()
                .unwrap(),
        );
        Ok(bits >> self.nbits)
    }

    fn has_data_left(&mut self) -> std::io::Result<bool> {
        Ok(!self.buffer().is_empty() || self.fill_buf()? != 0)
    }
}

pub trait ReadUntil {
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> std::io::Result<usize>;
}

impl<R: Read> ReadUntil for BitReader<R> {
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.byte_align();
        let mut n = 0;
        loop {
            match self.buffer().iter().position(|x| *x == byte) {
                Some(pos) => {
                    buf.extend_from_slice(&self.buffer()[..pos + 1]);
                    n += pos + 1;
                    self.begin += pos + 1;
                    return Ok(n);
                }
                None => {
                    buf.extend_from_slice(self.buffer());
                    n += self.buffer().len();
                    self.begin = self.cap;
                    if self.fill_buf()? == 0 {
                        return Ok(n);
                    }
                }
            }
        }
    }
}

impl<R: ReadUntil> ReadUntil for &mut R {
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        (**self).read_until(byte, buf)
    }
}

impl<R: BitRead> BitRead for &mut R {
    fn peek_bits(&mut self) -> std::io::Result<u32> {
        (**self).peek_bits()
    }

    fn consume(&mut self, n: u32) {
        (**self).consume(n)
    }

    fn byte_align(&mut self) {
        (**self).byte_align()
    }

    fn has_data_left(&mut self) -> std::io::Result<bool> {
        (**self).has_data_left()
    }
}