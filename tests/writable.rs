extern crate tempdir;

extern crate rtag;

use tempdir::TempDir;

use std::fs::{self, File, OpenOptions};
use std::path::Path;

use rtag::writable::WritableFactory;
use rtag::readable::ReadableFactory;

fn get_file(path: &Path) -> File {
    OpenOptions::new().read(true).write(true).open(path).unwrap()
}

fn init(path: &Path) {
    let _ = fs::remove_file(path);
    let file = OpenOptions::new().write(true).create_new(true).open(path).unwrap();
    let mut writable = file.to_writable();
    writable.string("1234567890abcdefghij").unwrap();
}

#[test]
fn shift() {
    let tmp_dir = TempDir::new("rtag").unwrap();
    let path = tmp_dir.path().join("write_shift.txt");
    let path = path.as_path();

    init(path);
    {
        let file = get_file(path);
        let mut writable = file.to_writable();
        writable.skip(5).unwrap();
        let text = "###";
        writable.shift(text.as_bytes().len()).unwrap();
        writable.string(text).unwrap();

        let file = get_file(path);
        let mut readable = file.to_readable();
        assert_eq!(readable.all_string().unwrap(),
                   "12345###67890abcdefghij".to_string());
    }

    init(path);
    {
        let file = get_file(path);
        let mut writable = file.to_writable();
        writable.skip(5).unwrap();
        let text = "#####$$$$$@@@@@";
        writable.shift(text.as_bytes().len()).unwrap();
        writable.string(text).unwrap();

        let file = get_file(path);
        let mut readable = file.to_readable();
        let _ = readable.position(0);
        assert_eq!(readable.all_string().unwrap(),
                   "12345#####$$$$$@@@@@67890abcdefghij".to_string());
    }

    init(path);
    {
        let file = get_file(path);
        let mut writable = file.to_writable();
        writable.skip(10).unwrap();
        let text = "#####$$";
        writable.shift(text.as_bytes().len()).unwrap();
        writable.string(text).unwrap();

        let file = get_file(path);
        let mut readable = file.to_readable();
        let _ = readable.position(0);
        assert_eq!(readable.all_string().unwrap(),
                   "1234567890#####$$abcdefghij".to_string());
    }

    init(path);
    {
        let file = get_file(path);
        let mut writable = file.to_writable();
        writable.skip(20).unwrap();
        let text = "$";
        writable.shift(text.as_bytes().len()).unwrap();
        writable.string(text).unwrap();

        let file = get_file(path);
        let mut readable = file.to_readable();
        let _ = readable.position(0);
        assert_eq!(readable.all_string().unwrap(),
                   "1234567890abcdefghij$".to_string());
    }

    init(path);
    {
        let file = get_file(path);
        let mut writable = file.to_writable();
        let text = "******";
        writable.shift(text.as_bytes().len()).unwrap();
        writable.string(text).unwrap();

        let file = get_file(path);
        let mut readable = file.to_readable();
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
        let file = get_file(path);
        let mut writable = file.to_writable();
        writable.unshift(5).unwrap();

        let file = get_file(path);
        let mut readable = file.to_readable();
        let _ = readable.position(0);
        assert_eq!(readable.all_string().unwrap(),
                   "67890abcdefghij\u{0}\u{0}\u{0}\u{0}\u{0}".to_string());
    }

    init(path);
    {
        let file = get_file(path);
        let mut writable = file.to_writable();
        writable.skip(5).unwrap();
        writable.unshift(5).unwrap();

        let file = get_file(path);
        let mut readable = file.to_readable();
        let _ = readable.position(0);
        assert_eq!(readable.all_string().unwrap(),
                   "12345abcdefghij\u{0}\u{0}\u{0}\u{0}\u{0}".to_string());
    }
}