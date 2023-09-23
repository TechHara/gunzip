use crate::bitread::BitRead;
use crate::error::Result;
use crate::error::Error;
use crate::sliding_window::SlidingWindow;
use crate::codebook::CodeBook;
use crate::huffman_decoder::HuffmanDecoder;
use crate::lz77::{CodeIterator, DecodeResult, decode};
use std::io::Write;

pub struct Inflate<R: BitRead, W: Write> {
    reader: R,
    writer: W,
    window: SlidingWindow,
}

impl<R: BitRead, W: Write> Inflate<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self { reader, writer, window: SlidingWindow::new() }
    }

    pub fn run(mut self) -> Result<()> {
        loop {
            let header = self.reader.read_bits(3)?;
            let is_final = header & 1 == 1;
            match header & 0b110 {
                0b000 => self.inflate_block0()?,
                0b010 => self.inflate_block1()?,
                0b100 => self.inflate_block2()?,
                _ => return Err(Error::InvalidBlockType),
            }
            if is_final {
                break;
            }
        }
        Ok(())
    }

    fn inflate_block0(&mut self) -> Result<()> {
        self.reader.byte_align();
        let len = self.reader.read_bits(16)?;
        let nlen = self.reader.read_bits(16)?;
        if len ^ nlen != 0xFFFF {
            Err(Error::BlockType0LenMismatch)
        } else {
            let buf = &mut self.window.write_buffer()[..len as usize];
            self.reader.read_exact(buf)?;
            self.writer.write_all(&buf)?;
            self.window.slide(len as usize);
            Ok(())
        }
    }

    fn inflate_block1(&mut self) -> Result<()> {
        let ll_decoder = HuffmanDecoder::new(CodeBook::default_ll());
        let dist_decoder = HuffmanDecoder::new(CodeBook::default_dist());
        self.inflate(ll_decoder, dist_decoder)
    }

    fn inflate_block2(&mut self) -> Result<()> {
        let (ll_decoder, dist_decoder) = self.read_dynamic_codebooks()?;
        self.inflate(ll_decoder, dist_decoder)
    }

    fn inflate(&mut self, ll_decoder: HuffmanDecoder, dist_decoder: HuffmanDecoder) -> Result<()> {
        let mut iter = CodeIterator::new(&mut self.reader, ll_decoder, dist_decoder);

        let mut done = false;
        loop {
            let boundary = self.window.boundary();
            let n = match decode(self.window.buffer(), boundary, &mut iter)? {
                DecodeResult::Done(n) => {
                    done = true;
                    n
                }
                DecodeResult::WindowIsFull(n) => n,
                DecodeResult::Error(e) => {
                    return Err(e);
                }
            };
            self.writer.write_all(&self.window.write_buffer()[..n])?;
            self.window.slide(n);
            if done {
                break;
            }
        }
        Ok(())
    }

    fn read_dynamic_codebooks(&mut self) -> Result<(HuffmanDecoder, HuffmanDecoder)> {
        let hlit = self.reader.read_bits(5)? as usize + 257;
        let hdist = self.reader.read_bits(5)? as usize + 1;
        let hclen = self.reader.read_bits(4)? as usize + 4;
        let mut cl_lengths = [0 as u32; 19];
        for idx in [
            16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
        ]
        .into_iter()
        .take(hclen)
        {
            cl_lengths[idx] = self.reader.read_bits(3)? as u32;
        }
        let cl_codes = CodeBook::new(&cl_lengths)?;
        let cl_decoder = HuffmanDecoder::new(cl_codes);

        // The code lengths contain LL codes and Distance codes as a single table
        let num_codes = hlit + hdist;
        let mut lengths = Vec::with_capacity(num_codes);
        while lengths.len() < num_codes {
            let (cl_code, len) = cl_decoder
                .decode(self.reader.peek_bits()?)
                .or(Err(Error::ReadDynamicCodebook))?;
            self.reader.consume(len);
            match cl_code {
                0..=15 => {
                    lengths.push(cl_code as u32);
                }
                16 if !lengths.is_empty() => {
                    let length = 3 + self.reader.read_bits(2)? as usize;
                    let x = *lengths.last().unwrap();
                    lengths.resize(lengths.len() + length, x);
                }
                17 => {
                    let length = 3 + self.reader.read_bits(3)? as usize;
                    lengths.resize(lengths.len() + length, 0);
                }
                18 => {
                    let length = 11 + self.reader.read_bits(7)? as usize;
                    lengths.resize(lengths.len() + length, 0);
                }
                _ => {
                    unreachable!()
                }
            }
        }

        if lengths.len() != num_codes {
            return Err(Error::ReadDynamicCodebook);
        }

        let ll_codes = CodeBook::new(&lengths[..hlit])?;
        let dist_codes = CodeBook::new(&lengths[hlit..])?;
        Ok((
            HuffmanDecoder::new(ll_codes),
            HuffmanDecoder::new(dist_codes),
        ))
    }
}