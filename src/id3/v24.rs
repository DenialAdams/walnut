use std;
use std::borrow::Cow;
use byteorder::{BigEndian, ByteOrder};
use super::{TextDecodeError, synchsafe_u32_to_u32};

bitflags! {
   pub(super) struct FrameFlags: u16 {
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

bitflags! {
   pub(super) struct TagFlags: u8 {
      const UNSYNCHRONIZED = 0b1000_0000;
      const EXTENDED_HEADER = 0b0100_0000;
      const EXPERIMENTAL_INDICATOR = 0b0010_0000;
      const FOOTER_PRESENT = 0b0001_0000;
   }
}

pub(super) struct Parser {
   content: Box<[u8]>,
   cursor: usize,
}

impl Parser {
   pub fn new(content: Box<[u8]>) -> Parser {
      Parser {
         content,
         cursor: 0,
      }
   }
}

#[derive(Debug)]
pub enum Frame {
   TALB(String),
   TCON(String),
   TIT2(String),
   TPE1(String),
   TPE2(String),
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

      let result = do catch {
         match name {
            b"TALB" => Frame::TALB(self.decode_text_frame(frame_size)?.into()),
            b"TCON" => {
               // TODO RX AND CR?
               let genre = self.decode_text_frame(frame_size)?;
               let genre = match genre.as_ref() {
                  "0" => "Blues",
                  "1" => "Classic Rock",
                  "2" => "Country",
                  "3" => "Dance",
                  "4" => "Disco",
                  "5" => "Funk",
                  "6" => "Grunge",
                  "7" => "Hip-Hop",
                  "8" => "Jazz",
                  "9" => "Metal",
                  "10" => "New Age",
                  "11" => "Oldies",
                  "12" => "Other",
                  "13" => "Pop",
                  "14" => "R&B",
                  "15" => "Rap",
                  "16" => "Reggae",
                  "17" => "Rock",
                  "18" => "Techno",
                  "19" => "Industrial",
                  "20" => "Alternative",
                  "21" => "Ska",
                  "22" => "Death Metal",
                  "23" => "Pranks",
                  "24" => "Soundtrack",
                  "25" => "Euro-Techno",
                  "26" => "Ambient",
                  "27" => "Trip-Hop",
                  "28" => "Vocal",
                  "29" => "Jazz+Funk",
                  "30" => "Fusion",
                  "31" => "Trance",
                  "32" => "Classical",
                  "33" => "Instrumental",
                  "34" => "Acid",
                  "35" => "House",
                  "36" => "Game",
                  "37" => "Sound Clip",
                  "38" => "Gospel",
                  "39" => "Noise",
                  "40" => "AlternRock",
                  "41" => "Bass",
                  "42" => "Soul",
                  "43" => "Punk",
                  "44" => "Space",
                  "45" => "Meditative",
                  "46" => "Instrumental Pop",
                  "47" => "Instrumental Rock",
                  "48" => "Ethnic",
                  "49" => "Gothic",
                  "50" => "Darkwave",
                  "51" => "Techno-Industrial",
                  "52" => "Electronic",
                  "53" => "Pop-Folk",
                  "54" => "Eurodance",
                  "55" => "Dream",
                  "56" => "Southern Rock",
                  "57" => "Comedy",
                  "58" => "Cult",
                  "59" => "Gangsta",
                  "60" => "Top 40",
                  "61" => "Christian Rap",
                  "62" => "Pop/Funk",
                  "63" => "Jungle",
                  "64" => "Native American",
                  "65" => "Cabaret",
                  "66" => "New Wave",
                  "67" => "Psychedelic",
                  "68" => "Rave",
                  "69" => "Showtunes",
                  "70" => "Trailer",
                  "71" => "Lo-Fi",
                  "72" => "Tribal",
                  "73" => "Acid Punk",
                  "74" => "Acid Jazz",
                  "75" => "Polka",
                  "76" => "Retro",
                  "77" => "Musical",
                  "78" => "Rock & Roll",
                  "79" => "Hard Rock",
                  _ => genre.as_ref(),
               };
               Frame::TCON(genre.into())
            }
            b"TIT2" => Frame::TIT2(self.decode_text_frame(frame_size)?.into()),
            b"TPE1" => Frame::TPE1(self.decode_text_frame(frame_size)?.into()),
            b"TPE2" => Frame::TPE2(self.decode_text_frame(frame_size)?.into()),
            _ => {
               let mut owned_name = [0; 4];
               owned_name.copy_from_slice(name);
               let mut bytes = vec![0; frame_size as usize].into_boxed_slice();
               bytes.copy_from_slice(&self.content[self.cursor..self.cursor + frame_size as usize]);
               Frame::Unknown(UnknownFrame {
                  name: owned_name,
                  data: bytes,
               })
            }
         }
      };

      self.cursor += frame_size as usize;

      Some(result)
   }
}

impl Parser {
   fn decode_text_frame(&mut self, frame_size: u32) -> Result<Cow<str>, TextDecodeError> {
      if frame_size == 1 {
         return Ok(Cow::from(""));
      }
      let encoding = self.content[self.cursor];
      let text_end = if encoding == 0 || encoding == 3 {
         if self.content[(self.cursor + frame_size as usize) - 1] == 0 {
            (self.cursor + frame_size as usize) - 1
         } else {
            self.cursor + frame_size as usize
         }
      } else if encoding == 1 || encoding == 2 {
         if &self.content[(self.cursor + frame_size as usize) - 2..self.cursor + frame_size as usize] == b"\0\0" {
            (self.cursor + frame_size as usize) - 2
         } else {
            self.cursor + frame_size as usize
         }
      } else {
         return Err(TextDecodeError::UnknownEncoding(encoding));
      };
      match encoding {
         0 => Ok(self.content[self.cursor + 1..text_end]
            .iter()
            .map(|c| *c as char)
            .collect()), // IS0 5859,
         1 => {
            let text_data = &self.content[self.cursor + 1..text_end];
            if text_data.len() % 2 != 0 {
               return Err(TextDecodeError::InvalidUtf16);
            }
            let mut buffer = vec![0u16; text_data.len() / 2].into_boxed_slice();
            unsafe { std::ptr::copy_nonoverlapping::<u8>(text_data.as_ptr(), buffer.as_mut_ptr() as *mut u8, text_data.len()) };
            Ok(Cow::from(String::from_utf16(&buffer)?))
         } // UTF 16 with BOM
         2 => unimplemented!(), // UTF 16 BE NO BOM
         3 => Ok(Cow::from(std::str::from_utf8(
            &self.content[self.cursor + 1..text_end],
         )?)), // UTF 8
         _ => unreachable!(),
      }
   }
}