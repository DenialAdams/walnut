#![feature(try_blocks, try_from)]

extern crate byteorder;
#[macro_use]
extern crate log;
extern crate walkdir;
#[macro_use]
extern crate bitflags;
extern crate pretty_env_logger;

mod id3;

use std::fs::File;
use walkdir::WalkDir;

fn main() {
   pretty_env_logger::init();

   for entry in WalkDir::new("C:\\music") {
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

      println!("{}", entry.path().display());

      let mut f = File::open(entry.path()).unwrap();
      match id3::parse_source(&mut f) {
         Ok(parser) => {
            println!("ID3v24");
            for frame in parser {
               match frame {
                  Err(e) => println!(
                     "Failed to parse frame {}: {:?}",
                     String::from_utf8_lossy(&e.name),
                     e.reason
                  ),
                  Ok(frame) => match frame {
                     id3::v24::OwnedFrame::COMM(x) => println!("Comment: {:?}", x),
                     id3::v24::OwnedFrame::PRIV(x) => println!("Private: {:?}", x),
                     id3::v24::OwnedFrame::TALB(x) => println!("Album: {}", x),
                     id3::v24::OwnedFrame::TCOM(x) => println!("Composer: {}", x),
                     id3::v24::OwnedFrame::TCON(x) => println!("Genre: {}", x),
                     id3::v24::OwnedFrame::TDRC(x) => println!("Recording Date: {:?}", x),
                     id3::v24::OwnedFrame::TDRL(x) => println!("Release Date: {:?}", x),
                     id3::v24::OwnedFrame::TENC(x) => println!("Encoded by: {}", x),
                     id3::v24::OwnedFrame::TIT2(x) => println!("Title: {}", x),
                     id3::v24::OwnedFrame::TLEN(x) => println!("Length: {}ms", x),
                     id3::v24::OwnedFrame::TPE1(x) => println!("Artist: {}", x),
                     id3::v24::OwnedFrame::TPE2(x) => println!("Album Artist: {}", x),
                     id3::v24::OwnedFrame::TPE3(x) => println!("Conductor: {}", x),
                     id3::v24::OwnedFrame::TPOS(x) => println!("CD: {:?}", x),
                     id3::v24::OwnedFrame::TPUB(x) => println!("Publisher: {:?}", x),
                     id3::v24::OwnedFrame::TRCK(x) => println!("Track: {:?}", x),
                     id3::v24::OwnedFrame::TSOP(x) => println!("Artist name for sorting: {}", x),
                     id3::v24::OwnedFrame::TSSE(x) => println!("Encoding settings: {:?}", x),
                     id3::v24::OwnedFrame::Unknown(u) => {
                        println!("Unknown frame: {}", String::from_utf8_lossy(&u.name))
                     }
                  },
               }
            }
         }
         Err(e) => match e {
            id3::TagParseError::NoTag => {
               println!("No ID3");
            }
            id3::TagParseError::UnsupportedVersion(ver) => {
               println!("ID3v2{}", ver);
            }
            id3::TagParseError::Io(io_err) => {
               warn!("Failed to parse file: {}", io_err);
            }
         },
      }
   }
}
