use super::{decode_text_frame, synchsafe_u32_to_u32, FrameParseError, FrameParseErrorReason, ParseDateError, ParseTrackError};
use byteorder::{BigEndian, ByteOrder};
use std::borrow::Cow;
use std::str::FromStr;
use std::num::ParseIntError;

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
      Parser { content, cursor: 0 }
   }
}

#[derive(Clone, Debug)]
pub enum Frame<'a> {
   TALB(Cow<'a, str>),
   TCOM(Cow<'a, str>),
   TCON(Cow<'a, str>),
   TDRC(Date),
   TENC(Cow<'a, str>),
   TIT2(Cow<'a, str>),
   TLEN(u64),
   TPE1(Cow<'a, str>),
   TPE2(Cow<'a, str>),
   TPE3(Cow<'a, str>),
   TPOS(Track),
   TPUB(Cow<'a, str>),
   TRCK(Track),
   TSOP(Cow<'a, str>),
   TSSE(Cow<'a, str>),
   Unknown(UnknownFrame<'a>),
}

impl<'a> Frame<'a> {
   /// Prefer `into_owned` if this `Frame` does not need to be used anymore
   pub fn to_owned(&self) -> OwnedFrame {
      match self {
         Frame::TALB(v) => OwnedFrame::TALB(v.to_string()),
         Frame::TCOM(v) => OwnedFrame::TCOM(v.to_string()),
         Frame::TCON(v) => OwnedFrame::TCON(v.to_string()),
         Frame::TDRC(v) => OwnedFrame::TDRC(v.clone()),
         Frame::TENC(v) => OwnedFrame::TENC(v.to_string()),
         Frame::TIT2(v) => OwnedFrame::TIT2(v.to_string()),
         Frame::TLEN(v) => OwnedFrame::TLEN(*v),
         Frame::TPE1(v) => OwnedFrame::TPE1(v.to_string()),
         Frame::TPE2(v) => OwnedFrame::TPE2(v.to_string()),
         Frame::TPE3(v) => OwnedFrame::TPE3(v.to_string()),
         Frame::TPOS(v) => OwnedFrame::TPOS(v.clone()),
         Frame::TPUB(v) => OwnedFrame::TPUB(v.to_string()),
         Frame::TRCK(v) => OwnedFrame::TRCK(v.clone()),
         Frame::TSOP(v) => OwnedFrame::TSOP(v.to_string()),
         Frame::TSSE(v) => OwnedFrame::TSSE(v.to_string()),
         Frame::Unknown(v) => OwnedFrame::Unknown(v.to_owned()),
      }
   }

   /// Should be preferred over `to_owned` if you no longer need the `Frame`
   pub fn into_owned(self) -> OwnedFrame {
      match self {
         Frame::TALB(v) => OwnedFrame::TALB(v.into_owned()),
         Frame::TCOM(v) => OwnedFrame::TCOM(v.into_owned()),
         Frame::TCON(v) => OwnedFrame::TCON(v.into_owned()),
         Frame::TDRC(v) => OwnedFrame::TDRC(v),
         Frame::TENC(v) => OwnedFrame::TENC(v.into_owned()),
         Frame::TIT2(v) => OwnedFrame::TIT2(v.into_owned()),
         Frame::TLEN(v) => OwnedFrame::TLEN(v),
         Frame::TPE1(v) => OwnedFrame::TPE1(v.into_owned()),
         Frame::TPE2(v) => OwnedFrame::TPE2(v.into_owned()),
         Frame::TPE3(v) => OwnedFrame::TPE3(v.into_owned()),
         Frame::TPOS(v) => OwnedFrame::TPOS(v),
         Frame::TPUB(v) => OwnedFrame::TPUB(v.into_owned()),
         Frame::TRCK(v) => OwnedFrame::TRCK(v),
         Frame::TSOP(v) => OwnedFrame::TSOP(v.into_owned()),
         Frame::TSSE(v) => OwnedFrame::TSSE(v.into_owned()),
         Frame::Unknown(v) => OwnedFrame::Unknown(v.to_owned()),
      }
   }
}

#[derive(Clone, Debug)]
pub enum OwnedFrame {
   TALB(String),
   TCOM(String),
   TCON(String),
   TDRC(Date),
   TENC(String),
   TIT2(String),
   TLEN(u64),
   TPE1(String),
   TPE2(String),
   TPE3(String),
   TPUB(String),
   TPOS(Track),
   TRCK(Track),
   TSOP(String),
   TSSE(String),
   Unknown(UnknownOwnedFrame),
}

#[derive(Clone, Debug)]
pub struct Date {
   pub year: u16,
   pub month: Option<u8>,
   pub day: Option<u8>,
   pub hour: Option<u8>,
   pub minutes: Option<u8>,
   pub seconds: Option<u8>,
}

// yyyy-MM-ddTHH:mm:ss
impl FromStr for Date {
   type Err = ParseDateError;

   fn from_str(s: &str) -> Result<Date, ParseDateError> {
      let year: u16 = {
         let result = s.get(0..4).map(|x| x.parse());
         match result {
            Some(v) => v?,
            None => return Err(ParseDateError::MissingYear), 
         }
      };

      let month: Option<u8> = {
         let result = s.get(5..7).map(|x| x.parse());
         match result {
            Some(v) => Some(v?),
            None => None,
         }
      };

      let day: Option<u8> = {
         let result = s.get(8..10).map(|x| x.parse());
         match result {
            Some(v) => Some(v?),
            None => None,
         }
      };

      let hour: Option<u8> = {
         let result = s.get(11..13).map(|x| x.parse());
         match result {
            Some(v) => Some(v?),
            None => None,
         }
      };

      let minutes: Option<u8> = {
         let result = s.get(14..16).map(|x| x.parse());
         match result {
            Some(v) => Some(v?),
            None => None,
         }
      };

      let seconds: Option<u8> = {
         let result = s.get(17..19).map(|x| x.parse());
         match result {
            Some(v) => Some(v?),
            None => None,
         }
      };


      Ok(Date {
         year,
         month,
         day,
         hour,
         minutes,
         seconds,
      })
   }
}

#[derive(Clone, Debug)]
pub struct Track {
   pub track_number: u64,
   pub track_max: Option<u64>,
}

impl FromStr for Track {
   type Err = ParseTrackError;

   fn from_str(s: &str) -> Result<Track, ParseTrackError> {
      let mut iter = s.splitn(2, '/');
      let track_number = iter.next().unwrap().parse()?;
      let track_max_result = iter.next().map(|x| x.parse());

      let track_max = match track_max_result {
         Some(v) => Some(v?),
         None => None,
      };

      Ok(Track {
         track_number,
         track_max,
      })
   }
}

#[derive(Clone, Debug)]
pub struct UnknownFrame<'a> {
   pub name: [u8; 4],
   pub data: &'a [u8],
}

impl<'a> UnknownFrame<'a> {
   fn to_owned(&self) -> UnknownOwnedFrame {
      let mut owned_bytes = vec![0; self.data.len()].into_boxed_slice();
      owned_bytes.copy_from_slice(self.data);

      UnknownOwnedFrame {
         name: self.name,
         data: owned_bytes,
      }
   }
}

#[derive(Clone, Debug)]
pub struct UnknownOwnedFrame {
   pub name: [u8; 4],
   pub data: Box<[u8]>,
}

impl Iterator for Parser {
   type Item = Result<OwnedFrame, FrameParseError>;

   fn next(&mut self) -> Option<Result<OwnedFrame, FrameParseError>> {
      if self.content.len() - self.cursor < 10 {
         return None;
      }

      let mut name: [u8; 4] = [0; 4];
      name.copy_from_slice(&self.content[self.cursor..self.cursor + 4]);
      if &name == b"\0\0\0\0" {
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

      if frame_size == 0 {
         return Some(Err(FrameParseError {
            name: name,
            reason: FrameParseErrorReason::EmptyFrame,
         }));
      }

      let frame_bytes = &self.content[self.cursor..self.cursor + frame_size as usize];

      let result: Result<Frame, FrameParseErrorReason> = try {
         match &name {
            b"TALB" => Frame::TALB(decode_text_frame(frame_bytes)?),
            b"TCOM" => Frame::TCOM(decode_text_frame(frame_bytes)?),
            b"TCON" => {
               let genre = decode_text_frame(frame_bytes)?;
               let genre = match genre.as_ref() {
                  "0" => Cow::Borrowed("Blues"),
                  "1" => Cow::Borrowed("Classic Rock"),
                  "2" => Cow::Borrowed("Country"),
                  "3" => Cow::Borrowed("Dance"),
                  "4" => Cow::Borrowed("Disco"),
                  "5" => Cow::Borrowed("Funk"),
                  "6" => Cow::Borrowed("Grunge"),
                  "7" => Cow::Borrowed("Hip-Hop"),
                  "8" => Cow::Borrowed("Jazz"),
                  "9" => Cow::Borrowed("Metal"),
                  "10" => Cow::Borrowed("New Age"),
                  "11" => Cow::Borrowed("Oldies"),
                  "12" => Cow::Borrowed("Other"),
                  "13" => Cow::Borrowed("Pop"),
                  "14" => Cow::Borrowed("R&B"),
                  "15" => Cow::Borrowed("Rap"),
                  "16" => Cow::Borrowed("Reggae"),
                  "17" => Cow::Borrowed("Rock"),
                  "18" => Cow::Borrowed("Techno"),
                  "19" => Cow::Borrowed("Industrial"),
                  "20" => Cow::Borrowed("Alternative"),
                  "21" => Cow::Borrowed("Ska"),
                  "22" => Cow::Borrowed("Death Metal"),
                  "23" => Cow::Borrowed("Pranks"),
                  "24" => Cow::Borrowed("Soundtrack"),
                  "25" => Cow::Borrowed("Euro-Techno"),
                  "26" => Cow::Borrowed("Ambient"),
                  "27" => Cow::Borrowed("Trip-Hop"),
                  "28" => Cow::Borrowed("Vocal"),
                  "29" => Cow::Borrowed("Jazz+Funk"),
                  "30" => Cow::Borrowed("Fusion"),
                  "31" => Cow::Borrowed("Trance"),
                  "32" => Cow::Borrowed("Classical"),
                  "33" => Cow::Borrowed("Instrumental"),
                  "34" => Cow::Borrowed("Acid"),
                  "35" => Cow::Borrowed("House"),
                  "36" => Cow::Borrowed("Game"),
                  "37" => Cow::Borrowed("Sound Clip"),
                  "38" => Cow::Borrowed("Gospel"),
                  "39" => Cow::Borrowed("Noise"),
                  "40" => Cow::Borrowed("AlternRock"),
                  "41" => Cow::Borrowed("Bass"),
                  "42" => Cow::Borrowed("Soul"),
                  "43" => Cow::Borrowed("Punk"),
                  "44" => Cow::Borrowed("Space"),
                  "45" => Cow::Borrowed("Meditative"),
                  "46" => Cow::Borrowed("Instrumental Pop"),
                  "47" => Cow::Borrowed("Instrumental Rock"),
                  "48" => Cow::Borrowed("Ethnic"),
                  "49" => Cow::Borrowed("Gothic"),
                  "50" => Cow::Borrowed("Darkwave"),
                  "51" => Cow::Borrowed("Techno-Industrial"),
                  "52" => Cow::Borrowed("Electronic"),
                  "53" => Cow::Borrowed("Pop-Folk"),
                  "54" => Cow::Borrowed("Eurodance"),
                  "55" => Cow::Borrowed("Dream"),
                  "56" => Cow::Borrowed("Southern Rock"),
                  "57" => Cow::Borrowed("Comedy"),
                  "58" => Cow::Borrowed("Cult"),
                  "59" => Cow::Borrowed("Gangsta"),
                  "60" => Cow::Borrowed("Top 40"),
                  "61" => Cow::Borrowed("Christian Rap"),
                  "62" => Cow::Borrowed("Pop/Funk"),
                  "63" => Cow::Borrowed("Jungle"),
                  "64" => Cow::Borrowed("Native American"),
                  "65" => Cow::Borrowed("Cabaret"),
                  "66" => Cow::Borrowed("New Wave"),
                  "67" => Cow::Borrowed("Psychedelic"),
                  "68" => Cow::Borrowed("Rave"),
                  "69" => Cow::Borrowed("Showtunes"),
                  "70" => Cow::Borrowed("Trailer"),
                  "71" => Cow::Borrowed("Lo-Fi"),
                  "72" => Cow::Borrowed("Tribal"),
                  "73" => Cow::Borrowed("Acid Punk"),
                  "74" => Cow::Borrowed("Acid Jazz"),
                  "75" => Cow::Borrowed("Polka"),
                  "76" => Cow::Borrowed("Retro"),
                  "77" => Cow::Borrowed("Musical"),
                  "78" => Cow::Borrowed("Rock & Roll"),
                  "79" => Cow::Borrowed("Hard Rock"),
                  "RX" => Cow::Borrowed("Remix"),
                  "CR" => Cow::Borrowed("Cover"),
                  _ => genre,
               };
               Frame::TCON(genre)
            }
            b"TDRC" => Frame::TDRC(decode_text_frame(frame_bytes)?.parse()?),
            b"TENC" => Frame::TENC(decode_text_frame(frame_bytes)?),
            b"TIT2" => Frame::TIT2(decode_text_frame(frame_bytes)?),
            b"TLEN" => Frame::TLEN(decode_text_frame(frame_bytes)?.parse()?),
            b"TPE1" => Frame::TPE1(decode_text_frame(frame_bytes)?),
            b"TPE2" => Frame::TPE2(decode_text_frame(frame_bytes)?),
            b"TPE3" => Frame::TPE3(decode_text_frame(frame_bytes)?),
            b"TPOS" => Frame::TPOS(decode_text_frame(frame_bytes)?.parse()?),
            b"TPUB" => Frame::TPUB(decode_text_frame(frame_bytes)?),
            b"TRCK" => Frame::TRCK(decode_text_frame(frame_bytes)?.parse()?),
            b"TSOP" => Frame::TSOP(decode_text_frame(frame_bytes)?),
            b"TSSE" => Frame::TSSE(decode_text_frame(frame_bytes)?),
            _ => Frame::Unknown(UnknownFrame {
               name,
               data: frame_bytes,
            }),
         }
      };

      self.cursor += frame_size as usize;

      Some(
         result
            .map(|v| v.into_owned())
            .map_err(|e| FrameParseError { name, reason: e }),
      )
   }
}
