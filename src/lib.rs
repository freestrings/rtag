//!
//! # Usage
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
//! # Example: reading and filtering
//!
//! When you read a frame information, you use [Unit](enum.Unit).
//!
//! # Example: find V1 frame information.
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
//! # Example: find V2 frame information.
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
//! # Example: modify tag
//! 
//!```rust
//! use rtag::frame::*;
//! use rtag::metadata::Unit;
//! use rtag::metadata::MetadataReader;
//! use rtag::metadata::MetadataWriter;
//!
//! let path = "./test-resources/240.mp3";
//! let new_data = MetadataReader::new(path)
//!         .unwrap()
//!         .fold(Vec::new(), |mut vec, unit| {
//!             if let Unit::FrameV2(frame_head, frame_data) = unit {
//!                 let new_frame_data = if let FrameBody::TALB(ref frame) = frame_data {
//!                     let mut new_frame = frame.clone();
//!                     new_frame.text = "Album!".to_string();
//!                     FrameBody::TALB(new_frame)
//!                 } else {
//!                     frame_data.clone()
//!                 };
//! 
//!                 vec.push(Unit::FrameV2(frame_head, new_frame_data));
//!             } else {
//!                 vec.push(unit);
//!             }
//! 
//!             vec
//!         });
//! 
//!     let writer = MetadataWriter::new(path).unwrap();
//!     let _ = writer.write(new_data);
//!```
#[macro_use]
extern crate log;

pub mod errors;
pub mod frame;
pub mod metadata;
pub mod readable;
pub mod writable;
mod util;
