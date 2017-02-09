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
//! ```rust
//! use rtag::frame::*;
//! use rtag::metadata::Unit;
//! use rtag::metadata::MetadataReader;
//!
//! for m in MetadataReader::new("./test-resources/240-pcnt.mp3").unwrap() {
//!     match m {
//!         Unit::FrameV2(head, FrameData::PCNT(frame)) => {
//!             assert_eq!(256, frame.counter);
//!         },
//!         _ => ()
//!     }
//! }
//!```
//!
#[macro_use]
extern crate log;

pub mod errors;
pub mod frame;
pub mod metadata;
pub mod readable;
pub mod writable;
mod util;
