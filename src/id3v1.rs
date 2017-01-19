//MIT License
//
//Copyright (c) [2017] [Mark Han]
//
//Permission is hereby granted, free of charge, to any person obtaining a copy
//of this software and associated documentation files (the "Software"), to deal
//in the Software without restriction, including without limitation the rights
//to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//copies of the Software, and to permit persons to whom the Software is
//furnished to do so, subject to the following conditions:
//
//The above copyright notice and this permission notice shall be included in all
//copies or substantial portions of the Software.
//
//THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//SOFTWARE.

extern crate encoding;

use std::{io, result, vec};
use self::encoding::{Encoding, DecoderTrap};

pub struct ID3v1Tag {
    title: String,
    artist: String,
    album: String,
    year: String,
    comment: String,
    track: String,
    genre: String
}

// @see http://id3.org/ID3v1
impl ID3v1Tag {
    pub fn new<T: io::Read + io::Seek>(readable: &mut ::readable::Readable<T>, file_len: u64)
                                       -> result::Result<Self, ::errors::ParsingError> {
        // id3v1 tag length is 128 bytes.
        if file_len < 128 as u64 {
            return Err(::errors::ParsingError::BadData(format!("Bad tag length: {}", file_len)));
        }
        // tag position is last 128 bytes.
        readable.skip((file_len - 128 as u64) as i64)?;

        let tad_id = readable.as_string(3)?;
        if tad_id != "TAG" {
            return Err(::errors::ParsingError::Id1TagNotFound);
        }

        // offset 3
        let title = &readable.as_bytes(30)?;
        trace!("title: {:?}", title);
        let title = Self::_to_string_with_rtrim(&title);

        // offset 33
        let artist = &readable.as_bytes(30)?;
        trace!("artist: {:?}", artist);
        let artist = Self::_to_string_with_rtrim(&artist);

        // offset 63
        let album = readable.as_bytes(30)?;
        trace!("album: {:?}", album);
        let album = Self::_to_string_with_rtrim(&album);

        // offset 93
        let year = readable.as_bytes(4)?;
        trace!("year: {:?}", year);
        let year = Self::_to_string_with_rtrim(&year);

        // goto track marker offset
        readable.skip(28)?;

        // offset 125
        let track_marker = readable.as_bytes(1)?[0];
        // offset 126
        let _track = readable.as_bytes(1)?[0] & 0xff;
        // offset 127
        let genre = (readable.as_bytes(1)?[0] & 0xff).to_string();
        // goto comment offset
        readable.skip(-31)?;

        let (comment, track) = if track_marker != 0 {
            (
                Self::_to_string_with_rtrim(&readable.as_bytes(30)?),
                String::new()
            )
        } else {
            (
                Self::_to_string_with_rtrim(&readable.as_bytes(28)?),
                if _track == 0 { String::new() } else { _track.to_string() }
            )
        };

        Ok(ID3v1Tag {
            title: title,
            artist: artist,
            album: album,
            year: year,
            comment: comment,
            track: track,
            genre: genre
        })
    }

    fn _rtrim(bytes: &vec::Vec<u8>) -> vec::Vec<u8> {
        let mut idx = 0;
        for v in bytes.iter().rev() {
            if v > &32 { break; }
            idx = idx + 1;
        }
        let mut clone = bytes.clone();
        clone.split_off(bytes.len() - idx);
        clone
    }

    // default encoding is ISO-8859-1
    fn _to_string_with_rtrim(bytes: &vec::Vec<u8>) -> String {
        let cloned = Self::_rtrim(bytes);
        match encoding::all::ISO_8859_1.decode(&cloned, encoding::DecoderTrap::Strict) {
            Ok(text) => text,
            Err(_) => "".to_string()
        }
    }

    pub fn get_title(&self) -> &str {
        self.title.as_ref()
    }

    pub fn get_artist(&self) -> &str {
        self.artist.as_ref()
    }

    pub fn get_album(&self) -> &str {
        self.album.as_ref()
    }

    pub fn get_year(&self) -> &str {
        self.year.as_ref()
    }

    pub fn get_comment(&self) -> &str {
        self.comment.as_ref()
    }

    pub fn get_track(&self) -> &str {
        self.track.as_ref()
    }

    pub fn get_genre(&self) -> &str {
        self.genre.as_ref()
    }
}

#[cfg(test)]
mod tests {
    extern crate env_logger;
    extern crate encoding;

    use std::{io, result, vec};
    use self::encoding::{Encoding, DecoderTrap};
    use std::fs;

    #[test]
    fn v1_bad_length() {
        let _ = env_logger::init();

        let id3v1_tag = "1234567890abcdefghij";
        let mut readable = ::readable::factory::from_str(id3v1_tag).unwrap();
        match super::ID3v1Tag::new(&mut readable, id3v1_tag.len() as u64) {
            Err(::errors::ParsingError::BadData(msg)) => assert_eq!("Bad tag length: 20", msg),
            _ => assert!(false)
        }
    }

