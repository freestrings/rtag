mod bytes;
mod reader;
mod tag;

#[cfg(test)]
mod tests {
    extern crate env_logger;

    use scanner;
    use super::reader::FrameIterator;

    #[test]
    fn scanner() {
        let _ = env_logger::init();

        match scanner::Scanner::new("./resources/file1.txt") {
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

        match scanner::Scanner::new("./resources/230.mp3") {
            Ok(mut scanner) => {
                if let Ok(bytes) = scanner.read_as_bytes(10) {
                    let header = super::tag::header::Header::new(bytes);
                    assert_eq!(header.get_version(), 3);
                    assert_eq!(header.get_minor_version(), 0);
                    assert_eq!(header.has_flag(super::tag::header::HeaderFlag::Unsynchronisation), false);
                    assert_eq!(header.has_flag(super::tag::header::HeaderFlag::ExtendedHeader), false);
                    assert_eq!(header.has_flag(super::tag::header::HeaderFlag::ExperimentalIndicator), false);
                    assert_eq!(header.get_size(), 1171);
                }
            },
            Err(_) => assert!(false)
        }
    }

    #[test]
    fn idv3_240_header() {
        let _ = env_logger::init();

        match scanner::Scanner::new("./resources/240.mp3") {
            Ok(mut scanner) => {
                if let Ok(bytes) = scanner.read_as_bytes(10) {
                    let header = super::tag::header::Header::new(bytes);
                    assert_eq!(header.get_version(), 4);
                    assert_eq!(header.get_minor_version(), 0);
                    assert_eq!(header.has_flag(super::tag::header::HeaderFlag::Unsynchronisation), false);
                    assert_eq!(header.has_flag(super::tag::header::HeaderFlag::ExtendedHeader), false);
                    assert_eq!(header.has_flag(super::tag::header::HeaderFlag::ExperimentalIndicator), false);
                    assert_eq!(header.get_size(), 165126);
                }
            },
            Err(_) => assert!(false)
        }
    }

    #[test]
    fn idv3_230_frame_id() {
        let _ = env_logger::init();

        match scanner::Scanner::new("./resources/ID3v1-ID3v2.mp3") {
            Ok(mut scanner) => {
                if let Ok(mut frame_reader) = super::reader::FrameReader::new(&mut scanner) {
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
    fn idv3_230_frame_id2() {
        let _ = env_logger::init();

        match scanner::Scanner::new("./resources/230.mp3") {
            Ok(mut scanner) => {
                if let Ok(mut frame_reader) = super::reader::FrameReader::new(&mut scanner) {
                    let mut v = vec!["TALB", "TCON", "TIT2", "TLEN", "TPE1", "TRCK", "COMM", "TYER"];
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
    fn idv3_240_frame_id() {
        let _ = env_logger::init();

        match scanner::Scanner::new("./resources/240.mp3") {
            Ok(mut scanner) => {
                if let Ok(mut frame_reader) = super::reader::FrameReader::new(&mut scanner) {
                    let mut v = vec!["TDRC", "TRCK", "TPOS", "TPE1", "TALB", "TPE2", "TIT2", "TSRC", "TCON", "COMM"];
                    v.reverse();
                    loop {
                        if frame_reader.has_next_frame() {
                            if let Ok(frame) = frame_reader.next_frame() {
                                assert_eq!(v.pop().unwrap(), frame.get_id());
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

        match scanner::Scanner::new("./resources/ID3v1-ID3v2.mp3") {
            Ok(mut scanner) => {
                if let Ok(mut frame_reader) = super::reader::FrameReader::new(&mut scanner) {
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

    #[test]
    fn idv3_230_frame_data2() {
        let _ = env_logger::init();

        match scanner::Scanner::new("./resources/230.mp3") {
            Ok(mut scanner) => {
                if let Ok(mut frame_reader) = super::reader::FrameReader::new(&mut scanner) {
                    // \u{feff} => 공백
                    let mut v = vec!["\u{feff}앨범", "Rock", "\u{feff}Tㅏi틀", "0", "\u{feff}아티st", "1", "eng\u{0}!!!@@#$@$^#$%^\\n123", "2017"];
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

    #[test]
    fn idv3_240_frame_data() {
        let _ = env_logger::init();

        match scanner::Scanner::new("./resources/240.mp3") {
            Ok(mut scanner) => {
                if let Ok(mut frame_reader) = super::reader::FrameReader::new(&mut scanner) {
                    // \u{feff} => 공백
                    let mut v = vec!["2017", "1", "1", "아티스트", "Album", "Artist/아티스트", "타이틀", "ABAB", "Alternative", "eng\u{0}~~"];
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