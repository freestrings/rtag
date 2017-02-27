extern crate tempdir;

extern crate rtag;

use tempdir::TempDir;

use std::fs::{self, File, OpenOptions};
use std::path::Path;

use rtag::rw::*;

fn get_file(path: &Path) -> File {
    OpenOptions::new().read(true).write(true).open(path).unwrap()
}

fn init(path: &Path) {
    let _ = fs::remove_file(path);
    let mut file = OpenOptions::new().write(true).create_new(true).open(path).unwrap();
    file.write_string("1234567890abcdefghij").unwrap();
}

#[test]
fn shift() {
    let tmp_dir = TempDir::new("rtag").unwrap();
    let path = tmp_dir.path().join("write_shift.txt");
    let path = path.as_path();

    init(path);
    {
        let mut writable = get_file(path);
        writable.skip_bytes(5).unwrap();
        let text = "###";
        writable.shift(text.as_bytes().len()).unwrap();
        writable.write_string(text).unwrap();

        let mut readable = get_file(path);
        assert_eq!(readable.all_string().unwrap(),
                   "12345###67890abcdefghij".to_string());
    }

    init(path);
    {
        let mut writable = get_file(path);
        writable.skip_bytes(5).unwrap();
        let text = "#####$$$$$@@@@@";
        writable.shift(text.as_bytes().len()).unwrap();
        writable.write_string(text).unwrap();

        let mut readable = get_file(path);
        let _ = readable.position(0);
        assert_eq!(readable.all_string().unwrap(),
                   "12345#####$$$$$@@@@@67890abcdefghij".to_string());
    }

    init(path);
    {
        let mut writable = get_file(path);
        writable.skip_bytes(10).unwrap();
        let text = "#####$$";
        writable.shift(text.as_bytes().len()).unwrap();
        writable.write_string(text).unwrap();

        let mut readable = get_file(path);
        let _ = readable.position(0);
        assert_eq!(readable.all_string().unwrap(),
                   "1234567890#####$$abcdefghij".to_string());
    }

    init(path);
    {
        let mut writable = get_file(path);
        writable.skip_bytes(20).unwrap();
        let text = "$";
        writable.shift(text.as_bytes().len()).unwrap();
        writable.write_string(text).unwrap();

        let mut readable = get_file(path);
        let _ = readable.position(0);
        assert_eq!(readable.all_string().unwrap(),
                   "1234567890abcdefghij$".to_string());
    }

    init(path);
    {
        let mut writable = get_file(path);
        let text = "******";
        writable.shift(text.as_bytes().len()).unwrap();
        writable.write_string(text).unwrap();

        let mut readable = get_file(path);
        let _ = readable.position(0);
        assert_eq!(readable.all_string().unwrap(),
                   "******1234567890abcdefghij".to_string());
    }
}

#[test]
fn unshift() {
    let tmp_dir = TempDir::new("rtag").unwrap();
    let path = tmp_dir.path().join("write_unshift.txt");
    let path = path.as_path();

    init(path);
    {
        let mut writable = get_file(path);
        writable.unshift(5).unwrap();

        let mut readable = get_file(path);
        let _ = readable.position(0);
        assert_eq!(readable.all_string().unwrap(),
                   "67890abcdefghij\u{0}\u{0}\u{0}\u{0}\u{0}".to_string());
    }

    init(path);
    {
        let mut writable = get_file(path);
        writable.skip_bytes(5).unwrap();
        writable.unshift(5).unwrap();

        let mut readable = get_file(path);
        let _ = readable.position(0);
        assert_eq!(readable.all_string().unwrap(),
                   "12345abcdefghij\u{0}\u{0}\u{0}\u{0}\u{0}".to_string());
    }
}