#[macro_use] extern crate log;

use std::{fs, vec, cmp, io};
use std::io::{Read};

/// 0: index
/// 1: length
struct Offset(usize, usize, &'static str);

trait NameOf {
    fn name_of(&self) -> String;
}

trait ReadFromFile {
    fn read_from_file(&self, buf: &mut fs::File) -> Option<vec::Vec<u8>>;
}

trait ReadFromBytes {
    fn read_from_bytes(&self, bytes: &vec::Vec<u8>) -> Option<vec::Vec<u8>>;
}

impl NameOf for Offset {
    fn name_of(&self) -> String {
        self.2.to_string()
    }
}

impl ReadFromFile for Offset {
    fn read_from_file(&self, file: &mut fs::File) -> Option<vec::Vec<u8>> {
        let mut buf = vec![0u8; self.1];
        if file.read(buf.as_mut_slice()).is_ok() {
            Some(buf)
        } else {
            None
        }
    }
}

impl ReadFromBytes for Offset {
    fn read_from_bytes(&self, bytes: &vec::Vec<u8>) -> Option<vec::Vec<u8>> {
        let mut buf = vec![0u8; self.1];
        let len = cmp::min(self.1, bytes.len());
        for i in self.0..(len + self.0) {
            buf[i - self.0] = bytes[i];
        }
        Some(buf)
    }
}

const TITLE_OFFSET: Offset = Offset(3, 30, "title");
const ARTIST_OFFSET: Offset = Offset(33, 30, "artist");
const ALBUM_OFFSET: Offset = Offset(63, 30, "album");
const YEAR_OFFSET: Offset = Offset(93, 4, "year");
const COMMENT1_OFFSET: Offset = Offset(97, 30, "comment1");
const COMMENT2_OFFSET: Offset = Offset(97, 28, "comment2");
const TRACK_MARKER_OFFSET: Offset = Offset(125, 1, "track_marker");
const TRACK_OFFSET: Offset = Offset(126, 1, "track");
const GENRE_OFFSET: Offset = Offset(127, 1, "genre");

const ID3V1_TAG_LENGTH: usize = 128;
const TAG: &'static str = "TAG";

struct ID3v1 {
    title: String,
    artist: String,
    album: String,
    year: String,
    comment: String,
    track: String,
    genre: String
}

impl ID3v1 {
    fn new(file_path: &str) -> Option<ID3v1> {
        fn read<'a, F>(offset: &Offset, bytes: &vec::Vec<u8>, trans: &F) -> String where F: Fn(&vec::Vec<u8>) -> String {
            match offset.read_from_bytes(bytes) {
                Some(ref read) => trans(read),
                None => {
                    debug!("read offset fail: {}", offset.name_of());
                    String::default()
                }
            }
        }

        fn read_safe(offset: Offset, bytes: &vec::Vec<u8>) -> String {
            read(&offset, bytes, &|b| {
                trace!("{}: {:?}", offset.name_of(), b);
                let cloned = trim_non_ascii(b);
                let value = String::from_utf8_lossy(&cloned).into_owned();
                debug!("{}: {}", offset.name_of(), value);
                value
            })
        }

        fn trim_non_ascii(bytes: &vec::Vec<u8>) -> vec::Vec<u8> {
            debug!("origin bytes: {:?}, len: {}", bytes, bytes.len());
            let mut idx = 0;
            for v in bytes.iter().rev() {
                if v > &32 { break; }
                idx = idx + 1;
            }
            debug!("found ascii index: {}", bytes.len() - idx);

            let mut clone = bytes.clone();
            clone.split_off(bytes.len() - idx);
            clone
        }

        fn has_track_marker(bytes: &vec::Vec<u8>) -> bool {
            read_safe(TRACK_MARKER_OFFSET, &bytes) != "0".to_string()
        }

        if let Ok(bytes) = Self::validation(file_path) {
            return Some(ID3v1 {
                title: read_safe(TITLE_OFFSET, &bytes),
                artist: read_safe(ARTIST_OFFSET, &bytes),
                album: read_safe(ALBUM_OFFSET, &bytes),
                year: read_safe(YEAR_OFFSET, &bytes),
                track: read(&TRACK_OFFSET, &bytes, &|b| {
                    if !has_track_marker(&bytes) {
                        String::new()
                    } else {
                        trace!("{}: {:?}", &TRACK_OFFSET.name_of(), b);
                        b[0].to_string()
                    }
                }),
                genre: read(&GENRE_OFFSET, &bytes, &|b| {
                    // TODO mapping
                    // https://de.wikipedia.org/wiki/Liste_der_ID3v1-Genres
                    trace!("{}: {:?}", GENRE_OFFSET.name_of(), b);
                    (b[0] & 0xFF).to_string()
                }),
                comment: {
                    if has_track_marker(&bytes) {
                        read_safe(COMMENT1_OFFSET, &bytes)
                    } else {
                        read_safe(COMMENT2_OFFSET, &bytes)
                    }
                }
            });
        }

        None
    }

    fn title(&self) -> &str {
        self.title.as_ref()
    }

    fn artist(&self) -> &str {
        self.artist.as_ref()
    }

    fn album(&self) -> &str {
        self.album.as_ref()
    }

    fn year(&self) -> &str {
        self.year.as_ref()
    }

    fn comment(&self) -> &str {
        self.comment.as_ref()
    }

    fn track(&self) -> &str {
        self.track.as_ref()
    }

    fn genre(&self) -> &str {
        self.genre.as_ref()
    }
}

