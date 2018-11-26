use super::synchsafe_u32_to_u32;
use byteorder::{BigEndian, ByteOrder};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::num::ParseIntError;
use std::str::{FromStr, Utf8Error};
use std::string::FromUtf16Error;

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
   COMM(LangDescriptionText<'a>),
   PRIV(Priv<'a>),
   TALB(Cow<'a, str>),
   TBPM(u64),
   TCOM(Cow<'a, str>),
   TCON(Cow<'a, str>),
   TDEN(Date),
   TDLY(u64),
   TDOR(Date),
   TDRC(Date),
   TDRL(Date),
   TDTG(Date),
   TENC(Cow<'a, str>),
   TEXT(Cow<'a, str>),
   TIT1(Cow<'a, str>),
   TIT2(Cow<'a, str>),
   TIT3(Cow<'a, str>),
   TLEN(u64),
   TMOO(Cow<'a, str>),
   TOPE(Cow<'a, str>),
   TPE1(Cow<'a, str>),
   TPE2(Cow<'a, str>),
   TPE3(Cow<'a, str>),
   TPOS(Track),
   TPUB(Cow<'a, str>),
   TRCK(Track),
   TSOP(Cow<'a, str>),
   TSSE(Cow<'a, str>),
   TXXX(Txxx<'a>),
   USLT(LangDescriptionText<'a>),
   Unknown(UnknownFrame<'a>),
}

impl<'a> Frame<'a> {
   /// Prefer `into_owned` if this `Frame` does not need to be used anymore
   pub fn to_owned(&self) -> OwnedFrame {
      match self {
         Frame::COMM(v) => OwnedFrame::COMM(v.to_owned()),
         Frame::PRIV(v) => OwnedFrame::PRIV(v.to_owned()),
         Frame::TALB(v) => OwnedFrame::TALB(v.to_string()),
         Frame::TBPM(v) => OwnedFrame::TBPM(*v),
         Frame::TCOM(v) => OwnedFrame::TCOM(v.to_string()),
         Frame::TCON(v) => OwnedFrame::TCON(v.to_string()),
         Frame::TDEN(v) => OwnedFrame::TDEN(v.clone()),
         Frame::TDLY(v) => OwnedFrame::TDLY(*v),
         Frame::TDOR(v) => OwnedFrame::TDOR(v.clone()),
         Frame::TDRC(v) => OwnedFrame::TDRC(v.clone()),
         Frame::TDRL(v) => OwnedFrame::TDRL(v.clone()),
         Frame::TDTG(v) => OwnedFrame::TDTG(v.clone()),
         Frame::TENC(v) => OwnedFrame::TENC(v.to_string()),
         Frame::TEXT(v) => OwnedFrame::TEXT(v.to_string()),
         Frame::TIT1(v) => OwnedFrame::TIT1(v.to_string()),
         Frame::TIT2(v) => OwnedFrame::TIT2(v.to_string()),
         Frame::TIT3(v) => OwnedFrame::TIT3(v.to_string()),
         Frame::TLEN(v) => OwnedFrame::TLEN(*v),
         Frame::TMOO(v) => OwnedFrame::TMOO(v.to_string()),
         Frame::TOPE(v) => OwnedFrame::TOPE(v.to_string()),
         Frame::TPE1(v) => OwnedFrame::TPE1(v.to_string()),
         Frame::TPE2(v) => OwnedFrame::TPE2(v.to_string()),
         Frame::TPE3(v) => OwnedFrame::TPE3(v.to_string()),
         Frame::TPOS(v) => OwnedFrame::TPOS(v.clone()),
         Frame::TPUB(v) => OwnedFrame::TPUB(v.to_string()),
         Frame::TRCK(v) => OwnedFrame::TRCK(v.clone()),
         Frame::TSOP(v) => OwnedFrame::TSOP(v.to_string()),
         Frame::TSSE(v) => OwnedFrame::TSSE(v.to_string()),
         Frame::TXXX(v) => OwnedFrame::TXXX(v.to_owned()),
         Frame::USLT(v) => OwnedFrame::USLT(v.to_owned()),
         Frame::Unknown(v) => OwnedFrame::Unknown(v.to_owned()),
      }
   }

   /// Should be preferred over `to_owned` if you no longer need the `Frame`
   pub fn into_owned(self) -> OwnedFrame {
      match self {
         Frame::COMM(v) => OwnedFrame::COMM(v.into_owned()),
         Frame::PRIV(v) => OwnedFrame::PRIV(v.into_owned()),
         Frame::TALB(v) => OwnedFrame::TALB(v.into_owned()),
         Frame::TBPM(v) => OwnedFrame::TBPM(v),
         Frame::TCOM(v) => OwnedFrame::TCOM(v.into_owned()),
         Frame::TCON(v) => OwnedFrame::TCON(v.into_owned()),
         Frame::TDEN(v) => OwnedFrame::TDEN(v),
         Frame::TDLY(v) => OwnedFrame::TDLY(v),
         Frame::TDOR(v) => OwnedFrame::TDOR(v),
         Frame::TDRC(v) => OwnedFrame::TDRC(v),
         Frame::TDRL(v) => OwnedFrame::TDRL(v),
         Frame::TDTG(v) => OwnedFrame::TDTG(v),
         Frame::TENC(v) => OwnedFrame::TENC(v.into_owned()),
         Frame::TEXT(v) => OwnedFrame::TEXT(v.into_owned()),
         Frame::TIT1(v) => OwnedFrame::TIT1(v.into_owned()),
         Frame::TIT2(v) => OwnedFrame::TIT2(v.into_owned()),
         Frame::TIT3(v) => OwnedFrame::TIT3(v.into_owned()),
         Frame::TLEN(v) => OwnedFrame::TLEN(v),
         Frame::TMOO(v) => OwnedFrame::TMOO(v.into_owned()),
         Frame::TOPE(v) => OwnedFrame::TOPE(v.into_owned()),
         Frame::TPE1(v) => OwnedFrame::TPE1(v.into_owned()),
         Frame::TPE2(v) => OwnedFrame::TPE2(v.into_owned()),
         Frame::TPE3(v) => OwnedFrame::TPE3(v.into_owned()),
         Frame::TPOS(v) => OwnedFrame::TPOS(v),
         Frame::TPUB(v) => OwnedFrame::TPUB(v.into_owned()),
         Frame::TRCK(v) => OwnedFrame::TRCK(v),
         Frame::TSOP(v) => OwnedFrame::TSOP(v.into_owned()),
         Frame::TSSE(v) => OwnedFrame::TSSE(v.into_owned()),
         Frame::TXXX(v) => OwnedFrame::TXXX(v.into_owned()),
         Frame::USLT(v) => OwnedFrame::USLT(v.into_owned()),
         Frame::Unknown(v) => OwnedFrame::Unknown(v.to_owned()),
      }
   }
}

#[derive(Clone, Debug)]
pub enum OwnedFrame {
   COMM(OwnedLangDescriptionText),
   PRIV(OwnedPriv),
   TALB(String),
   TBPM(u64),
   TCOM(String),
   TCON(String),
   TDEN(Date),
   TDLY(u64),
   TDOR(Date),
   TDRC(Date),
   TDRL(Date),
   TDTG(Date),
   TENC(String),
   TEXT(String),
   TIT1(String),
   TIT2(String),
   TIT3(String),
   TLEN(u64),
   TMOO(String),
   TOPE(String),
   TPE1(String),
   TPE2(String),
   TPE3(String),
   TPUB(String),
   TPOS(Track),
   TRCK(Track),
   TSOP(String),
   TSSE(String),
   TXXX(OwnedTxxx),
   USLT(OwnedLangDescriptionText),
   Unknown(OwnedUnknownFrame),
}

#[derive(Clone, Debug)]
pub struct LangDescriptionText<'a> {
   pub iso_639_2_lang: [u8; 3],
   pub description: Cow<'a, str>,
   pub text: Cow<'a, str>,
}

impl<'a> LangDescriptionText<'a> {
   fn to_owned(&self) -> OwnedLangDescriptionText {
      OwnedLangDescriptionText {
         iso_639_2_lang: self.iso_639_2_lang,
         description: self.description.to_string(),
         text: self.text.to_string(),
      }
   }

   fn into_owned(self) -> OwnedLangDescriptionText {
      OwnedLangDescriptionText {
         iso_639_2_lang: self.iso_639_2_lang,
         description: self.description.into_owned(),
         text: self.text.into_owned(),
      }
   }
}

#[derive(Clone, Debug)]
pub struct OwnedLangDescriptionText {
   pub iso_639_2_lang: [u8; 3],
   pub description: String,
   pub text: String,
}

#[derive(Clone, Debug)]
pub struct Txxx<'a> {
   pub description: Cow<'a, str>,
   pub text: Cow<'a, str>,
}

impl<'a> Txxx<'a> {
   fn to_owned(&self) -> OwnedTxxx {
      OwnedTxxx {
         description: self.description.to_string(),
         text: self.text.to_string(),
      }
   }

   fn into_owned(self) -> OwnedTxxx {
      OwnedTxxx {
         description: self.description.into_owned(),
         text: self.text.into_owned(),
      }
   }
}

#[derive(Clone, Debug)]
pub struct OwnedTxxx {
   pub description: String,
   pub text: String,
}

#[derive(Clone, Debug)]
pub struct Priv<'a> {
   pub owner: String,
   pub data: &'a [u8],
}

impl<'a> Priv<'a> {
   fn to_owned(&self) -> OwnedPriv {
      let mut owned_bytes = vec![0; self.data.len()].into_boxed_slice();
      owned_bytes.copy_from_slice(self.data);

      OwnedPriv {
         owner: self.owner.clone(),
         data: owned_bytes,
      }
   }

   fn into_owned(self) -> OwnedPriv {
      let mut owned_bytes = vec![0; self.data.len()].into_boxed_slice();
      owned_bytes.copy_from_slice(self.data);

      OwnedPriv {
         owner: self.owner,
         data: owned_bytes,
      }
   }
}

#[derive(Clone, Debug)]
pub struct OwnedPriv {
   pub owner: String,
   pub data: Box<[u8]>,
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
      // @Performance:
      // I'm sure there are many opportunities to optimize here:
      // for one thing, we can stop trying to parse the date as soon
      // as we get one None. Micro-benchmarks need to be set up
      // and then we can re-visit.

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
   pub number: u64,
   pub max: Option<u64>,
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
         number: track_number,
         max: track_max,
      })
   }
}

#[derive(Clone, Debug)]
pub struct UnknownFrame<'a> {
   pub name: [u8; 4],
   pub data: &'a [u8],
}

impl<'a> UnknownFrame<'a> {
   fn to_owned(&self) -> OwnedUnknownFrame {
      let mut owned_bytes = vec![0; self.data.len()].into_boxed_slice();
      owned_bytes.copy_from_slice(self.data);

      OwnedUnknownFrame {
         name: self.name,
         data: owned_bytes,
      }
   }
}

#[derive(Clone, Debug)]
pub struct OwnedUnknownFrame {
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
            name,
            reason: FrameParseErrorReason::EmptyFrame,
         }));
      }

      let frame_bytes = &self.content[self.cursor..self.cursor + frame_size as usize];

      let result: Result<Frame, FrameParseErrorReason> = try {
         match &name {
            b"COMM" => Frame::COMM(decode_lang_description_text(frame_bytes)?),
            b"PRIV" => decode_priv_frame(frame_bytes)?,
            b"TALB" => Frame::TALB(decode_text_frame(frame_bytes)?),
            b"TBPM" => Frame::TBPM(decode_text_frame(frame_bytes)?.parse()?),
            b"TCOM" => Frame::TCOM(decode_text_frame(frame_bytes)?),
            b"TCON" => decode_genre_frame(frame_bytes)?,
            b"TDEN" => Frame::TDEN(decode_text_frame(frame_bytes)?.parse()?),
            b"TDOR" => Frame::TDOR(decode_text_frame(frame_bytes)?.parse()?),
            b"TDLY" => Frame::TDLY(decode_text_frame(frame_bytes)?.parse()?),
            b"TDRC" => Frame::TDRC(decode_text_frame(frame_bytes)?.parse()?),
            b"TDRL" => Frame::TDRL(decode_text_frame(frame_bytes)?.parse()?),
            b"TDTG" => Frame::TDTG(decode_text_frame(frame_bytes)?.parse()?),
            b"TENC" => Frame::TENC(decode_text_frame(frame_bytes)?),
            b"TEXT" => Frame::TEXT(decode_text_frame(frame_bytes)?),
            b"TIT1" => Frame::TIT1(decode_text_frame(frame_bytes)?),
            b"TIT2" => Frame::TIT2(decode_text_frame(frame_bytes)?),
            b"TIT3" => Frame::TIT3(decode_text_frame(frame_bytes)?),
            b"TLEN" => Frame::TLEN(decode_text_frame(frame_bytes)?.parse()?),
            b"TMOO" => Frame::TMOO(decode_text_frame(frame_bytes)?),
            b"TOPE" => Frame::TOPE(decode_text_frame(frame_bytes)?),
            b"TPE1" => Frame::TPE1(decode_text_frame(frame_bytes)?),
            b"TPE2" => Frame::TPE2(decode_text_frame(frame_bytes)?),
            b"TPE3" => Frame::TPE3(decode_text_frame(frame_bytes)?),
            b"TPOS" => Frame::TPOS(decode_text_frame(frame_bytes)?.parse()?),
            b"TPUB" => Frame::TPUB(decode_text_frame(frame_bytes)?),
            b"TRCK" => Frame::TRCK(decode_text_frame(frame_bytes)?.parse()?),
            b"TSOP" => Frame::TSOP(decode_text_frame(frame_bytes)?),
            b"TSSE" => Frame::TSSE(decode_text_frame(frame_bytes)?),
            b"TXXX" => decode_txxx_frame(frame_bytes)?,
            b"USLT" => Frame::USLT(decode_lang_description_text(frame_bytes)?),
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

#[derive(Clone, Debug)]
pub struct FrameParseError {
   pub name: [u8; 4],
   pub reason: FrameParseErrorReason,
}

#[derive(Clone, Debug)]
pub enum FrameParseErrorReason {
   EmptyFrame,
   TextDecodeError(TextDecodeError),
   ParseTrackError(ParseTrackError),
   ParseDateError(ParseDateError),
   ParseIntError(ParseIntError),
   MissingNullTerminator,
   FrameTooSmall,
}

impl From<ParseIntError> for FrameParseErrorReason {
   fn from(e: ParseIntError) -> FrameParseErrorReason {
      FrameParseErrorReason::ParseIntError(e)
   }
}

impl From<TextDecodeError> for FrameParseErrorReason {
   fn from(e: TextDecodeError) -> FrameParseErrorReason {
      FrameParseErrorReason::TextDecodeError(e)
   }
}

impl From<ParseTrackError> for FrameParseErrorReason {
   fn from(e: ParseTrackError) -> FrameParseErrorReason {
      FrameParseErrorReason::ParseTrackError(e)
   }
}

impl From<ParseDateError> for FrameParseErrorReason {
   fn from(e: ParseDateError) -> FrameParseErrorReason {
      FrameParseErrorReason::ParseDateError(e)
   }
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub enum ParseTrackError {
   InvalidTrackNumber(ParseIntError),
}

impl From<ParseIntError> for ParseTrackError {
   fn from(e: ParseIntError) -> ParseTrackError {
      ParseTrackError::InvalidTrackNumber(e)
   }
}

#[derive(Clone, Debug)]
pub enum ParseDateError {
   MissingYear,
   ParseIntError(ParseIntError),
}

impl From<ParseIntError> for ParseDateError {
   fn from(e: ParseIntError) -> ParseDateError {
      ParseDateError::ParseIntError(e)
   }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum TextEncoding {
   ISO8859,
   UTF16BOM,
   UTF16BE,
   UTF8,
}

impl TryFrom<u8> for TextEncoding {
   type Error = TextDecodeError;

   fn try_from(v: u8) -> Result<TextEncoding, TextDecodeError> {
      match v {
         0 => Ok(TextEncoding::ISO8859),
         1 => Ok(TextEncoding::UTF16BOM),
         2 => Ok(TextEncoding::UTF16BE),
         3 => Ok(TextEncoding::UTF8),
         _ => Err(TextDecodeError::UnknownEncoding(v)),
      }
   }
}

impl TextEncoding {
   fn has_two_trailing_nulls(self) -> bool {
      self == TextEncoding::UTF16BOM || self == TextEncoding::UTF16BE
   }
}

fn decode_text_segment(encoding: TextEncoding, text_slice: &[u8]) -> Result<Cow<str>, TextDecodeError> {
   match encoding {
      TextEncoding::ISO8859 => Ok(text_slice.iter().map(|c| *c as char).collect()),
      TextEncoding::UTF16BOM => {
         if text_slice[0..2] == [0xFE, 0xFF] {
            // Big endian
            unimplemented!();
         }
         if text_slice.len() % 2 != 0 {
            return Err(TextDecodeError::InvalidUtf16);
         }
         // The intermediate buffer is needed due to alignment concerns, I think
         let mut buffer = vec![0u16; text_slice.len() / 2].into_boxed_slice();
         unsafe {
            std::ptr::copy_nonoverlapping::<u8>(text_slice.as_ptr(), buffer.as_mut_ptr() as *mut u8, text_slice.len())
         };
         Ok(Cow::Owned(String::from_utf16(&buffer[1..])?)) // 1.. to skip BOM
      }
      TextEncoding::UTF16BE => unimplemented!(),
      TextEncoding::UTF8 => Ok(Cow::Borrowed(std::str::from_utf8(text_slice)?)),
   }
}

fn cut_trailing_nulls_if_present(encoding: TextEncoding, slice: &mut &[u8]) {
   if encoding.has_two_trailing_nulls() && slice.ends_with(b"\0\0") {
      *slice = &slice[..slice.len() - 2]
   } else if !encoding.has_two_trailing_nulls() && slice[slice.len() - 1] == 0 {
      *slice = &slice[..slice.len() - 1]
   }
}

/// Panics if frame is 0 length.
fn decode_text_frame(frame: &[u8]) -> Result<Cow<str>, TextDecodeError> {
   if frame.len() == 1 {
      // Only encoding is present
      return Ok(Cow::Borrowed(""));
   }

   let encoding = TextEncoding::try_from(frame[0])?;

   let mut text_segment = &frame[1..frame.len()];
   cut_trailing_nulls_if_present(encoding, &mut text_segment);

   decode_text_segment(encoding, text_segment)
}

fn decode_priv_frame(frame_bytes: &[u8]) -> Result<Frame, FrameParseErrorReason> {
   let owner_end = match frame_bytes.iter().position(|x| *x == 0) {
      Some(v) => v,
      None => return Err(FrameParseErrorReason::MissingNullTerminator),
   };

   let data_ref = if owner_end + 1 == frame_bytes.len() {
      &[]
   } else {
      &frame_bytes[owner_end + 1..]
   };

   Ok(Frame::PRIV(Priv {
      owner: frame_bytes[0..owner_end].iter().map(|c| *c as char).collect(), // IS0 8859,
      data: data_ref,
   }))
}

fn decode_description_text(encoding: TextEncoding, bytes: &[u8]) -> Result<(Cow<str>, Cow<str>), FrameParseErrorReason> {
   let (description_end, text_start) = if encoding.has_two_trailing_nulls() {
      // @Performance exact_chunks?
      let description_end = match bytes[0..].chunks(2).position(|x| *x == [0, 0]) {
         Some(v) => v * 2,
         None => return Err(FrameParseErrorReason::MissingNullTerminator),
      };
      (description_end, description_end + 2)
   } else {
      let description_end = match bytes[0..].iter().position(|x| *x == 0) {
         Some(v) => v,
         None => return Err(FrameParseErrorReason::MissingNullTerminator),
      };
      (description_end, description_end + 1)
   };

   let description = decode_text_segment(encoding, &bytes[..description_end])?;

   let text = if text_start == bytes.len() {
      Cow::Borrowed("")
   } else {
      let mut temp = &bytes[text_start..];
      cut_trailing_nulls_if_present(encoding, &mut temp);
      decode_text_segment(encoding, temp)?
   };

   Ok((description, text))
}

fn decode_lang_description_text(frame_bytes: &[u8]) -> Result<LangDescriptionText, FrameParseErrorReason> {
   if frame_bytes.len() < 5 {
      return Err(FrameParseErrorReason::FrameTooSmall);
   }

   let encoding = TextEncoding::try_from(frame_bytes[0])?;

   let iso_639_2_lang = {
      let mut lang_code = [0; 3];
      lang_code.copy_from_slice(&frame_bytes[1..4]);
      lang_code
   };

   let (description, text) = decode_description_text(encoding, &frame_bytes[4..])?;

   Ok(LangDescriptionText {
      iso_639_2_lang,
      description,
      text,
   })
}

fn decode_txxx_frame(frame_bytes: &[u8]) -> Result<Frame, FrameParseErrorReason> {
   if frame_bytes.len() < 2 {
      return Err(FrameParseErrorReason::FrameTooSmall);
   }

   let encoding = TextEncoding::try_from(frame_bytes[0])?;

   let (description, text) = decode_description_text(encoding, &frame_bytes[1..])?;

   Ok(Frame::TXXX(Txxx {
      description,
      text,
   }))
}


fn decode_genre_frame(frame_bytes: &[u8]) -> Result<Frame, TextDecodeError> {
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
   Ok(Frame::TCON(genre))
}
