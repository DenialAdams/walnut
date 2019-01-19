use super::synchsafe_u32_to_u32;
use bitflags::bitflags;
use byteorder::{BigEndian, ByteOrder};
use std::collections::HashMap;
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

bitflags! {
   pub(super) struct ExtendedHeaderFlags: u8 {
      const TAG_IS_UPDATE = 0b0100_0000;
      const CRC_DATA_PRESENT = 0b0010_0000;
      const TAG_RESTRICTIONS = 0b0001_0000;
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
pub struct Frame {
   pub data: FrameData,
   pub group: Option<u8>,
}

#[derive(Clone, Debug)]
pub enum FrameData {
   COMM(LangDescriptionText),
   PRIV(Priv),
   RVRB(Reverb),
   TALB(Vec<String>),
   TBPM(Vec<u64>),
   TCOM(Vec<String>),
   TCON(Vec<String>),
   TCOP(Vec<Copyright>),
   TDEN(Vec<Date>),
   TDLY(Vec<u64>),
   TDOR(Vec<Date>),
   TDRC(Vec<Date>),
   TDRL(Vec<Date>),
   TDTG(Vec<Date>),
   TENC(Vec<String>),
   TEXT(Vec<String>),
   TIPL(HashMap<String, String>),
   TIT1(Vec<String>),
   TIT2(Vec<String>),
   TIT3(Vec<String>),
   TLEN(Vec<u64>),
   TMCL(HashMap<String, String>),
   TMOO(Vec<String>),
   TOAL(Vec<String>),
   TOFN(Vec<String>),
   TOLY(Vec<String>),
   TOPE(Vec<String>),
   TOWN(Vec<String>),
   TPE1(Vec<String>),
   TPE2(Vec<String>),
   TPE3(Vec<String>),
   TPE4(Vec<String>),
   TPOS(Vec<Track>),
   TPRO(Vec<Copyright>),
   TPUB(Vec<String>),
   TRCK(Vec<Track>),
   TRSN(Vec<String>),
   TRSO(Vec<String>),
   TSOA(Vec<String>),
   TSOP(Vec<String>),
   TSOT(Vec<String>),
   TSRC(Vec<String>),
   TSSE(Vec<String>),
   TSST(Vec<String>),
   TXXX(Txxx),
   USLT(LangDescriptionText),
   WCOM(String),
   WCOP(String),
   WOAF(String),
   WOAR(String),
   WOAS(String),
   WORS(String),
   WPAY(String),
   WPUB(String),
   Unknown(Unknown),
}

#[derive(Clone, Debug)]
pub struct LangDescriptionText {
   pub iso_639_2_lang: [u8; 3],
   pub description: String,
   pub text: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Txxx {
   pub description: String,
   pub text: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Priv {
   pub owner: String,
   pub data: Box<[u8]>,
}

#[derive(Clone, Debug)]
pub struct Copyright {
   pub year: u16,
   pub message: String,
}

#[derive(Clone, Debug)]
pub struct Reverb {
   pub ms_left: u16,
   pub ms_right: u16,
   pub bounces_left: u8,
   pub bounces_right: u8,
   pub feedback_left_to_left: u8,
   pub feedback_left_to_right: u8,
   pub feedback_right_to_right: u8,
   pub feedback_right_to_left: u8,
   pub premix_left_to_right: u8,
   pub premix_right_to_left: u8,
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
pub struct Unknown {
   pub name: [u8; 4],
   pub data: Box<[u8]>,
}

fn map_parse<T: FromStr>(str_vec: Vec<String>) -> Result<Vec<T>, T::Err> {
   let mut new_vec = Vec::new();
   for item in str_vec {
      new_vec.push(item.parse()?);
   }
   Ok(new_vec)
}

impl Iterator for Parser {
   type Item = Result<Frame, FrameParseError>;

   fn next(&mut self) -> Option<Result<Frame, FrameParseError>> {
      // Each frame must be at least 10 bytes
      if self.content.len() - self.cursor < 10 {
         return None;
      }

      let mut name: [u8; 4] = [0; 4];
      name.copy_from_slice(&self.content[self.cursor..self.cursor + 4]);
      if &name == b"\0\0\0\0" {
         // Padding
         return None;
      }

      let mut frame_size = synchsafe_u32_to_u32(BigEndian::read_u32(&self.content[self.cursor + 4..self.cursor + 8]));
      let frame_flags_raw = BigEndian::read_u16(&self.content[self.cursor + 8..self.cursor + 10]);
      let frame_flags = FrameFlags::from_bits_truncate(frame_flags_raw);

      self.cursor += 10;

      let mut group = None;
      if frame_flags.contains(FrameFlags::GROUPING_IDENTITY) {
         group = Some(self.content[self.cursor]);
         self.cursor += 1;
         // frame size includes the flags, so we have to adjust it, as the code after this
         // assumes frame size == data size.
         // saturating sub so we don't underflow on a bad frame size input
         frame_size.saturating_sub(1);
      }

      if frame_flags.contains(FrameFlags::DATA_LENGTH_INDICATOR) {
         // TODO: we only need to use this when we implement compression,
         // and some forms of encryption.
         frame_size = synchsafe_u32_to_u32(BigEndian::read_u32(&self.content[self.cursor..self.cursor + 4]));
         self.cursor += 4;
      }

      let frame_bytes = &self.content[self.cursor..self.cursor + frame_size as usize];

      let result: Result<FrameData, FrameParseErrorReason> = try {
         match &name {
            b"COMM" => FrameData::COMM(decode_lang_description_text(frame_bytes)?),
            b"PRIV" => decode_priv_frame(frame_bytes)?,
            b"RVRB" => FrameData::RVRB(decode_reverb_frame(frame_bytes)?),
            b"TALB" => FrameData::TALB(decode_text_frame(frame_bytes)?),
            b"TBPM" => FrameData::TBPM(map_parse(decode_text_frame(frame_bytes)?)?),
            b"TCOM" => FrameData::TCOM(decode_text_frame(frame_bytes)?),
            b"TCON" => decode_genre_frame(frame_bytes)?,
            b"TCOP" => FrameData::TCOP({
               let mut new_vec = Vec::new();
               for segment in decode_text_frame(frame_bytes)? {
                  new_vec.push(decode_copyright_frame(segment)?);
               }
               new_vec
            }),
            b"TDEN" => FrameData::TDEN(map_parse(decode_text_frame(frame_bytes)?)?),
            b"TDOR" => FrameData::TDOR(map_parse(decode_text_frame(frame_bytes)?)?),
            b"TDLY" => FrameData::TDLY(map_parse(decode_text_frame(frame_bytes)?)?),
            b"TDRC" => FrameData::TDRC(map_parse(decode_text_frame(frame_bytes)?)?),
            b"TDRL" => FrameData::TDRL(map_parse(decode_text_frame(frame_bytes)?)?),
            b"TDTG" => FrameData::TDTG(map_parse(decode_text_frame(frame_bytes)?)?),
            b"TENC" => FrameData::TENC(decode_text_frame(frame_bytes)?),
            b"TEXT" => FrameData::TEXT(decode_text_frame(frame_bytes)?),
            b"TIPL" => FrameData::TIPL(decode_text_map_frame(frame_bytes)?),
            b"TIT1" => FrameData::TIT1(decode_text_frame(frame_bytes)?),
            b"TIT2" => FrameData::TIT2(decode_text_frame(frame_bytes)?),
            b"TIT3" => FrameData::TIT3(decode_text_frame(frame_bytes)?),
            b"TLEN" => FrameData::TLEN(map_parse(decode_text_frame(frame_bytes)?)?),
            b"TMCL" => FrameData::TMCL(decode_text_map_frame(frame_bytes)?),
            b"TMOO" => FrameData::TMOO(decode_text_frame(frame_bytes)?),
            b"TOAL" => FrameData::TOAL(decode_text_frame(frame_bytes)?),
            b"TOFN" => FrameData::TOFN(decode_text_frame(frame_bytes)?),
            b"TOLY" => FrameData::TOLY(decode_text_frame(frame_bytes)?),
            b"TOPE" => FrameData::TOPE(decode_text_frame(frame_bytes)?),
            b"TOWN" => FrameData::TOWN(decode_text_frame(frame_bytes)?),
            b"TPE1" => FrameData::TPE1(decode_text_frame(frame_bytes)?),
            b"TPE2" => FrameData::TPE2(decode_text_frame(frame_bytes)?),
            b"TPE3" => FrameData::TPE3(decode_text_frame(frame_bytes)?),
            b"TPE4" => FrameData::TPE4(decode_text_frame(frame_bytes)?),
            b"TPOS" => FrameData::TPOS(map_parse(decode_text_frame(frame_bytes)?)?),
            b"TPRO" => FrameData::TPRO({
               let mut new_vec = Vec::new();
               for segment in decode_text_frame(frame_bytes)? {
                  new_vec.push(decode_copyright_frame(segment)?);
               }
               new_vec
            }),
            b"TPUB" => FrameData::TPUB(decode_text_frame(frame_bytes)?),
            b"TRCK" => FrameData::TRCK(map_parse(decode_text_frame(frame_bytes)?)?),
            b"TRSN" => FrameData::TRSN(decode_text_frame(frame_bytes)?),
            b"TRSO" => FrameData::TRSO(decode_text_frame(frame_bytes)?),
            b"TSOA" => FrameData::TSOA(decode_text_frame(frame_bytes)?),
            b"TSOP" => FrameData::TSOP(decode_text_frame(frame_bytes)?),
            b"TSOT" => FrameData::TSOT(decode_text_frame(frame_bytes)?),
            b"TSRC" => FrameData::TSRC(decode_text_frame(frame_bytes)?),
            b"TSSE" => FrameData::TSSE(decode_text_frame(frame_bytes)?),
            b"TSST" => FrameData::TSST(decode_text_frame(frame_bytes)?),
            b"TXXX" => decode_txxx_frame(frame_bytes)?,
            b"USLT" => FrameData::USLT(decode_lang_description_text(frame_bytes)?),
            b"WCOM" => FrameData::WCOM(decode_url_frame(frame_bytes)),
            b"WCOP" => FrameData::WCOP(decode_url_frame(frame_bytes)),
            b"WOAF" => FrameData::WOAF(decode_url_frame(frame_bytes)),
            b"WOAR" => FrameData::WOAR(decode_url_frame(frame_bytes)),
            b"WOAS" => FrameData::WOAS(decode_url_frame(frame_bytes)),
            b"WORS" => FrameData::WORS(decode_url_frame(frame_bytes)),
            b"WPAY" => FrameData::WPAY(decode_url_frame(frame_bytes)),
            b"WPUB" => FrameData::WPUB(decode_url_frame(frame_bytes)),
            _ => FrameData::Unknown(Unknown {
               name,
               data: Box::from(frame_bytes),
            }),
         }
      };

      self.cursor += frame_size as usize;

      Some(
         result
            .map(|data| Frame { data, group })
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
   FrameTooSmall,
   MissingNullTerminator,
   MissingValueInMapFrame,
   ParseDateError(ParseDateError),
   ParseIntError(ParseIntError),
   ParseTrackError(ParseTrackError),
   TextDecodeError(TextDecodeError),
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

   fn get_trailing_null_slice(self) -> &'static [u8] {
      if self.has_two_trailing_nulls() {
         b"\0\0"
      } else {
         b"\0"
      }
   }
}

fn decode_text_segments(encoding: TextEncoding, mut text_slice: &[u8]) -> Result<Vec<String>, TextDecodeError> {
   let separator = encoding.get_trailing_null_slice();
   let mut text_segments = Vec::new();
   while let Some(pos) = text_slice
      .chunks_exact(separator.len())
      .position(|x| x == separator)
      .map(|x| x * separator.len())
   {
      text_segments.push(decode_text_segment(encoding, &text_slice[..pos])?);
      text_slice = &text_slice[pos + separator.len()..];
   }

   if !text_slice.is_empty() {
      // There is more text, but no null terminator
      // We assume that it ends with the frame
      text_segments.push(decode_text_segment(encoding, text_slice)?);
   }

   Ok(text_segments)
}

fn decode_text_segment(encoding: TextEncoding, text_slice: &[u8]) -> Result<String, TextDecodeError> {
   match encoding {
      TextEncoding::ISO8859 => Ok(text_slice.iter().map(|c| *c as char).collect()),
      TextEncoding::UTF16BOM => {
         if text_slice.len() % 2 != 0 {
            return Err(TextDecodeError::InvalidUtf16);
         }
         // @Speed this can be uninitialized
         // The intermediate buffer is needed due to alignment concerns
         let mut buffer = vec![0u16; text_slice.len() / 2].into_boxed_slice();
         if text_slice[0..2] == [0xFE, 0xFF] {
            text_slice.chunks(2).enumerate().for_each(|(i, c)| {
               buffer[i] = (u16::from(c[1]) << 8) & u16::from(c[0]);
            });
         } else {
            unsafe {
               std::ptr::copy_nonoverlapping::<u8>(
                  text_slice.as_ptr(),
                  buffer.as_mut_ptr() as *mut u8,
                  text_slice.len(),
               )
            };
         }
         Ok(String::from_utf16(&buffer[1..])?) // 1.. to skip BOM
      }
      TextEncoding::UTF16BE => {
         if text_slice.len() % 2 != 0 {
            return Err(TextDecodeError::InvalidUtf16);
         }
         // @Speed this can be uninitialized
         // The intermediate buffer is needed due to alignment concerns
         let mut buffer = vec![0u16; text_slice.len() / 2].into_boxed_slice();
         text_slice.chunks(2).enumerate().for_each(|(i, c)| {
            buffer[i] = (u16::from(c[1]) << 8) & u16::from(c[0]);
         });
         Ok(String::from_utf16(&buffer)?) // No BOM
      }
      TextEncoding::UTF8 => Ok(String::from(std::str::from_utf8(text_slice)?)),
   }
}

/// Panics if frame is 0 length.
fn decode_text_frame(frame: &[u8]) -> Result<Vec<String>, TextDecodeError> {
   let encoding = TextEncoding::try_from(frame[0])?;
   decode_text_segments(encoding, &frame[1..frame.len()])
}

/// Panics if frame is 0 length.
fn decode_text_map_frame(frame: &[u8]) -> Result<HashMap<String, String>, FrameParseErrorReason> {
   let encoding = TextEncoding::try_from(frame[0])?;
   let separator = encoding.get_trailing_null_slice();
   let mut start = 1;
   let mut segment_iter = frame[1..]
      .chunks_exact(separator.len())
      .enumerate()
      .filter(|(_, x)| *x == separator)
      .map(|(i, _)| i * separator.len());
   let mut map = HashMap::new();
   loop {
      let (opt_k_end, opt_v_end) = (segment_iter.next(), segment_iter.next());
      match (opt_k_end, opt_v_end) {
         (Some(k_end), Some(v_end)) => {
            let key = decode_text_segment(encoding, &frame[start..k_end])?;
            let value = decode_text_segment(encoding, &frame[k_end + separator.len()..v_end])?;
            start = v_end + separator.len();
            map.insert(key, value);
         }
         (Some(k_end), None) => {
            if k_end + separator.len() == frame.len() {
               return Err(FrameParseErrorReason::MissingValueInMapFrame);
            }
            let key = decode_text_segment(encoding, &frame[start..k_end])?;
            let value = decode_text_segment(encoding, &frame[k_end + separator.len()..])?;
            map.insert(key, value);
            break;
         }
         (None, _) => break,
      }
   }
   Ok(map)
}

fn decode_priv_frame(frame_bytes: &[u8]) -> Result<FrameData, FrameParseErrorReason> {
   let owner_end = match frame_bytes.iter().position(|x| *x == 0) {
      Some(v) => v,
      None => return Err(FrameParseErrorReason::MissingNullTerminator),
   };

   let data_ref = if owner_end + 1 == frame_bytes.len() {
      &[]
   } else {
      &frame_bytes[owner_end + 1..]
   };

   Ok(FrameData::PRIV(Priv {
      owner: frame_bytes[0..owner_end].iter().map(|c| *c as char).collect(), // IS0 8859,
      data: Box::from(data_ref),
   }))
}

fn decode_description_text(
   encoding: TextEncoding,
   bytes: &[u8],
) -> Result<(String, Vec<String>), FrameParseErrorReason> {
   let separator = encoding.get_trailing_null_slice();
   let description_end = match bytes
      .chunks_exact(separator.len())
      .position(|x| x == separator)
      .map(|x| x * separator.len())
   {
      Some(v) => v,
      None => return Err(FrameParseErrorReason::MissingNullTerminator),
   };

   let description = decode_text_segment(encoding, &bytes[..description_end])?;
   let text = decode_text_segments(encoding, &bytes[description_end + separator.len()..])?;

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

fn decode_txxx_frame(frame_bytes: &[u8]) -> Result<FrameData, FrameParseErrorReason> {
   if frame_bytes.len() < 2 {
      return Err(FrameParseErrorReason::FrameTooSmall);
   }

   let encoding = TextEncoding::try_from(frame_bytes[0])?;

   let (description, text) = decode_description_text(encoding, &frame_bytes[1..])?;

   Ok(FrameData::TXXX(Txxx { description, text }))
}

fn decode_genre_frame(frame_bytes: &[u8]) -> Result<FrameData, TextDecodeError> {
   let mut genres = decode_text_frame(frame_bytes)?;
   for genre in genres.iter_mut() {
      match genre.as_ref() {
         "0" => *genre = String::from("Blues"),
         "1" => *genre = String::from("Classic Rock"),
         "2" => *genre = String::from("Country"),
         "3" => *genre = String::from("Dance"),
         "4" => *genre = String::from("Disco"),
         "5" => *genre = String::from("Funk"),
         "6" => *genre = String::from("Grunge"),
         "7" => *genre = String::from("Hip-Hop"),
         "8" => *genre = String::from("Jazz"),
         "9" => *genre = String::from("Metal"),
         "10" => *genre = String::from("New Age"),
         "11" => *genre = String::from("Oldies"),
         "12" => *genre = String::from("Other"),
         "13" => *genre = String::from("Pop"),
         "14" => *genre = String::from("R&B"),
         "15" => *genre = String::from("Rap"),
         "16" => *genre = String::from("Reggae"),
         "17" => *genre = String::from("Rock"),
         "18" => *genre = String::from("Techno"),
         "19" => *genre = String::from("Industrial"),
         "20" => *genre = String::from("Alternative"),
         "21" => *genre = String::from("Ska"),
         "22" => *genre = String::from("Death Metal"),
         "23" => *genre = String::from("Pranks"),
         "24" => *genre = String::from("Soundtrack"),
         "25" => *genre = String::from("Euro-Techno"),
         "26" => *genre = String::from("Ambient"),
         "27" => *genre = String::from("Trip-Hop"),
         "28" => *genre = String::from("Vocal"),
         "29" => *genre = String::from("Jazz+Funk"),
         "30" => *genre = String::from("Fusion"),
         "31" => *genre = String::from("Trance"),
         "32" => *genre = String::from("Classical"),
         "33" => *genre = String::from("Instrumental"),
         "34" => *genre = String::from("Acid"),
         "35" => *genre = String::from("House"),
         "36" => *genre = String::from("Game"),
         "37" => *genre = String::from("Sound Clip"),
         "38" => *genre = String::from("Gospel"),
         "39" => *genre = String::from("Noise"),
         "40" => *genre = String::from("AlternRock"),
         "41" => *genre = String::from("Bass"),
         "42" => *genre = String::from("Soul"),
         "43" => *genre = String::from("Punk"),
         "44" => *genre = String::from("Space"),
         "45" => *genre = String::from("Meditative"),
         "46" => *genre = String::from("Instrumental Pop"),
         "47" => *genre = String::from("Instrumental Rock"),
         "48" => *genre = String::from("Ethnic"),
         "49" => *genre = String::from("Gothic"),
         "50" => *genre = String::from("Darkwave"),
         "51" => *genre = String::from("Techno-Industrial"),
         "52" => *genre = String::from("Electronic"),
         "53" => *genre = String::from("Pop-Folk"),
         "54" => *genre = String::from("Eurodance"),
         "55" => *genre = String::from("Dream"),
         "56" => *genre = String::from("Southern Rock"),
         "57" => *genre = String::from("Comedy"),
         "58" => *genre = String::from("Cult"),
         "59" => *genre = String::from("Gangsta"),
         "60" => *genre = String::from("Top 40"),
         "61" => *genre = String::from("Christian Rap"),
         "62" => *genre = String::from("Pop/Funk"),
         "63" => *genre = String::from("Jungle"),
         "64" => *genre = String::from("Native American"),
         "65" => *genre = String::from("Cabaret"),
         "66" => *genre = String::from("New Wave"),
         "67" => *genre = String::from("Psychedelic"),
         "68" => *genre = String::from("Rave"),
         "69" => *genre = String::from("Showtunes"),
         "70" => *genre = String::from("Trailer"),
         "71" => *genre = String::from("Lo-Fi"),
         "72" => *genre = String::from("Tribal"),
         "73" => *genre = String::from("Acid Punk"),
         "74" => *genre = String::from("Acid Jazz"),
         "75" => *genre = String::from("Polka"),
         "76" => *genre = String::from("Retro"),
         "77" => *genre = String::from("Musical"),
         "78" => *genre = String::from("Rock & Roll"),
         "79" => *genre = String::from("Hard Rock"),
         "RX" => *genre = String::from("Remix"),
         "CR" => *genre = String::from("Cover"),
         _ => (),
      };
   }
   Ok(FrameData::TCON(genres))
}

fn decode_copyright_frame(mut text: String) -> Result<Copyright, FrameParseErrorReason> {
   if text.len() < 4 {
      return Err(FrameParseErrorReason::FrameTooSmall);
   }
   let year = text[0..4].parse()?;
   let text_bytes = unsafe { text.as_mut_vec() };
   unsafe {
      if text_bytes.len() > 4 && text_bytes[4] == b' ' {
         text_bytes.set_len(text_bytes.len() - 5);
         std::ptr::copy(text_bytes.as_ptr().offset(5), text_bytes.as_mut_ptr(), text_bytes.len());
      } else {
         text_bytes.set_len(text_bytes.len() - 4);
         std::ptr::copy(text_bytes.as_ptr().offset(4), text_bytes.as_mut_ptr(), text_bytes.len());
      }
   }
   Ok(Copyright { year, message: text })
}

// We don't do full URL parsing (for instance; with the URL crate)
// because the id3 spec says that relative URLs are always ok
// and that doesn't jive with general URL parsing
fn decode_url_frame(mut frame: &[u8]) -> String {
   if frame[frame.len() - 1] == 0 {
      frame = &frame[..frame.len() - 1];
   }

   frame.iter().map(|c| *c as char).collect()
}

fn decode_reverb_frame(frame: &[u8]) -> Result<Reverb, FrameParseErrorReason> {
   if frame.len() < 12 {
      return Err(FrameParseErrorReason::FrameTooSmall);
   }

   Ok(Reverb {
      ms_left: BigEndian::read_u16(&frame[0..2]),
      ms_right: BigEndian::read_u16(&frame[2..4]),
      bounces_left: frame[4],
      bounces_right: frame[5],
      feedback_left_to_left: frame[6],
      feedback_left_to_right: frame[7],
      feedback_right_to_right: frame[8],
      feedback_right_to_left: frame[9],
      premix_left_to_right: frame[10],
      premix_right_to_left: frame[11],
   })
}