type VaidationResult = io::Result<vec::Vec<u8>>;

trait ID3v1Tag {
    fn validation(file_path: &str) -> VaidationResult;
}

impl ID3v1Tag for ID3v1 {
    fn validation(file_path: &str) -> VaidationResult {
        let file_meta = try!(fs::metadata(file_path));

        if (ID3V1_TAG_LENGTH as u64) > file_meta.len() {
            return Err(io::Error::new(io::ErrorKind::Other, "Not found `ID3v1`"))
        }

        let mut file = try!(fs::File::open(file_path));

        let file_len = file_meta.len() as usize;
        let offset = file_len - ID3V1_TAG_LENGTH;

        if let Some(total_bytes) = Offset(0, file_len, "total").read_from_file(&mut file) {
            let tag_offset = Offset(offset, TAG.len(), "tag");

            let is_valid = match tag_offset.read_from_bytes(&total_bytes) {
                Some(tag_bytes) => String::from_utf8(tag_bytes).unwrap() == TAG,
                None => false
            };

            if is_valid == true {
                let id3v1_offset = Offset(offset, ID3V1_TAG_LENGTH, "id3v1");
                if let Some(bytes) = id3v1_offset.read_from_bytes(&total_bytes) {
                    return Ok(bytes);
                }
            }
        }

        Err(io::Error::new(io::ErrorKind::Other, "Invalid `ID3v1`"))
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    extern crate env_logger;

    use std::fs;
    use std::io::{Read, SeekFrom, Seek};
    use super::ID3v1Tag;

    #[test]
    fn test_validation1() {
        let _ = env_logger::init();

        let file_path = "./resources/ID3v1-ID3v2.mp3";
        let mut assert = false;
        if let Ok(bytes) = super::ID3v1::validation(file_path) {
            if let Ok(ref mut file) = fs::File::open(file_path) {
                let mut tag_buf = vec![0u8; super::ID3V1_TAG_LENGTH];
                if file.seek(SeekFrom::Start(575102 - super::ID3V1_TAG_LENGTH as u64)).is_ok() {
                    if let Ok(read) = file.read(tag_buf.as_mut_slice()) {
                        assert = read == super::ID3V1_TAG_LENGTH;
                        assert = assert && bytes == tag_buf;
                    }
                }
            }
        }

        assert!(assert);
    }

    #[test]
    fn test_validation2() {
        let _ = env_logger::init();

        let file_path = "./resources/empty-meta.mp3";
        match super::ID3v1::validation(file_path) {
            Ok(_) => assert!(false),
            Err(_) => assert!(true)
        }
    }

    #[test]
    fn test_tags() {
        let _ = env_logger::init();

        let file_path = "./resources/ID3v1-ID3v2.mp3";
        match super::ID3v1::new(file_path) {
            Some(id3v1) => {
                assert_eq!(id3v1.artist(), "Artist");
                assert_eq!(id3v1.album(), "");
                assert_eq!(id3v1.comment(), "!@#$");
                assert_eq!(id3v1.track(), "1");
                assert_eq!(id3v1.genre(), "137");
            },
            None => assert!(false)
        }
    }
}