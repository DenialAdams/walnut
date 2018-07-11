use byteorder::{BigEndian, ReadBytesExt};
use std::io::{self, Cursor, Read, Seek};

macro_rules! wtry {
   ( $x:expr ) => {
      {
         match $x {
            Err(e) => return Some(Err(e)),
            Ok(v) => v
         }
      }
   };
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
   cursor: Cursor<Box<[u8]>>
}

#[derive(Debug)]
pub enum Frame {
   Unknown(UnknownFrame),
}

#[derive(Debug)]
pub struct UnknownFrame {
   pub name: [u8; 4],
   pub data: Box<[u8]>,
}

impl Iterator for Parser {
   type Item = Result<Frame, io::Error>;

   fn next(&mut self) -> Option<Result<Frame, io::Error>> {
      let mut name: [u8; 4] = [0; 4];
      wtry!(self.cursor.read(&mut name));
      if &name == b"\0\0\0\0" {
         // Padding or end of buffer
         return None;
      }

      let frame_size = synchsafe_u32_to_u32(wtry!(self.cursor.read_u32::<BigEndian>()));
      let frame_flags_raw = wtry!(self.cursor.read_u16::<BigEndian>());
      let frame_flags = FrameFlags::from_bits_truncate(frame_flags_raw);

      if !frame_flags.is_empty() {
         unimplemented!();
      }

      match &name {
         _ => {
            warn!("Unknown frame: {}", String::from_utf8_lossy(&name));
            let mut bytes = vec![0; frame_size as usize].into_boxed_slice();
            wtry!(self.cursor.read_exact(&mut bytes));
            Some(Ok(Frame::Unknown(UnknownFrame{
               name,
               data: bytes,
            })))
         }
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
            warn!("Unknown revision {}, proceeding anyway but may miss data", header.revision);
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
      cursor: Cursor::new(frames),
   })
}

struct Header {
   flags: TagFlags,
   revision: u8,
   size: u32,
}

fn parse_header(header: &[u8]) -> Result<Header, TagParseError> {
   let mut cursor = Cursor::new(header);
   let major_version = cursor.read_u8()?;
   let revision = cursor.read_u8()?;
   let raw_flags = cursor.read_u8()?;
   let flags = match major_version {
      2 => TagFlags::V22(v22::TagFlags::from_bits_truncate(raw_flags)),
      3 => TagFlags::V23(v23::TagFlags::from_bits_truncate(raw_flags)),
      4 => TagFlags::V24(v24::TagFlags::from_bits_truncate(raw_flags)),
      _ => return Err(TagParseError::UnsupportedVersion(major_version)),
   };

   Ok(Header {
      flags,
      revision,
      size: synchsafe_u32_to_u32(cursor.read_u32::<BigEndian>()?),
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