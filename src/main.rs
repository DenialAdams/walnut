#![feature(nll)]

extern crate byteorder;
#[macro_use]
extern crate log;
extern crate walkdir;
#[macro_use]
extern crate bitflags;
extern crate pretty_env_logger;

use byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;
use std::io::{self, Cursor, Read, Seek};
use walkdir::WalkDir;

enum Id3Flags {
   V24(Id324Flags),
   V23(Id323Flags),
   V22(Id322Flags),
}

bitflags! {
    struct Id324Flags: u8 {
        const UNSYNCHRONIZED = 0b1000_0000;
        const EXTENDED_HEADER = 0b0100_0000;
        const EXPERIMENTAL_INDICATOR = 0b0010_0000;
        const FOOTER_PRESENT = 0b0001_0000;
    }
}

bitflags! {
   struct Id323Flags: u8 {
       const UNSYNCHRONIZED = 0b1000_0000;
       const EXTENDED_HEADER = 0b0100_0000;
       const EXPERIMENTAL_INDICATOR = 0b0010_0000;
   }
}

bitflags! {
   struct Id322Flags: u8 {
      const UNSYNCHRONIZED = 0b1000_0000;
      const COMPRESSED = 0b0100_0000;
   }
}

bitflags! {
   struct Id3FrameFlags: u16 {
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

fn main() {
   pretty_env_logger::init();

   for entry in WalkDir::new("C:\\music").into_iter() {
      let entry = match entry {
         Err(err) => {
            warn!("Failed to open file/directory: {}", err);
            continue;
         }
         Ok(entry) => {
            if !entry.file_type().is_file() {
               continue;
            }

            if let Some(ref extension) = entry.file_name().to_string_lossy().split('.').last() {
               if *extension == "mp3" {
                  entry
               } else {
                  continue;
               }
            } else {
               continue;
            }
         }
      };

      println!("{}", entry.path().file_name().unwrap().to_string_lossy());

      let mut f = File::open(entry.path()).unwrap();
      match parse_id3(&mut f) {
         Ok(()) => {
            println!("ID3v24");
         }
         Err(e) => match e {
            Id3ParseError::NoId3Tag => {
               println!("No ID3");
            }
            Id3ParseError::UnsupportedVersion(ver) => {
               println!("ID3v2{}", ver);
            }
            Id3ParseError::Io(io_err) => {
               warn!("Failed to parse file: {}", io_err);
            }
         },
      }
   }
}

#[derive(Debug)]
enum Id3ParseError {
   NoId3Tag,
   UnsupportedVersion(u8),
   Io(io::Error),
}

impl From<io::Error> for Id3ParseError {
   fn from(e: io::Error) -> Id3ParseError {
      Id3ParseError::Io(e)
   }
}

fn parse_id3<S: Read + Seek>(source: &mut S) -> Result<(), Id3ParseError> {
   let mut header: &mut [u8] = &mut [0u8; 10];
   source.read_exact(&mut header)?;

   let header = if &header[0..3] == b"ID3" {
      parse_id3_header(&header[3..])
   } else {
      // Seek to bottom of file minus 10 bytes
      // check for "3DI"
      // if yes tag at bottom
      unimplemented!();
      return Err(Id3ParseError::NoId3Tag);
   }?;

   match header.major_version {
      Id3Flags::V24(flags) => {
         if flags.contains(Id324Flags::UNSYNCHRONIZED) {
            unimplemented!();
         }

         if flags.contains(Id324Flags::EXTENDED_HEADER) {
            unimplemented!();
         }

         if flags.contains(Id324Flags::EXPERIMENTAL_INDICATOR) {
            unimplemented!();
         }

         if flags.contains(Id324Flags::FOOTER_PRESENT) {
            unimplemented!();
         }
      }
      Id3Flags::V23(_flags) => {
         return Err(Id3ParseError::UnsupportedVersion(3));
      }
      Id3Flags::V22(_flags) => {
         return Err(Id3ParseError::UnsupportedVersion(2));
      }
   }

   let mut frames = vec![0u8; header.size as usize].into_boxed_slice();
   source.read_exact(&mut frames)?;

   let mut frames_cursor = Cursor::new(frames);
   while frames_cursor.position() as u32 != header.size {
      let mut name: [u8; 4] = [0; 4];
      frames_cursor.read_exact(&mut name)?;
      let frame_size = synchsafe_u32_to_u32(frames_cursor.read_u32::<BigEndian>()?);
      let frame_flags_raw = frames_cursor.read_u16::<BigEndian>()?;
      let frame_flags = Id3FrameFlags::from_bits_truncate(frame_flags_raw);
      match &name {
         b"\0\0\0\0" => {
            // Padding, safe to skip to end
            frames_cursor.set_position(frames_cursor.position() + frame_size as u64);
         }
         _ => {
            warn!("Unknown frame: {}", String::from_utf8_lossy(&name));
            frames_cursor.set_position(frames_cursor.position() + frame_size as u64);
         }
      }
   }

   Ok(())
}

struct Id3Header {
   major_version: Id3Flags,
   revision: u8,
   size: u32,
}

fn parse_id3_header(header: &[u8]) -> Result<Id3Header, Id3ParseError> {
   // TODO eh don't really neeed cursor here
   let mut cursor = Cursor::new(header);
   let major_version = cursor.read_u8()?;
   let revision = cursor.read_u8()?;
   let raw_flags = cursor.read_u8()?;
   let flags = match major_version {
      2 => Id3Flags::V22(Id322Flags::from_bits_truncate(raw_flags)),
      3 => Id3Flags::V23(Id323Flags::from_bits_truncate(raw_flags)),
      4 => Id3Flags::V24(Id324Flags::from_bits_truncate(raw_flags)),
      _ => return Err(Id3ParseError::UnsupportedVersion(major_version)),
   };
   Ok(Id3Header {
      major_version: flags,
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
