use byteorder::{BigEndian, ByteOrder};
use std;
use std::borrow::Cow;
use std::io::{self, Read, Seek};
use std::str::Utf8Error;
use std::string::FromUtf16Error;

macro_rules! wtry {
   ($x:expr) => {{
      match $x {
         Err(e) => return Some(Err(e)),
         Ok(v) => v,
      }
   }};
}

enum TagFlags {
   V24(v24::TagFlags),
   V23(v23::TagFlags),
   V22(v22::TagFlags),
}

mod v24 {
   bitflags! {
      pub(super) struct TagFlags: u8 {
         const UNSYNCHRONIZED = 0b1000_0000;
         const EXTENDED_HEADER = 0b0100_0000;
         const EXPERIMENTAL_INDICATOR = 0b0010_0000;
         const FOOTER_PRESENT = 0b0001_0000;
      }
   }
}

mod v23 {
   bitflags! {
      pub(super) struct TagFlags: u8 {
         const UNSYNCHRONIZED = 0b1000_0000;
         const EXTENDED_HEADER = 0b0100_0000;
         const EXPERIMENTAL_INDICATOR = 0b0010_0000;
      }
   }
}

mod v22 {
   bitflags! {
      pub(super) struct TagFlags: u8 {
         const UNSYNCHRONIZED = 0b1000_0000;
         const COMPRESSED = 0b0100_0000;
      }
   }
}

bitflags! {
   struct FrameFlags: u16 {
      // Status
      const TAG_ALTER_PRESERVATION = 0b0100_0000_0000_0000;
      const FILE_ALTER_PRESERVATION = 0b0010_0000_0000_0000;
      const READ_ONLY = 0b0001_0000_0000_0000;

      // Format
      const GROUPING_IDENTITY = 0b0000_0000_0100_0000;
      const COMPRESSION = 0b0000_0000_0000_1000;
      const ENCRYPTION = 0b0000_0000_0000_0100;
      const UNSYNCHRONIZATION = 0b0000_0000_0000_0010;
      const DATA_LENGTH_INDICATOR = 0b0000_0000_0000_0001;
   }
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

pub struct Parser {
   content: Box<[u8]>,
   cursor: usize,
}

#[derive(Debug)]
pub enum Frame {
   TPE1(String),
   Unknown(UnknownFrame),
}

#[derive(Debug)]
pub struct UnknownFrame {
   pub name: [u8; 4],
   pub data: Box<[u8]>,
}

impl Iterator for Parser {
   type Item = Result<Frame, TextDecodeError>;

   fn next(&mut self) -> Option<Result<Frame, TextDecodeError>> {
      if self.content.len() - self.cursor < 10 {
         return None;
      }

      let name = &self.content[self.cursor..self.cursor + 4];
      if name == b"\0\0\0\0" {
         // Padding or end of buffer
         return None;
      }

      let frame_size = synchsafe_u32_to_u32(BigEndian::read_u32(&self.content[self.cursor + 4..self.cursor + 8]));
      let frame_flags_raw = BigEndian::read_u16(&self.content[self.cursor + 8..self.cursor + 10]);
      let frame_flags = FrameFlags::from_bits_truncate(frame_flags_raw);

      if !frame_flags.is_empty() {
         unimplemented!();
      }

      self.cursor += 10;

      let result = match name {
         b"TPE1" => Some(Ok(Frame::TPE1(wtry!(self.decode_text_frame(frame_size)).into_owned()))),
         _ => {
            let mut owned_name = [0; 4];
            owned_name.copy_from_slice(name);
            let mut bytes = vec![0; frame_size as usize].into_boxed_slice();
            bytes.copy_from_slice(&self.content[self.cursor..self.cursor + frame_size as usize]);
            Some(Ok(Frame::Unknown(UnknownFrame {
               name: owned_name,
               data: bytes,
            })))
         }
      };

      self.cursor += frame_size as usize;

      return result;
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

impl Parser {
   fn decode_text_frame(&mut self, frame_size: u32) -> Result<Cow<str>, TextDecodeError> {
      let encoding = self.content[self.cursor];
      match encoding {
         0 => Ok(self.content[self.cursor + 1..self.cursor + frame_size as usize]
            .iter()
            .map(|c| *c as char)
            .collect()), // IS0 5859,
         1 => {
            let text_data = &self.content[self.cursor + 1..self.cursor + frame_size as usize];
            if text_data.len() % 2 != 0 {
               assert!(false);
            }
            let text_data =
               unsafe { std::slice::from_raw_parts(text_data.as_ptr() as *const u16, text_data.len() / 2) };
            Ok(Cow::from(String::from_utf16(text_data)?))
         } // UTF 16 with BOM
         2 => unimplemented!(), // UTF 16 BE NO BOM
         3 => Ok(Cow::from(std::str::from_utf8(
            &self.content[self.cursor + 1..self.cursor + frame_size as usize],
         )?)), // UTF 8
         _ => Err(TextDecodeError::UnknownEncoding(encoding)),
      }
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
      if true {
         unimplemented!();
      } else {
         return Err(TagParseError::NoTag);
      }
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
      }
      TagFlags::V23(_flags) => {
         return Err(TagParseError::UnsupportedVersion(3));
      }
      TagFlags::V22(_flags) => {
         return Err(TagParseError::UnsupportedVersion(2));
      }
   }

   let mut frames = vec![0u8; header.size as usize].into_boxed_slice();
   source.read_exact(&mut frames)?;

   Ok(Parser {
      cursor: 0,
      content: frames,
   })
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
