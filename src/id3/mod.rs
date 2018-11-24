use byteorder::{BigEndian, ByteOrder};
use std;
use std::borrow::Cow;
use std::io::{self, Read, Seek};
use std::num::ParseIntError;
use std::str::Utf8Error;
use std::string::FromUtf16Error;

mod v22;
mod v23;
pub mod v24;

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
pub enum FrameParseError {
   EmptyFrame,
   TextDecodeError(TextDecodeError),
   ParseTrackError(ParseTrackError),
}

impl From<TextDecodeError> for FrameParseError {
   fn from(e: TextDecodeError) -> FrameParseError {
      FrameParseError::TextDecodeError(e)
   }
}

impl From<ParseTrackError> for FrameParseError {
   fn from(e: ParseTrackError) -> FrameParseError {
      FrameParseError::ParseTrackError(e)
   }
}

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

#[derive(Debug)]
pub enum ParseTrackError {
   InvalidTrackNumber(ParseIntError),
}

impl From<ParseIntError> for ParseTrackError {
   fn from(e: ParseIntError) -> ParseTrackError {
      ParseTrackError::InvalidTrackNumber(e)
   }
}

pub struct Parser {
   inner: Box<dyn Iterator<Item = Result<v24::Frame, FrameParseError>>>,
}

impl Iterator for Parser {
   type Item = Result<v24::Frame, FrameParseError>;

   fn next(&mut self) -> Option<Result<v24::Frame, FrameParseError>> {
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
      TagFlags::V23(_flags) => Err(TagParseError::UnsupportedVersion(3)),
      TagFlags::V22(_flags) => Err(TagParseError::UnsupportedVersion(2)),
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

fn decode_text_frame(frame: &[u8]) -> Result<Cow<str>, TextDecodeError> {
   if frame.len() == 1 {
      // Only encoding is present
      return Ok(Cow::from(""));
   }

   let encoding = frame[0];

   // Adjust length to account for trailing nulls (if present)
   let text_end = if encoding == 0 || encoding == 3 {
      // 1 byte per char, 1 null at end
      if frame[frame.len() - 1] == 0 {
         frame.len() as usize - 1
      } else {
         frame.len() as usize
      }
   } else if encoding == 1 || encoding == 2 {
      // 2 bytes per char, 2 nulls at end
      if &frame[frame.len() - 2..frame.len()] == b"\0\0" {
         frame.len() - 2
      } else {
         frame.len()
      }
   } else {
      return Err(TextDecodeError::UnknownEncoding(encoding));
   };

   match encoding {
      0 => Ok(frame[1..text_end].iter().map(|c| *c as char).collect()), // IS0 5859,
      1 => {
         let text_data = &frame[1..text_end];
         if text_data[0..2] == [0xFE, 0xFF] {
            // Big endian
            unimplemented!();
         }
         if text_data.len() % 2 != 0 {
            return Err(TextDecodeError::InvalidUtf16);
         }
         let mut buffer = vec![0u16; text_data.len() / 2].into_boxed_slice();
         unsafe {
            std::ptr::copy_nonoverlapping::<u8>(text_data.as_ptr(), buffer.as_mut_ptr() as *mut u8, text_data.len())
         };
         Ok(Cow::from(String::from_utf16(&buffer[1..])?)) // 1.. to skip BOM
      } // UTF-16 with BOM
      2 => unimplemented!(),                                            // UTF-16 BE NO BOM
      3 => Ok(Cow::from(std::str::from_utf8(&frame[1..text_end])?)),    // UTF-8
      _ => unreachable!(),
   }
}

mod test {
   #[cfg(test)]
   use super::*;

   #[test]
   fn synchsafe_conversions() {
      assert_eq!(synchsafe_u32_to_u32(0x7f_7f_7f_7f), 0x0f_ff_ff_ff);
   }
}
