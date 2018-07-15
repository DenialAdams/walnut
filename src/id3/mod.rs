use byteorder::{BigEndian, ByteOrder};
use std::io::{self, Read, Seek};
use std::str::Utf8Error;
use std::string::FromUtf16Error;

pub mod v24;
mod v23;
mod v22;

enum TagFlags {
   V24(v24::TagFlags),
   V23(v23::TagFlags),
   V22(v22::TagFlags),
}

#[derive(Debug)]
pub enum TagParseError {
   NoTag,
   UnsupportedVersion(u8),
   Io(io::Error),
}

impl From<io::Error> for TagParseError {
   fn from(e: io::Error) -> TagParseError {
      TagParseError::Io(e)
   }
}

// TODO STREAMING ITERATOR
// IT WILL PREVENT ALLOCATIONS AND MAKE DREAMS REAL
// https://github.com/rust-lang/rust/issues/44265

#[derive(Debug)]
pub enum TextDecodeError {
   InvalidUtf16,
   InvalidUtf8,
   UnknownEncoding(u8),
}

impl From<FromUtf16Error> for TextDecodeError {
   fn from(_: FromUtf16Error) -> TextDecodeError {
      TextDecodeError::InvalidUtf16
   }
}

impl From<Utf8Error> for TextDecodeError {
   fn from(_: Utf8Error) -> TextDecodeError {
      TextDecodeError::InvalidUtf8
   }
}

pub struct Parser {
   inner: Box<dyn Iterator<Item=Result<v24::Frame, TextDecodeError>>>,
}

impl Iterator for Parser {
   type Item = Result<v24::Frame, TextDecodeError>;

   fn next(&mut self) -> Option<Result<v24::Frame, TextDecodeError>> {
      self.inner.next()
   }
}

pub fn parse_source<S: Read + Seek>(source: &mut S) -> Result<Parser, TagParseError> {
   let mut header: &mut [u8] = &mut [0u8; 10];
   source.read_exact(&mut header)?;

   let header = if &header[0..3] == b"ID3" {
      parse_header(&header[3..])
   } else {
      // Seek to bottom of file minus 10 bytes
      // check for "3DI"
      // if yes tag at bottom
      // unimplemented!()
      Err(TagParseError::NoTag)
   }?;

   match header.flags {
      TagFlags::V24(flags) => {
         if header.revision > 0 {
            warn!(
               "Unknown revision {}, proceeding anyway but may miss data",
               header.revision
            );
         }

         if flags.contains(v24::TagFlags::UNSYNCHRONIZED) {
            unimplemented!();
         }

         if flags.contains(v24::TagFlags::EXTENDED_HEADER) {
            unimplemented!();
         }

         if flags.contains(v24::TagFlags::EXPERIMENTAL_INDICATOR) {
            unimplemented!();
         }

         if flags.contains(v24::TagFlags::FOOTER_PRESENT) {
            unimplemented!();
         }

         let mut frames = vec![0u8; header.size as usize].into_boxed_slice();
         source.read_exact(&mut frames)?;

         Ok(Parser {
            inner: Box::new(v24::Parser::new(frames)),
         })
      }
      TagFlags::V23(_flags) => {
         Err(TagParseError::UnsupportedVersion(3))
      }
      TagFlags::V22(_flags) => {
         Err(TagParseError::UnsupportedVersion(2))
      }
   }
}

struct Header {
   flags: TagFlags,
   revision: u8,
   size: u32,
}

fn parse_header(header: &[u8]) -> Result<Header, TagParseError> {
   let major_version = header[0];
   let revision = header[1];
   let raw_flags = header[2];
   let flags = match major_version {
      2 => TagFlags::V22(v22::TagFlags::from_bits_truncate(raw_flags)),
      3 => TagFlags::V23(v23::TagFlags::from_bits_truncate(raw_flags)),
      4 => TagFlags::V24(v24::TagFlags::from_bits_truncate(raw_flags)),
      _ => return Err(TagParseError::UnsupportedVersion(major_version)),
   };

   Ok(Header {
      flags,
      revision,
      size: synchsafe_u32_to_u32(BigEndian::read_u32(&header[3..7])),
   })
}

fn synchsafe_u32_to_u32(sync_int: u32) -> u32 {
   let low = (sync_int & 0x00_00_00_ff) | (sync_int & 0x00_00_01_00) >> 1;
   let mid_low = (sync_int & 0x00_00_fe_00) >> 1 | (sync_int & 0x00_03_00_00) >> 2;
   let mid_high = (sync_int & 0x00_fc_00_00) >> 2 | (sync_int & 0x07_00_00_00) >> 3;
   let high = (sync_int & 0xf8_00_00_00) >> 3;
   high | mid_high | mid_low | low
}

mod test {
   #[cfg(test)]
   use super::*;

   #[test]
   fn synchsafe_conversions() {
      assert_eq!(synchsafe_u32_to_u32(0x7f_7f_7f_7f), 0x0f_ff_ff_ff);
   }
}
