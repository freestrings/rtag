extern crate libc;

use std::env;
use std::ptr;
use std::ffi::*;
use libc::*;

//------------------------------------------------------------------------------------------------//
// start extern
//------------------------------------------------------------------------------------------------//

struct ID3v2_frame_text_content {
    size: c_int,
    encoding: c_char,
    data: *const c_char,
}

struct ID3v2_tag;

struct ID3v2_frame;

#[link(name = "id3v2")]
extern {
    // frame
    fn parse_text_frame_content(ptr: *mut ID3v2_frame) -> *mut ID3v2_frame_text_content;

    // id3v2lib
    fn load_tag(file_name: *const c_char) -> *mut ID3v2_tag;
    fn load_tag_with_buffer(buffer: *const c_char, length: c_int) -> *mut ID3v2_tag;
    fn remove_tag(file_name: *const c_char);
    fn set_tag(file_name: *const c_char, tag: *mut ID3v2_tag);

    fn tag_get_title(ptr: *mut ID3v2_tag) -> *mut ID3v2_frame;
    fn tag_get_artist(ptr: *mut ID3v2_tag) -> *mut ID3v2_frame;
    fn tag_get_album(ptr: *mut ID3v2_tag) -> *mut ID3v2_frame;
    fn tag_get_album_artist(ptr: *mut ID3v2_tag) -> *mut ID3v2_frame;
    fn tag_get_genre(ptr: *mut ID3v2_tag) -> *mut ID3v2_frame;
    fn tag_get_track(ptr: *mut ID3v2_tag) -> *mut ID3v2_frame;
    fn tag_get_year(ptr: *mut ID3v2_tag) -> *mut ID3v2_frame;
    fn tag_get_comment(ptr: *mut ID3v2_tag) -> *mut ID3v2_frame;
    fn tag_get_disc_number(ptr: *mut ID3v2_tag) -> *mut ID3v2_frame;
    fn tag_get_composer(ptr: *mut ID3v2_tag) -> *mut ID3v2_frame;
    fn tag_get_album_cover(ptr: *mut ID3v2_tag) -> *mut ID3v2_frame;

    fn tag_set_title(title: *const c_char, encoding: *const c_char, tag: *mut ID3v2_tag);
    fn tag_set_artist(artist: *const c_char, encoding: *const c_char, tag: *mut ID3v2_tag);
    fn tag_set_album(album: *const c_char, encoding: *const c_char, tag: *mut ID3v2_tag);
    fn tag_set_album_artist(album_artist: *const c_char, encoding: *const c_char, tag: *mut ID3v2_tag);
    fn tag_set_genre(genre: *const c_char, encoding: *const c_char, tag: *mut ID3v2_tag);
    fn tag_set_track(track: *const c_char, encoding: *const c_char, tag: *mut ID3v2_tag);
    fn tag_set_year(year: *const c_char, encoding: *const c_char, tag: *mut ID3v2_tag);
    fn tag_set_comment(comment: *const c_char, encoding: *const c_char, tag: *mut ID3v2_tag);
    fn tag_set_disc_number(disc_number: *const c_char, encoding: *const c_char, tag: *mut ID3v2_tag);
    fn tag_set_composer(composer: *const c_char, encoding: *const c_char, tag: *mut ID3v2_tag);
    fn tag_set_album_cover(file_name: *const c_char, tag: *mut ID3v2_tag);
    fn tag_set_album_cover_from_bytes(album_cover_bytes: *const c_char, mimetype: *const c_char, picture_size: c_int, tag: *mut ID3v2_tag);
}
//------------------------------------------------------------------------------------------------//
// end extern
//------------------------------------------------------------------------------------------------//

struct RawTag {
    raw_tag: *mut ID3v2_tag,
}

impl RawTag {
    fn new(file_name: &str) -> RawTag {
        let file_string = CString::new(file_name).unwrap();
        unsafe {
            RawTag {
                raw_tag: load_tag(file_string.as_ptr())
            }
        }
    }

    fn _frame<F>(&self, closure: F) -> String where F: Fn() -> *mut ID3v2_frame {
        Self::_read_content(closure())
    }

    fn title(&self) -> String {
        self._frame(|| unsafe { tag_get_title(self.raw_tag) })
    }

    fn artist(&self) -> String {
        self._frame(|| unsafe { tag_get_artist(self.raw_tag) })
    }

    fn album(&self) -> String {
        self._frame(|| unsafe { tag_get_album(self.raw_tag) })
    }

    fn album_artist(&self) -> String {
        self._frame(|| unsafe { tag_get_album_artist(self.raw_tag) })
    }

    fn genre(&self) -> String {
        self._frame(|| unsafe { tag_get_genre(self.raw_tag) })
    }

    fn track(&self) -> String {
        self._frame(|| unsafe { tag_get_track(self.raw_tag) })
    }

    fn year(&self) -> String {
        self._frame(|| unsafe { tag_get_year(self.raw_tag) })
    }

    fn comment(&self) -> String {
        self._frame(|| unsafe { tag_get_comment(self.raw_tag) })
    }

    fn disc_number(&self) -> String {
        self._frame(|| unsafe { tag_get_disc_number(self.raw_tag) })
    }

    fn composer(&self) -> String {
        self._frame(|| unsafe { tag_get_composer(self.raw_tag) })
    }

    fn album_cover(&self) -> String {
        self._frame(|| unsafe { tag_get_album_cover(self.raw_tag) })
    }

    //
    // FIXME id3v2에서 인코딩이 깨진다.
    //
    fn _read_content(frame: *mut ID3v2_frame) -> String {
        unsafe {
            if frame.is_null() {
                String::default()
            } else {
                let content = parse_text_frame_content(frame);
                CStr::from_ptr((*content).data).to_string_lossy().into_owned()
            }
        }
    }
}

fn main() {
    if let Some(file_name) = env::args().nth(1) {
        let meta_tag = RawTag::new(file_name.as_str());

        let title = meta_tag.title();
        let artist = meta_tag.artist();
        let album = meta_tag.album();
        let album_artist = meta_tag.album_artist();
        let genre = meta_tag.genre();
        let track = meta_tag.track();
        let year = meta_tag.year();
        let comment = meta_tag.comment();
        let disc_number = meta_tag.disc_number();
        let composer = meta_tag.composer();
        let album_cover = meta_tag.album_cover();

        println!("Title: {}", title);
        println!("Artist: {}", artist);
        println!("Album: {}", album);
        println!("Album Artist: {}", album_artist);
        println!("Genre: {}", genre);
        println!("Track: {}", track);
        println!("Year: {}", year);
        println!("Comment: {}", comment);
        println!("Disc_number: {}", disc_number);
        println!("Composer: {}", composer);
        println!("Album_cover: {}", album_cover);
    } else {
        println!("Usage: rtag <file name>");
    }
}