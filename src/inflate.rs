use crate::bitread::BitRead;
use crate::error::Result;
use crate::error::Error;
use crate::sliding_window::SlidingWindow;
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
        todo!()
    }

    fn inflate_block2(&mut self) -> Result<()> {
        todo!()
    }
}