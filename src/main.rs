extern crate libc;

use std::env;
use std::ptr;
use std::ffi::*;
use libc::*;

// type.h
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

struct MetaTag {
    tag: *mut ID3v2_tag,
}

impl MetaTag {

    fn new(file_name: &str) -> MetaTag {
        let file_string = CString::new(file_name).unwrap();
        unsafe {
            MetaTag {
                tag: load_tag(file_string.as_ptr())
            }
        }
    }

    fn title(&self) -> String {
        let frame = unsafe {
            tag_get_title(self.tag)
        };
        Self::_read_content(frame)
    }

    fn _read_content(frame: *mut ID3v2_frame) -> String {
        unsafe {
            let content = parse_text_frame_content(frame);
            CStr::from_ptr((*content).data).to_string_lossy().into_owned()
        }
    }
}

fn main() {
    if let Some(file_name) = env::args().nth(1) {
        let meta_tag = MetaTag::new(file_name.as_str());
        let title = meta_tag.title();
        println!("Title: {}", title);
    } else {
        println!("Usage: rtag <file name>");
    }
}