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

use readable;
use std::{io, vec};
use std::io::Result;

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
    pub fn new<T: io::Read + io::Seek>(readable: &mut readable::Readable<T>, file_len: u64) -> Result<Self> {
        // id3v1 tag length is 128 bytes.
        if file_len < 128 as u64 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Error1"));
        }
        // tag position is last 128 bytes.
        readable.skip((file_len - 128 as u64) as i64)?;

        if readable.as_string(3)? != "TAG" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Error2"));
        }

        // offset 3
        let title = Self::_to_string_with_rtrim(&readable.as_bytes(30)?);
        // offset 33
        let artist = Self::_to_string_with_rtrim(&readable.as_bytes(30)?);
        // offset 63
        let album = Self::_to_string_with_rtrim(&readable.as_bytes(30)?);
        // offset 93
        let year = Self::_to_string_with_rtrim(&readable.as_bytes(4)?);
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

    fn _to_string_with_rtrim(bytes: &vec::Vec<u8>) -> String {
        let cloned = Self::_rtrim(bytes);
        let value = String::from_utf8_lossy(&cloned).into_owned();
        value
    }

    pub fn title(&self) -> &str {
        self.title.as_ref()
    }

    pub fn artist(&self) -> &str {
        self.artist.as_ref()
    }

    pub fn album(&self) -> &str {
        self.album.as_ref()
    }

    pub fn year(&self) -> &str {
        self.year.as_ref()
    }

    pub fn comment(&self) -> &str {
        self.comment.as_ref()
    }

    pub fn track(&self) -> &str {
        self.track.as_ref()
    }

    pub fn genre(&self) -> &str {
        self.genre.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use readable;
    use std::fs;

    #[test]
    fn id3v1_test1() {
        let file = fs::File::open("./resources/file1.txt").unwrap();
        let len = file.metadata().unwrap().len();
        let mut readable = readable::Readable::new(file);
        if let Err(msg) = super::ID3v1Tag::new(&mut readable, len) {
            assert_eq!(msg.to_string(), "Error1");
        }
    }

    #[test]
    fn id3v1_test2() {
        let file = fs::File::open("./resources/empty-meta.mp3").unwrap();
        let len = file.metadata().unwrap().len();
        let mut readable = readable::Readable::new(file);
        assert!(super::ID3v1Tag::new(&mut readable, len).is_err());
    }

    #[test]
    fn id3v1_test3() {
        let file = fs::File::open("./resources/id3v1-id3v2.mp3").unwrap();
        let len = file.metadata().unwrap().len();
        let mut readable = readable::Readable::new(file);
        let id3v1 = super::ID3v1Tag::new(&mut readable, len).unwrap();
        assert_eq!(id3v1.artist(), "Artist");
        assert_eq!(id3v1.album(), "");
        assert_eq!(id3v1.comment(), "!@#$");
        assert_eq!(id3v1.track(), "1");
        assert_eq!(id3v1.genre(), "137");
    }

    #[test]
    fn id3v1_test4() {
        let id3v1_tag = "TAGTITLETITLETITLETITLETITLETITLEARTISTARTISTARTISTARTISTARTISTALBUMALBUMALBUMALBUMALBUMALBUM2017COMMENTCOMMENTCOMMENTCOMMENTCO4";

        let mut readable = readable::factory::from_str(id3v1_tag).unwrap();
        let id3v1 = super::ID3v1Tag::new(&mut readable, id3v1_tag.len() as u64).unwrap();
        assert_eq!(id3v1.title(), "TITLETITLETITLETITLETITLETITLE");
        assert_eq!(id3v1.artist(), "ARTISTARTISTARTISTARTISTARTIST");
        assert_eq!(id3v1.album(), "ALBUMALBUMALBUMALBUMALBUMALBUM");
        assert_eq!(id3v1.comment(), "COMMENTCOMMENTCOMMENTCOMMENTCO");
        assert_eq!(id3v1.year(), "2017");
    }

    #[test]
    fn id3v1_test5() {
        let id3v1_tag = "TAGTITLE                         ARTIST                        ALBUM                         2017COMMENT                        ";

        let mut readable = readable::factory::from_str(id3v1_tag).unwrap();
        let id3v1 = super::ID3v1Tag::new(&mut readable, id3v1_tag.len() as u64).unwrap();
        assert_eq!(id3v1.title(), "TITLE");
        assert_eq!(id3v1.artist(), "ARTIST");
        assert_eq!(id3v1.album(), "ALBUM");
        assert_eq!(id3v1.comment(), "COMMENT");
        assert_eq!(id3v1.year(), "2017");
    }
}