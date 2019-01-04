#![feature(try_blocks, try_from, duration_as_u128)]

extern crate byteorder;
#[macro_use]
extern crate log;
extern crate walkdir;
#[macro_use]
extern crate bitflags;
extern crate pretty_env_logger;

mod id3;

use std::fs::File;
use std::time::Instant;
use walkdir::WalkDir;

fn main() {
   pretty_env_logger::init();

   // TODO: use map_or_else when it is stable
   // WalkDir::new("C:\\music").into_iter().map_or_else(|e| warn!("Failed to open file/directory: {}", e), |v| v).filter(|v| v.file_type().is_file()).filter(is_mp3_file);
   let mp3_files: Vec<_> = WalkDir::new("C:\\music")
      .into_iter()
      .flat_map(|v| match v {
         Ok(v) => Some(v),
         Err(e) => {
            warn!("Failed to open file/directory: {}", e);
            None
         }
      })
      .filter(|v| v.file_type().is_file() && v.file_name().to_string_lossy().split('.').last() == Some("mp3"))
      .collect();

   let start = Instant::now();
   let mut ok_counter: u64 = 0;
   let mut ignored_counter: u64 = 0;
   for entry in mp3_files.into_iter() {
      println!("{}", entry.path().display());

      let mut f = File::open(entry.path()).unwrap();
      match id3::parse_source(&mut f) {
         Ok(parser) => {
            println!("ID3v24");
            for frame in parser {
               match frame {
                  Err(e) => warn!(
                     "Failed to parse frame {}: {:?}",
                     String::from_utf8_lossy(&e.name),
                     e.reason
                  ),
                  Ok(frame) => match frame.data {
                     id3::v24::FrameData::COMM(x) => println!("Comment: {:?}", x),
                     id3::v24::FrameData::PRIV(x) => println!("Private: {:?}", x),
                     id3::v24::FrameData::RVRB(x) => println!("Reverb: {:?}", x),
                     id3::v24::FrameData::TALB(x) => println!("Album: {:?}", x),
                     id3::v24::FrameData::TBPM(x) => println!("BPM: {:?}", x),
                     id3::v24::FrameData::TCOM(x) => println!("Composer: {:?}", x),
                     id3::v24::FrameData::TCON(x) => println!("Genre: {:?}", x),
                     id3::v24::FrameData::TCOP(x) => println!("Copyright: {:?}", x),
                     id3::v24::FrameData::TDEN(x) => println!("Encoding Date: {:?}", x),
                     id3::v24::FrameData::TDOR(x) => println!("Original Release Date: {:?}", x),
                     id3::v24::FrameData::TDLY(x) => println!("Delay: {:?}ms", x),
                     id3::v24::FrameData::TDRC(x) => println!("Recording Date: {:?}", x),
                     id3::v24::FrameData::TDRL(x) => println!("Release Date: {:?}", x),
                     id3::v24::FrameData::TDTG(x) => println!("Tagging Date: {:?}", x),
                     id3::v24::FrameData::TENC(x) => println!("Encoded by: {:?}", x),
                     id3::v24::FrameData::TEXT(x) => println!("Lyricist/Text Writer: {:?}", x),
                     id3::v24::FrameData::TIPL(x) => println!("Involved People: {:?}", x),
                     id3::v24::FrameData::TIT1(x) => println!("Content group description: {:?}", x),
                     id3::v24::FrameData::TIT2(x) => println!("Title: {:?}", x),
                     id3::v24::FrameData::TIT3(x) => println!("Substitle/description refinement: {:?}", x),
                     id3::v24::FrameData::TLEN(x) => println!("Length: {:?}ms", x),
                     id3::v24::FrameData::TMCL(x) => println!("Musician Credits: {:?}", x),
                     id3::v24::FrameData::TMOO(x) => println!("Mood: {:?}", x),
                     id3::v24::FrameData::TOAL(x) => println!("Original Album Title: {:?}", x),
                     id3::v24::FrameData::TOFN(x) => println!("Original filename: {:?}", x),
                     id3::v24::FrameData::TOLY(x) => println!("Original Lyricist/Text Writer: {:?}", x),
                     id3::v24::FrameData::TOPE(x) => println!("Original Artist: {:?}", x),
                     id3::v24::FrameData::TOWN(x) => println!("File Owner/Licensee: {:?}", x),
                     id3::v24::FrameData::TPE1(x) => println!("Artist: {:?}", x),
                     id3::v24::FrameData::TPE2(x) => println!("Album Artist: {:?}", x),
                     id3::v24::FrameData::TPE3(x) => println!("Conductor: {:?}", x),
                     id3::v24::FrameData::TPE4(x) => {
                        println!("Interpreted, remixed, or otherwise modified by: {:?}", x)
                     }
                     id3::v24::FrameData::TPOS(x) => println!("CD: {:?}", x),
                     id3::v24::FrameData::TPRO(x) => println!("Production Copyright: {:?}", x),
                     id3::v24::FrameData::TPUB(x) => println!("Publisher: {:?}", x),
                     id3::v24::FrameData::TRCK(x) => println!("Track: {:?}", x),
                     id3::v24::FrameData::TRSN(x) => println!("Internet Radio Station Name: {:?}", x),
                     id3::v24::FrameData::TRSO(x) => println!("Internet Radio Station Owner: {:?}", x),
                     id3::v24::FrameData::TSOA(x) => println!("Album for sorting: {:?}", x),
                     id3::v24::FrameData::TSOP(x) => println!("Artist name for sorting: {:?}", x),
                     id3::v24::FrameData::TSOT(x) => println!("Title for sorting: {:?}", x),
                     id3::v24::FrameData::TSRC(x) => println!("ISRC: {:?}", x),
                     id3::v24::FrameData::TSSE(x) => println!("Encoding settings: {:?}", x),
                     id3::v24::FrameData::TSST(x) => println!("Set Subtitle: {:?}", x),
                     id3::v24::FrameData::TXXX(x) => println!("User defined text: {:?}", x),
                     id3::v24::FrameData::USLT(x) => println!("Lyrics: {:?}", x),
                     id3::v24::FrameData::WCOM(x) => println!("Commercial Information URL: {:?}", x),
                     id3::v24::FrameData::WCOP(x) => println!("Copyright/Legal Info URL: {:?}", x),
                     id3::v24::FrameData::WOAF(x) => println!("Audio File URL: {:?}", x),
                     id3::v24::FrameData::WOAR(x) => println!("Artist/Performer URL: {:?}", x),
                     id3::v24::FrameData::WOAS(x) => println!("Audio Source URL: {:?}", x),
                     id3::v24::FrameData::WORS(x) => println!("Internet Radio Station URL: {:?}", x),
                     id3::v24::FrameData::WPAY(x) => println!("Payment URL: {:?}", x),
                     id3::v24::FrameData::WPUB(x) => println!("Publisher URL: {:?}", x),
                     id3::v24::FrameData::Unknown(u) => println!("Unknown frame: {}", String::from_utf8_lossy(&u.name)),
                  },
               }
            }
            ok_counter += 1;
         }
         Err(e) => {
            match e {
               id3::TagParseError::NoTag => {
                  println!("No ID3");
               }
               id3::TagParseError::UnsupportedVersion(ver) => {
                  println!("ID3v2{}", ver);
               }
               id3::TagParseError::Io(io_err) => {
                  warn!("Failed to parse file: {}", io_err);
               }
            }
            ignored_counter += 1;
         }
      }
   }

   let elapsed = start.elapsed();
   info!(
      "Parsed {} mp3 files in {}ms ({:.2}ms avg)",
      ok_counter,
      elapsed.as_millis(),
      elapsed.as_millis() as f64 / ok_counter as f64
   );
   info!("Failed to parse {} mp3 files", ignored_counter);
}
