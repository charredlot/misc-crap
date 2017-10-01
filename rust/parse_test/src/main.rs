#![no_std]

extern crate libc;

use libc::{printf, c_uint};

macro_rules! cprintf {
       ($cfmt:expr) => (
           unsafe { printf(concat!($cfmt, "\0").as_ptr() as *const i8); };
        );
       ($cfmt:expr, $($args:tt)*) => (
            unsafe {
                printf(concat!($cfmt, "\0").as_ptr() as *const i8,
                       $($args)*);
            };
        );
}

macro_rules! cstr {
    ( $s:expr ) => (concat!($s, "\0").as_ptr() as *const i8);
}

#[derive(Debug)]
pub enum ParserResult {
    Done,
    NeedsMore,
    Detach,
    EOF,
}

pub struct Parser<'a> {
    bytes: &'a [u8],
    bytes_iter: usize,
    extract_len: usize,
    extracted: usize,
    uint_buf: [u8; 8],
}

fn convert_uint8(buf: &[u8; 8]) -> u8 {
    buf[0]
}

fn convert_uint16_be(buf: &[u8; 8]) -> u16 {
    ((buf[0] as u16) << 8) + (buf[1] as u16)
}

fn convert_uint16_le(buf: &[u8; 8]) -> u16 {
    ((buf[1] as u16) << 8) + (buf[0] as u16)
}

impl<'a> Parser<'a> {
    fn extract_to_uint(&mut self) -> ParserResult {
        loop {
            if self.bytes_iter >= self.bytes.len() {
                return ParserResult::NeedsMore;
            }
            let b = self.bytes[self.bytes_iter];
            self.bytes_iter += 1;

            self.uint_buf[self.extracted] = b;
            self.extracted += 1;
            if self.extracted == self.extract_len {
                self.extracted = 0;
                return ParserResult::Done;
            }
        }
    }

    fn extract_uint<T>(&mut self,
                       convert_uint: fn (buf: &[u8; 8]) -> T,
                       len: usize) -> Result<T, ParserResult> {
        if self.extracted == 0 {
            self.extract_len = len;
        }
        match self.extract_to_uint() {
            ParserResult::Done => Ok(convert_uint(&self.uint_buf)),
            other @ _ => Err(other),
        }

    }

    pub fn extract_uint8(&mut self) -> Result<u8, ParserResult> {
        self.extract_uint::<u8>(convert_uint8, 1)
    }

    pub fn extract_uint16_le(&mut self) -> Result<u16, ParserResult> {
        self.extract_uint::<u16>(convert_uint16_le, 2)
    }

    pub fn extract_uint16_be(&mut self) -> Result<u16, ParserResult> {
        self.extract_uint::<u16>(convert_uint16_be, 2)
    }
}

fn main() {
    cprintf!("boop boop hello world\n");
    let a = [0u8, 1u8, 2u8, 3u8,
             4u8, 5u8, 6u8, 7u8,
             8u8, 9u8];
    {
        let mut p = Parser{bytes: &a,
                           bytes_iter: 0,
                           extracted: 0,
                           extract_len: 0,
                           uint_buf: [0u8; 8]};

        loop {
            let r = p.extract_uint8();
            match r {
                Ok(b) => cprintf!("got uint 0x%04x\n", b as c_uint),
                Err(e) => {
                    cprintf!("err %u (should == %u)\n", e,
                             ParserResult::NeedsMore);
                    break;
                },
            };
        }
        cprintf!("done\n");
    }
}