    #[test]
    fn v1_invalid_id3_tag() {
        let _ = env_logger::init();

        let file = fs::File::open("./test-resources/230-no-id3.mp3").unwrap();
        let len = file.metadata().unwrap().len();
        let mut readable = ::readable::Readable::new(file);
        match super::ID3v1Tag::new(&mut readable, len) {
            Err(::errors::ParsingError::Id1TagNotFound) => assert!(true),
            _ => assert!(false)
        }
    }

    #[test]
    fn v1_empty() {
        let _ = env_logger::init();

        let file = fs::File::open("./test-resources/empty-meta.mp3").unwrap();
        let len = file.metadata().unwrap().len();
        let mut readable = ::readable::Readable::new(file);
        assert!(super::ID3v1Tag::new(&mut readable, len).is_err());
    }

    #[test]
    fn v1_test1() {
        let _ = env_logger::init();

        let file = fs::File::open("./test-resources/v1-v2.mp3").unwrap();
        let len = file.metadata().unwrap().len();
        let mut readable = ::readable::Readable::new(file);
        let id3v1 = super::ID3v1Tag::new(&mut readable, len).unwrap();
        assert_eq!(id3v1.get_artist(), "Artist");
        assert_eq!(id3v1.get_album(), "");
        assert_eq!(id3v1.get_comment(), "!@#$");
        assert_eq!(id3v1.get_track(), "1");
        assert_eq!(id3v1.get_genre(), "137");
    }

    #[test]
    fn v1_test2() {
        let _ = env_logger::init();

        let id3v1_tag = "TAGTITLETITLETITLETITLETITLETITLEARTISTARTISTARTISTARTISTARTISTALBUMALBUMALBUMALBUMALBUMALBUM2017COMMENTCOMMENTCOMMENTCOMMENTCO4";

        let mut readable = ::readable::factory::from_str(id3v1_tag).unwrap();
        let id3v1 = super::ID3v1Tag::new(&mut readable, id3v1_tag.len() as u64).unwrap();
        assert_eq!(id3v1.get_title(), "TITLETITLETITLETITLETITLETITLE");
        assert_eq!(id3v1.get_artist(), "ARTISTARTISTARTISTARTISTARTIST");
        assert_eq!(id3v1.get_album(), "ALBUMALBUMALBUMALBUMALBUMALBUM");
        assert_eq!(id3v1.get_comment(), "COMMENTCOMMENTCOMMENTCOMMENTCO");
        assert_eq!(id3v1.get_year(), "2017");
    }

    #[test]
    fn v1_test3() {
        let _ = env_logger::init();

        let id3v1_tag = "TAGTITLE                         ARTIST                        ALBUM                         2017COMMENT                        ";

        let mut readable = ::readable::factory::from_str(id3v1_tag).unwrap();
        let id3v1 = super::ID3v1Tag::new(&mut readable, id3v1_tag.len() as u64).unwrap();
        assert_eq!(id3v1.get_title(), "TITLE");
        assert_eq!(id3v1.get_artist(), "ARTIST");
        assert_eq!(id3v1.get_album(), "ALBUM");
        assert_eq!(id3v1.get_comment(), "COMMENT");
        assert_eq!(id3v1.get_year(), "2017");
    }

    #[test]
    fn v1_iso_8859_1() {
        let _ = env_logger::init();

        let file = fs::File::open("./test-resources/v1-iso-8859-1.mp3").unwrap();
        let len = file.metadata().unwrap().len();
        let mut readable = ::readable::Readable::new(file);
        let id3v1 = super::ID3v1Tag::new(&mut readable, len).unwrap();

        assert_eq!("räksmörgås", id3v1.get_title());
        assert_eq!("räksmörgås", id3v1.get_artist());
        assert_eq!("räksmörgås", id3v1.get_album());
        assert_eq!("räksmörgås", id3v1.get_comment());
    }

    #[test]
    fn v1_utf8() {
        let _ = env_logger::init();

        let file = fs::File::open("./test-resources/v1-utf8.mp3").unwrap();
        let len = file.metadata().unwrap().len();
        let mut readable = ::readable::Readable::new(file);
        let id3v1 = super::ID3v1Tag::new(&mut readable, len).unwrap();

        assert_eq!("rÃ¤ksmÃ¶rgÃ¥s", id3v1.get_title());
        assert_eq!("rÃ¤ksmÃ¶rgÃ¥s", id3v1.get_artist());
        assert_eq!("rÃ¤ksmÃ¶rgÃ¥s", id3v1.get_album());
        assert_eq!("rÃ¤ksmÃ¶rgÃ¥s", id3v1.get_comment());
    }
}