mod frame;
mod frame_reader;
mod scanner;
mod tag_header;

fn to_u32(bytes: &[u8]) -> u32 {
    let mut v: u32 = (bytes[3] & 0xff) as u32;
    v = v | ((bytes[2] & 0xff) as u32) << 8;
    v = v | ((bytes[1] & 0xff) as u32) << 16;
    v = v | ((bytes[0] & 0xff) as u32) << 24;
    v
}

// Sizes are 4bytes long big-endian but first bit is 0
fn to_synchsafe(bytes: &[u8]) -> u32 {
    let mut v: u32 = (bytes[3] & 0x7f) as u32;
    v = v | ((bytes[2] & 0x7f) as u32) << 7;
    v = v | ((bytes[1] & 0x7f) as u32) << 14;
    v = v | ((bytes[0] & 0x7f) as u32) << 21;
    v
}

#[cfg(test)]
mod tests {
    extern crate env_logger;

    #[test]
    fn scanner() {
        let _ = env_logger::init();

        match super::scanner::Scanner::new("./resources/file1.txt") {
            Ok(mut scanner) => {
                assert_eq! ( match scanner.read_as_bytes(10) {
                    Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                    Err(_) => "".to_string()
                }, "1234567890");
                assert! (scanner.has_next());
                assert! (scanner.skip(5).is_ok());
                assert! (scanner.has_next());
                assert! (scanner.rewind(5).is_ok());
                assert_eq! ( match scanner.read_as_bytes(15) {
                    Ok(ref bytes) => String::from_utf8_lossy(bytes).into_owned(),
                    Err(_) => "".to_string()
                }, "abcdefghij");
                assert! ( !scanner.has_next());
            },
            Err(_) => assert! ( false)
        }
    }

    #[test]
    fn idv3_230_header() {
        let _ = env_logger::init();

        match super::scanner::Scanner::new("./resources/230.mp3") {
            Ok(mut scanner) => {
                if let Ok(bytes) = scanner.read_as_bytes(10) {
                    let tag_header = super::tag_header::TagHeader::new(bytes);
                    assert_eq!(tag_header.get_version(), 3);
                    assert_eq!(tag_header.get_minor_version(), 0);
                    assert_eq!(tag_header.has_unsynchronisation(), false);
                    assert_eq!(tag_header.has_extended(), false);
                    assert_eq!(tag_header.has_experimental(), false);
                    assert_eq!(tag_header.get_size(), 1182);
                }
            },
            Err(_) => assert!(false)
        }
    }

    #[test]
    fn idv3_230_frame_id() {
        let _ = env_logger::init();

        match super::scanner::Scanner::new("./resources/ID3v1-ID3v2.mp3") {
            Ok(mut scanner) => {
                if let Ok(mut frame_reader) = super::frame_reader::FrameReader::new(&mut scanner) {
                    let mut v = vec!["TIT2", "TPE1", "TALB", "TPE2", "TCON", "COMM", "TRCK", "TPOS"];
                    v.reverse();
                    loop {
                        if frame_reader.has_next_frame() {
                            if let Ok(frame) = frame_reader.next_frame() {
                                assert_eq!(v.pop().unwrap(), frame.get_id())
                            }
                        } else {
                            break;
                        }
                    }
                }
            },
            Err(_) => assert!(false)
        }
    }

    #[test]
    fn idv3_230_frame_data() {
        let _ = env_logger::init();

        match super::scanner::Scanner::new("./resources/ID3v1-ID3v2.mp3") {
            Ok(mut scanner) => {
                if let Ok(mut frame_reader) = super::frame_reader::FrameReader::new(&mut scanner) {
                    let mut v = vec!["타이틀", "Artist", "アルバム", "Album Artist", "Heavy Metal", "eng\u{0}!@#$", "1", "0"];
                    v.reverse();
                    loop {
                        if frame_reader.has_next_frame() {
                            if let Ok(frame) = frame_reader.next_frame() {
                                debug!("{}: {:?}", frame.get_id(), frame.get_data());
                                assert_eq!(v.pop().unwrap(), frame.get_data().unwrap())
                            }
                        } else {
                            break;
                        }
                    }
                }
            },
            Err(_) => assert!(false)
        }
    }
}