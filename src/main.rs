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
                  Ok(frame) => match frame {
                     id3::v24::Frame::COMM(x) => println!("Comment: {:?}", x),
                     id3::v24::Frame::PRIV(x) => println!("Private: {:?}", x),
                     id3::v24::Frame::TALB(x) => println!("Album: {:?}", x),
                     id3::v24::Frame::TBPM(x) => println!("BPM: {:?}", x),
                     id3::v24::Frame::TCOM(x) => println!("Composer: {:?}", x),
                     id3::v24::Frame::TCON(x) => println!("Genre: {:?}", x),
                     id3::v24::Frame::TCOP(x) => println!("Copyright: {:?}", x),
                     id3::v24::Frame::TDEN(x) => println!("Encoding Date: {:?}", x),
                     id3::v24::Frame::TDOR(x) => println!("Original Release Date: {:?}", x),
                     id3::v24::Frame::TDLY(x) => println!("Delay: {:?}ms", x),
                     id3::v24::Frame::TDRC(x) => println!("Recording Date: {:?}", x),
                     id3::v24::Frame::TDRL(x) => println!("Release Date: {:?}", x),
                     id3::v24::Frame::TDTG(x) => println!("Tagging Date: {:?}", x),
                     id3::v24::Frame::TENC(x) => println!("Encoded by: {:?}", x),
                     id3::v24::Frame::TEXT(x) => println!("Lyricist/Text Writer: {:?}", x),
                     id3::v24::Frame::TIPL(x) => println!("Involved People: {:?}", x),
                     id3::v24::Frame::TIT1(x) => println!("Content group description: {:?}", x),
                     id3::v24::Frame::TIT2(x) => println!("Title: {:?}", x),
                     id3::v24::Frame::TIT3(x) => println!("Substitle/description refinement: {:?}", x),
                     id3::v24::Frame::TLEN(x) => println!("Length: {:?}ms", x),
                     id3::v24::Frame::TMCL(x) => println!("Musician Credits: {:?}", x),
                     id3::v24::Frame::TMOO(x) => println!("Mood: {:?}", x),
                     id3::v24::Frame::TOAL(x) => println!("Original Album Title: {:?}", x),
                     id3::v24::Frame::TOFN(x) => println!("Original filename: {:?}", x),
                     id3::v24::Frame::TOLY(x) => println!("Original Lyricist/Text Writer: {:?}", x),
                     id3::v24::Frame::TOPE(x) => println!("Original Artist: {:?}", x),
                     id3::v24::Frame::TOWN(x) => println!("File Owner/Licensee: {:?}", x),
                     id3::v24::Frame::TPE1(x) => println!("Artist: {:?}", x),
                     id3::v24::Frame::TPE2(x) => println!("Album Artist: {:?}", x),
                     id3::v24::Frame::TPE3(x) => println!("Conductor: {:?}", x),
                     id3::v24::Frame::TPE4(x) => println!("Interpreted, remixed, or otherwise modified by: {:?}", x),
                     id3::v24::Frame::TPOS(x) => println!("CD: {:?}", x),
                     id3::v24::Frame::TPRO(x) => println!("Production Copyright: {:?}", x),
                     id3::v24::Frame::TPUB(x) => println!("Publisher: {:?}", x),
                     id3::v24::Frame::TRCK(x) => println!("Track: {:?}", x),
                     id3::v24::Frame::TRSN(x) => println!("Internet Radio Station Name: {:?}", x),
                     id3::v24::Frame::TRSO(x) => println!("Internet Radio Station Owner: {:?}", x),
                     id3::v24::Frame::TSOA(x) => println!("Album for sorting: {:?}", x),
                     id3::v24::Frame::TSOP(x) => println!("Artist name for sorting: {:?}", x),
                     id3::v24::Frame::TSOT(x) => println!("Title for sorting: {:?}", x),
                     id3::v24::Frame::TSRC(x) => println!("ISRC: {:?}", x),
                     id3::v24::Frame::TSSE(x) => println!("Encoding settings: {:?}", x),
                     id3::v24::Frame::TSST(x) => println!("Set Subtitle: {:?}", x),
                     id3::v24::Frame::TXXX(x) => println!("User defined text: {:?}", x),
                     id3::v24::Frame::USLT(x) => println!("Lyrics: {:?}", x),
                     id3::v24::Frame::WCOM(x) => println!("Commercial Information URL: {:?}", x),
                     id3::v24::Frame::WCOP(x) => println!("Copyright/Legal Info URL: {:?}", x),
                     id3::v24::Frame::WOAF(x) => println!("Audio File URL: {:?}", x),
                     id3::v24::Frame::WOAR(x) => println!("Artist/Performer URL: {:?}", x),
                     id3::v24::Frame::WOAS(x) => println!("Audio Source URL: {:?}", x),
                     id3::v24::Frame::WORS(x) => println!("Internet Radio Station URL: {:?}", x),
                     id3::v24::Frame::WPAY(x) => println!("Payment URL: {:?}", x),
                     id3::v24::Frame::WPUB(x) => println!("Publisher URL: {:?}", x),
                     id3::v24::Frame::Unknown(u) => println!("Unknown frame: {}", String::from_utf8_lossy(&u.name)),
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
