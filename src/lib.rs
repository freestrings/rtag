//!
//!# Basic usage
//!
//! This can be used by adding `rtag` to your dependencies in your project's `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! rtag = "0.2"
//! ```
//!
//! and this to your crate root:
//!
//! ```rust
//! extern crate rtag;
//! ```
//!
//!# Example: reading and filtering
//!
//! When you read a frame information, you use `[Unit](metadata/enum.Unit.html)`.
//!
//!# Example: reading V1 frame.
//!
//! ```rust
//! use rtag::metadata::Unit;
//! use rtag::metadata::MetadataReader;
//!
//! for m in MetadataReader::new("./test-resources/v1-v2.mp3").unwrap() {
//!     match m {
//!         Unit::FrameV1(frame) => {
//!             assert_eq!("Artist", frame.artist);
//!             assert_eq!("!@#$", frame.comment);
//!             assert_eq!("1", frame.track);
//!             assert_eq!("137", frame.genre);
//!         },
//!         _ => ()
//!     }
//! }
//! ```
//!
//!# Example: reading V2 frame.
//!
//!```rust
//! use rtag::frame::*;
//! use rtag::metadata::Unit;
//! use rtag::metadata::MetadataReader;
//!
//! for m in MetadataReader::new("./test-resources/240-pcnt.mp3").unwrap() {
//!     match m {
//!         Unit::FrameV2(head, FrameBody::PCNT(frame)) => {
//!             assert_eq!(256, frame.counter);
//!         },
//!         _ => ()
//!     }
//! }
//!
//!```
//!
//!# Example: modifying a frame.
//!
//!```rust
//! use std::fs;
//! use rtag::frame::*;
//! use rtag::metadata::Unit;
//! use rtag::metadata::MetadataReader;
//! use rtag::metadata::MetadataWriter;
//!
//! let path = "./test-resources/240.test.mp3";
//! fs::copy("./test-resources/240.mp3", path).unwrap();
//!
//! let new_data = MetadataReader::new(path)
//!     .unwrap()
//!     .fold(Vec::new(), |mut vec, unit| {
//!         if let Unit::FrameV2(frame_head, frame_body) = unit {
//!             let new_frame_body = if let FrameBody::TALB(ref frame) = frame_body {
//!                 let mut new_frame = frame.clone();
//!                 new_frame.text = "Album!".to_string();
//!                 FrameBody::TALB(new_frame)
//!             } else {
//!                 frame_body.clone()
//!             };
//!
//!             vec.push(Unit::FrameV2(frame_head, new_frame_body));
//!         } else {
//!             vec.push(unit);
//!         }
//!
//!         vec
//!     });
//!
//! let writer = MetadataWriter::new(path).unwrap();
//! let _ = writer.write(new_data, false);
//! let _ = fs::remove_file(path).unwrap();
//!```
//!
//!# Example: rewriting all the frame to version 4.
//!
//!```rust
//! use std::fs;
//! use rtag::frame::*;
//! use rtag::metadata::Unit;
//! use rtag::metadata::MetadataReader;
//! use rtag::metadata::MetadataWriter;
//!
//! let path = "./test-resources/v2.2.test.mp3";
//! fs::copy("./test-resources/v2.2.mp3", path).unwrap();
//! 
//! let frames2_2 = MetadataReader::new(path).unwrap().collect::<Vec<Unit>>();
//! let _ = MetadataWriter::new(path).unwrap().write(frames2_2, true);
//! let i = MetadataReader::new(path)
//!     .unwrap()
//!     .filter(|unit| match unit {
//!         &Unit::FrameV2(FrameHeader::V22(_), _) => true,
//!         _ => false,
//!     });
//! 
//! assert_eq!(i.count(), 0);
//! 
//! let i = MetadataReader::new(path)
//!     .unwrap()
//!     .filter(|unit| match unit {
//!         &Unit::FrameV2(FrameHeader::V24(_), _) => true,
//!         _ => false,
//!     });
//! 
//! assert_eq!(i.count(), 5);
//! let _ = fs::remove_file(path).unwrap();
//!```
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

pub mod errors;
pub mod frame;
pub mod metadata;
pub mod readable;
pub mod writable;
mod util;
