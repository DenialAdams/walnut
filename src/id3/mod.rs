use byteorder::{BigEndian, ByteOrder, ReadBytesExt};
use log::warn;
use std;
use std::io::{self, Read, Seek};

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

pub struct Parser {
   inner: Box<dyn Iterator<Item = Result<v24::Frame, v24::FrameParseError>>>,
}

impl Iterator for Parser {
   type Item = Result<v24::Frame, v24::FrameParseError>;

   fn next(&mut self) -> Option<Result<v24::Frame, v24::FrameParseError>> {
      self.inner.next()
   }
}

pub fn parse_source<S: Read + Seek>(source: &mut S) -> Result<Parser, TagParseError> {
   let mut header: &mut [u8] = &mut [0u8; 10];
   source.read_exact(&mut header)?;

   // TODO: search for ID3 from top of file
   let header = if &header[0..3] == b"ID3" {
      parse_header(&header[3..])
   } else {
      // TODO: search for 3DI from bottom of file
      Err(TagParseError::NoTag)
   }?;

   let mut size_of_frames = header.size;

   match header.flags {
      TagFlags::V24(flags) => {
         if header.revision > 0 {
            warn!(
               "Unknown revision ({}); proceeding anyway but may miss data",
               header.revision
            );
         }

         if flags.contains(v24::TagFlags::UNSYNCHRONIZED) {
            unimplemented!();
         }

         if flags.contains(v24::TagFlags::EXTENDED_HEADER) {
            let eh_size = synchsafe_u32_to_u32(source.read_u32::<BigEndian>()?);
            let mut eh_bytes = vec![0u8; eh_size as usize].into_boxed_slice();
            source.read_exact(&mut eh_bytes)?;
            debug_assert_eq!(eh_bytes[0], 1); // Number of flag bytes
            // eh_bytes[0] is always (supposed to be) set to 1
            let eh_flags = v24::ExtendedHeaderFlags::from_bits_truncate(eh_bytes[1]);

            size_of_frames -= 6;

            // v24::ExtendedHeaderFlags::TAG_IS_UPDATE

            if eh_flags.contains(v24::ExtendedHeaderFlags::CRC_DATA_PRESENT) {
               let mut crc_bytes = [0; 5];
               source.read_exact(&mut crc_bytes)?;
               // TODO: do something with this? and other EH FLAGS?
               // note to future self: haven't dealt with endianness of crc_bytes yet
               size_of_frames -= 5;
            }

            if eh_flags.contains(v24::ExtendedHeaderFlags::TAG_RESTRICTIONS) {
               // not really sure why we care, is there any point in obeying these restrictions?
               let _restrictions = source.read_u8();
               size_of_frames -= 1;
            }
         }

         if flags.contains(v24::TagFlags::EXPERIMENTAL_INDICATOR) {
            warn!("Tag is marked as experimental; proceeding anyway but may miss data");
         }

         if flags.contains(v24::TagFlags::FOOTER_PRESENT) {
            unimplemented!();
         }

         let mut frames = vec![0u8; size_of_frames as usize].into_boxed_slice();
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

fn synchsafe_u40_to_u32(sync_int: u64) -> u32 {
   let low = (sync_int & 0x00_00_00_ff) | (sync_int & 0x00_00_01_00) >> 1;
   let mid_low = (sync_int & 0x00_00_fe_00) >> 1 | (sync_int & 0x00_03_00_00) >> 2;
   let mid_high = (sync_int & 0x00_fc_00_00) >> 2 | (sync_int & 0x07_00_00_00) >> 3;
   let high = (sync_int & 0xf8_00_00_00) >> 3 | (sync_int & 0x0f_00_00_00_0) >> 4;
   let highest = (sync_int & 0xf0_00_00_00_00) >> 4;
   (highest | high | mid_high | mid_low | low) as u32
}

mod test {
   #[cfg(test)]
   use super::*;

   #[test]
   fn synchsafe_conversions() {
      assert_eq!(synchsafe_u32_to_u32(0x7f_7f_7f_7f), 0x0f_ff_ff_ff);
      assert_eq!(synchsafe_u40_to_u32(0x7f_7f_7f_7f_7f), 0xff_ff_ff_ff);
   }
}
