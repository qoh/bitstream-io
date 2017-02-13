use std::io;
use std::collections::VecDeque;

pub trait BitRead {
    /// Reads an unsigned value from the stream with
    /// the given number of bits.  This method assumes
    /// that the programmer is using an output value
    /// sufficiently large to hold those bits.
    fn read(&mut self, bits: u32) -> Result<u32, io::Error>;

    /// Reads a twos-complement signed value from the stream with
    /// the given number of bits.  This method assumes
    /// that the programmer is using an output value
    /// sufficiently large to hold those bits.
    fn read_signed(&mut self, bits: u32) -> Result<i32, io::Error>;

    /// Skips the given number of bits in the stream.
    /// Since this method does not need an accumulator,
    /// it may be slightly faster than reading to an empty variable.
    fn skip(&mut self, bits: u32) -> Result<(), io::Error>;

    /// Completely fills the given buffer with whole bytes.
    /// If the stream is already byte-aligned, it will map
    /// to a faster read_exact call.  Otherwise it will read
    /// bytes individually in 8-bit increments.
    fn read_bytes(&mut self, buf: &mut [u8]) -> Result<(), io::Error>;

    /// Reads an unsigned unary value with a stop bit of 0.
    fn read_unary0(&mut self) -> Result<u32, io::Error>;

    /// Reads an unsigned unary value with a stop bit of 1.
    fn read_unary1(&mut self) -> Result<u32, io::Error>;

    /// Returns true if the stream is aligned at a whole byte.
    fn byte_aligned(&self) -> bool;

    /// Throws away all unread bit values until the next whole byte.
    fn byte_align(&mut self);
}

pub struct BitReaderBE<'a> {
    reader: &'a mut io::Read,
    buffer: VecDeque<u32>
}

impl<'a> BitReaderBE<'a> {
    pub fn new(reader: &mut io::Read) -> BitReaderBE {
        BitReaderBE{reader: reader, buffer: VecDeque::with_capacity(8)}
    }

    fn next_bit(&mut self) -> Result<u32, io::Error> {
        if self.buffer.len() == 0 {
            let mut buf = [0; 1];
            self.reader.read_exact(&mut buf)?;
            self.buffer.push_back(((buf[0] >> 7) & 1) as u32);
            self.buffer.push_back(((buf[0] >> 6) & 1) as u32);
            self.buffer.push_back(((buf[0] >> 5) & 1) as u32);
            self.buffer.push_back(((buf[0] >> 4) & 1) as u32);
            self.buffer.push_back(((buf[0] >> 3) & 1) as u32);
            self.buffer.push_back(((buf[0] >> 2) & 1) as u32);
            self.buffer.push_back(((buf[0] >> 1) & 1) as u32);
            self.buffer.push_back(((buf[0] >> 0) & 1) as u32);
        }
        Ok(self.buffer.pop_front().unwrap())
    }
}

impl<'a> BitRead for BitReaderBE<'a> {
    fn read(&mut self, mut bits: u32) -> Result<u32, io::Error> {
        /*FIXME - make this generalized?*/
        /*FIXME - optimize this*/
        let mut acc = 0;
        while bits > 0 {
            acc = (acc << 1) | self.next_bit()?;
            bits -= 1;
        }
        Ok(acc)
    }

    fn read_signed(&mut self, bits: u32) -> Result<i32, io::Error> {
        /*FIXME - optimize this*/
        self.read(bits).map(|u| if (u & (1 << (bits - 1))) == 0 {
            u as i32
        } else {
            -((1 << bits) - (u as i32))
        })
    }

    fn skip(&mut self, bits: u32) -> Result<(), io::Error> {
        /*FIXME - optimize this*/
        self.read(bits).map(|_| ())
    }

    fn read_bytes(&mut self, buf: &mut [u8]) -> Result<(), io::Error> {
        if self.byte_aligned() {
            self.reader.read_exact(buf)
        } else {
            for b in buf.iter_mut() {
                *b = self.read(8)? as u8;
            }
            Ok(())
        }
    }

    fn read_unary0(&mut self) -> Result<u32, io::Error> {
        /*FIXME - optimize this*/
        let mut acc = 0;
        while self.read(1)? != 0 {
            acc += 1;
        }
        Ok(acc)
    }

    fn read_unary1(&mut self) -> Result<u32, io::Error> {
        /*FIXME - optimize this*/
        let mut acc = 0;
        while self.read(1)? != 1 {
            acc += 1;
        }
        Ok(acc)
    }

    fn byte_aligned(&self) -> bool {
        self.buffer.is_empty()
    }

    fn byte_align(&mut self) {
        self.buffer.clear()
    }
}