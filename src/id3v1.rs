use scanner;
use std::{io, vec};

const TITLE_LEN: usize = 30;
const ARTIST_LEN: usize = 30;
const ALBUM_LEN: usize = 30;
const YEAR_LEN: usize = 4;
const COMMENT1_LEN: usize = 30;
const COMMENT2_LEN: usize = 28;
const TRACK_MARKER_LEN: usize = 1;
const TRACK_LEN: usize = 1;
const GENRE_LEN: usize = 1;
const ID3V1_TAG_LENGTH: u8 = 128;
const TAG: &'static str = "TAG";

pub struct ID3v1Tag {
    title: String,
    artist: String,
    album: String,
    year: String,
    comment: String,
    track: String,
    genre: String
}

impl ID3v1Tag {
    fn trim_non_ascii(bytes: &vec::Vec<u8>) -> vec::Vec<u8> {
        let mut idx = 0;
        for v in bytes.iter().rev() {
            if v > &32 { break; }
            idx = idx + 1;
        }
        let mut clone = bytes.clone();
        clone.split_off(bytes.len() - idx);
        clone
    }

    fn trimed_string(bytes: &vec::Vec<u8>) -> String {
        let cloned = Self::trim_non_ascii(bytes);
        let value = String::from_utf8_lossy(&cloned).into_owned();
        value
    }

    pub fn new(scanner: &mut scanner::Scanner) -> io::Result<Self> {
        if scanner.get_len() < ID3V1_TAG_LENGTH as u64 {
            return Err(io::Error::new(io::ErrorKind::Other, "Not found `ID3v1` tag"));
        }

        let v1_offset = scanner.get_len() - ID3V1_TAG_LENGTH as u64;
        scanner.skip(v1_offset);

        if let Ok(str) = scanner.read_as_string(TAG.len()) {
            if str != TAG {
                return Err(io::Error::new(io::ErrorKind::Other, "Invalid `ID3v1` tag"));
            }
        }

        let title = Self::trimed_string(&try!(scanner.read_as_bytes(TITLE_LEN)));
        let artist = Self::trimed_string(&try!(scanner.read_as_bytes(ARTIST_LEN)));
        let album = Self::trimed_string(&try!(scanner.read_as_bytes(ALBUM_LEN)));
        let year = Self::trimed_string(&try!(scanner.read_as_bytes(YEAR_LEN)));
        let comment1 = try!(scanner.read_as_bytes(COMMENT1_LEN));
        scanner.rewind(30);
        let comment2 = try!(scanner.read_as_bytes(COMMENT2_LEN));
        let track_marker = try!(scanner.read_as_string(TRACK_MARKER_LEN));
        let mut track = String::new();
        if track_marker != "0" {
            track = (try!(scanner.read_as_bytes(TRACK_LEN))[0] & 0xff).to_string();
        }
        let genre = (try!(scanner.read_as_bytes(GENRE_LEN))[0] & 0xff).to_string();
        let comment = if track_marker == "0" {
            Self::trimed_string(&comment1)
        } else {
            Self::trimed_string(&comment2)
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
    use scanner;

    #[test]
    fn id3v1_test1() {
        match scanner::Scanner::new("./resources/empty-meta.mp3") {
            Ok(mut scanner) => {
                match super::ID3v1Tag::new(&mut scanner) {
                    Ok(_) => assert!(false),
                    Err(_) => assert!(true)
                }
            },
            Err(_) => assert!(false)
        }
    }

    #[test]
    fn id3v1_test2() {
        match scanner::Scanner::new("./resources/ID3v1-ID3v2.mp3") {
            Ok(mut scanner) => {
                match super::ID3v1Tag::new(&mut scanner) {
                    Ok(id3v1) => {
                        assert_eq!(id3v1.artist(), "Artist");
                        assert_eq!(id3v1.album(), "");
                        assert_eq!(id3v1.comment(), "!@#$");
                        assert_eq!(id3v1.track(), "1");
                        assert_eq!(id3v1.genre(), "137");
                    },
                    Err(_) => assert!(true)
                }
            },
            Err(_) => assert!(false)
        }
    }
}