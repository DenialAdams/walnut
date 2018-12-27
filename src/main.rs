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
use walkdir::{DirEntry, WalkDir};

fn is_mp3_file(entry: &DirEntry) -> bool {
   if let Some(ref extension) = entry.file_name().to_string_lossy().split('.').last() {
      *extension == "mp3"
   } else {
      false
   }
}

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
      .filter(|v| v.file_type().is_file())
      .filter(is_mp3_file)
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
                     id3::v24::OwnedFrame::COMM(x) => println!("Comment: {:?}", x),
                     id3::v24::OwnedFrame::PRIV(x) => println!("Private: {:?}", x),
                     id3::v24::OwnedFrame::TALB(x) => println!("Album: {}", x),
                     id3::v24::OwnedFrame::TBPM(x) => println!("BPM: {:?}", x),
                     id3::v24::OwnedFrame::TCOM(x) => println!("Composer: {}", x),
                     id3::v24::OwnedFrame::TCON(x) => println!("Genre: {:?}", x),
                     id3::v24::OwnedFrame::TCOP(x) => println!("Copyright: {:?}", x),
                     id3::v24::OwnedFrame::TDEN(x) => println!("Encoding Date: {:?}", x),
                     id3::v24::OwnedFrame::TDOR(x) => println!("Original Release Date: {:?}", x),
                     id3::v24::OwnedFrame::TDLY(x) => println!("Delay: {:?}ms", x),
                     id3::v24::OwnedFrame::TDRC(x) => println!("Recording Date: {:?}", x),
                     id3::v24::OwnedFrame::TDRL(x) => println!("Release Date: {:?}", x),
                     id3::v24::OwnedFrame::TDTG(x) => println!("Tagging Date: {:?}", x),
                     id3::v24::OwnedFrame::TENC(x) => println!("Encoded by: {}", x),
                     id3::v24::OwnedFrame::TEXT(x) => println!("Lyricist/Text Writer: {}", x),
                     id3::v24::OwnedFrame::TIT1(x) => println!("Content group description: {}", x),
                     id3::v24::OwnedFrame::TIT2(x) => println!("Title: {}", x),
                     id3::v24::OwnedFrame::TIT3(x) => println!("Substitle/description refinement: {}", x),
                     id3::v24::OwnedFrame::TLEN(x) => println!("Length: {:?}ms", x),
                     id3::v24::OwnedFrame::TMOO(x) => println!("Mood: {}", x),
                     id3::v24::OwnedFrame::TOAL(x) => println!("Original Album Title: {}", x),
                     id3::v24::OwnedFrame::TOFN(x) => println!("Original filename: {}", x),
                     id3::v24::OwnedFrame::TOLY(x) => println!("Original Lyricist/Text Writer: {}", x),
                     id3::v24::OwnedFrame::TOPE(x) => println!("Original Artist: {}", x),
                     id3::v24::OwnedFrame::TOWN(x) => println!("File Owner/Licensee: {}", x),
                     id3::v24::OwnedFrame::TPE1(x) => println!("Artist: {}", x),
                     id3::v24::OwnedFrame::TPE2(x) => println!("Album Artist: {}", x),
                     id3::v24::OwnedFrame::TPE3(x) => println!("Conductor: {}", x),
                     id3::v24::OwnedFrame::TPE4(x) => println!("Interpreted, remixed, or otherwise modified by: {}", x),
                     id3::v24::OwnedFrame::TPOS(x) => println!("CD: {:?}", x),
                     id3::v24::OwnedFrame::TPRO(x) => println!("Production Copyright: {:?}", x),
                     id3::v24::OwnedFrame::TPUB(x) => println!("Publisher: {:?}", x),
                     id3::v24::OwnedFrame::TRCK(x) => println!("Track: {:?}", x),
                     id3::v24::OwnedFrame::TRSN(x) => println!("Internet Radio Station Name: {}", x),
                     id3::v24::OwnedFrame::TRSO(x) => println!("Internet Radio Station Owner: {}", x),
                     id3::v24::OwnedFrame::TSOA(x) => println!("Album for sorting: {}", x),
                     id3::v24::OwnedFrame::TSOP(x) => println!("Artist name for sorting: {}", x),
                     id3::v24::OwnedFrame::TSOT(x) => println!("Title for sorting: {}", x),
                     id3::v24::OwnedFrame::TSRC(x) => println!("ISRC: {}", x),
                     id3::v24::OwnedFrame::TSSE(x) => println!("Encoding settings: {:?}", x),
                     id3::v24::OwnedFrame::TSST(x) => println!("Set Subtitle: {:?}", x),
                     id3::v24::OwnedFrame::TXXX(x) => println!("User defined text: {:?}", x),
                     id3::v24::OwnedFrame::USLT(x) => println!("Lyrics: {:?}", x),
                     id3::v24::OwnedFrame::WCOM(x) => println!("Commercial Information URL: {}", x),
                     id3::v24::OwnedFrame::WCOP(x) => println!("Copyright/Legal Info URL: {}", x),
                     id3::v24::OwnedFrame::WOAF(x) => println!("Audio File URL: {}", x),
                     id3::v24::OwnedFrame::WOAR(x) => println!("Artist/Performer URL: {}", x),
                     id3::v24::OwnedFrame::WOAS(x) => println!("Audio Source URL: {}", x),
                     id3::v24::OwnedFrame::WORS(x) => println!("Internet Radio Station URL: {}", x),
                     id3::v24::OwnedFrame::WPAY(x) => println!("Payment URL: {}", x),
                     id3::v24::OwnedFrame::WPUB(x) => println!("Publisher URL: {}", x),
                     id3::v24::OwnedFrame::Unknown(u) => {
                        println!("Unknown frame: {}", String::from_utf8_lossy(&u.name))
                     }
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
