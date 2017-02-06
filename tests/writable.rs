extern crate rtag;

use std::fs::{
    self,
    OpenOptions
};
use rtag::writable::WritableFactory;
use rtag::readable::ReadableFactory;

#[test]
fn shift() {
    fn init(path: &str) {
        let file = OpenOptions::new().write(true).create_new(true).open(path).unwrap();
        let mut writable = file.to_writable();
        writable.string("1234567890abcdefghij").unwrap();
    }

    let path = "./test-resources/write.txt";
    let _ = fs::remove_file(path);

    init(path);
    {
        let file = OpenOptions::new().read(true).write(true).open(path).unwrap();
        let mut writable = file.to_writable();
        writable.skip(5).unwrap();
        let text = "###";
        writable.shift(text.as_bytes().len()).unwrap();
        writable.string(text).unwrap();

        let file = OpenOptions::new().read(true).open(path).unwrap();
        let mut readable = file.to_readable();
        assert_eq!(readable.all_string().unwrap(), "12345###67890abcdefghij".to_string());
        let _ = fs::remove_file(path);
    }

    init(path);
    {
        let file = OpenOptions::new().read(true).write(true).open(path).unwrap();
        let mut writable = file.to_writable();
        writable.skip(5).unwrap();
        let text = "#####$$$$$@@@@@";
        writable.shift(text.as_bytes().len()).unwrap();
        writable.string(text).unwrap();

        let file = OpenOptions::new().read(true).open(path).unwrap();
        let mut readable = file.to_readable();
        let _ = readable.position(0);
        assert_eq!(readable.all_string().unwrap(), "12345#####$$$$$@@@@@67890abcdefghij".to_string());
        let _ = fs::remove_file(path);
    }

    init(path);
    {
        let file = OpenOptions::new().read(true).write(true).open(path).unwrap();
        let mut writable = file.to_writable();
        writable.skip(10).unwrap();
        let text = "#####$$";
        writable.shift(text.as_bytes().len()).unwrap();
        writable.string(text).unwrap();

        let file = OpenOptions::new().read(true).open(path).unwrap();
        let mut readable = file.to_readable();
        let _ = readable.position(0);
        assert_eq!(readable.all_string().unwrap(), "1234567890#####$$abcdefghij".to_string());
        let _ = fs::remove_file(path);
    }

    init(path);
    {
        let file = OpenOptions::new().read(true).write(true).open(path).unwrap();
        let mut writable = file.to_writable();
        writable.skip(20).unwrap();
        let text = "$";
        writable.shift(text.as_bytes().len()).unwrap();
        writable.string(text).unwrap();

        let file = OpenOptions::new().read(true).open(path).unwrap();
        let mut readable = file.to_readable();
        let _ = readable.position(0);
        assert_eq!(readable.all_string().unwrap(), "1234567890abcdefghij$".to_string());
        let _ = fs::remove_file(path);
    }

    init(path);
    {
        let file = OpenOptions::new().read(true).write(true).open(path).unwrap();
        let mut writable = file.to_writable();
        let text = "******";
        writable.shift(text.as_bytes().len()).unwrap();
        writable.string(text).unwrap();

        let file = OpenOptions::new().read(true).open(path).unwrap();
        let mut readable = file.to_readable();
        let _ = readable.position(0);
        assert_eq!(readable.all_string().unwrap(), "******1234567890abcdefghij".to_string());
        let _ = fs::remove_file(path);
    }
}

#[test]
fn unshift() {
    fn init(path: &str) {
        let file = OpenOptions::new().write(true).create_new(true).open(path).unwrap();
        let mut writable = file.to_writable();
        writable.string("1234567890abcdefghij").unwrap();
    }

    let path = "./test-resources/write.txt";
    let _ = fs::remove_file(path);

    init(path);
    {
        let file = OpenOptions::new().read(true).write(true).open(path).unwrap();
        let mut writable = file.to_writable();
        writable.unshift(5).unwrap();

        let file = OpenOptions::new().read(true).open(path).unwrap();
        let mut readable = file.to_readable();
        let _ = readable.position(0);
        assert_eq!(readable.all_string().unwrap(), "67890abcdefghij\u{0}\u{0}\u{0}\u{0}\u{0}".to_string());
        let _ = fs::remove_file(path);
    }

    init(path);
    {
        let file = OpenOptions::new().read(true).write(true).open(path).unwrap();
        let mut writable = file.to_writable();
        writable.skip(5).unwrap();
        writable.unshift(5).unwrap();

        let file = OpenOptions::new().read(true).open(path).unwrap();
        let mut readable = file.to_readable();
        let _ = readable.position(0);
        assert_eq!(readable.all_string().unwrap(), "12345abcdefghij\u{0}\u{0}\u{0}\u{0}\u{0}".to_string());
        let _ = fs::remove_file(path);
    }
}