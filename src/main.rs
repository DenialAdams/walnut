#![feature(nll)]

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
               match frame.unwrap() {
                  id3::Frame::TALB(x) => println!("Album: {}", x),
                  id3::Frame::TCON(x) => println!("Genre: {}", x),
                  id3::Frame::TPE1(x) => println!("Artist: {}", x),
                  id3::Frame::TPE2(x) => println!("ft: {}", x),
                  id3::Frame::Unknown(u) => println!("Unknown frame: {}", String::from_utf8_lossy(&u.name)),
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
