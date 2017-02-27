#[macro_use]
extern crate log;
extern crate env_logger;
extern crate tempdir;

extern crate rtag;

use tempdir::TempDir;

use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::vec::Vec;

use rtag::frame::*;
use rtag::frame::types::*;
use rtag::metadata::*;
use rtag::rw::*;

#[test]
fn metadata_regex() {
    let frame_id = regex::Regex::new(r"^[A-Z][A-Z0-9]{2,}$").unwrap();
    assert!(frame_id.is_match("AAA0"));
    assert!(frame_id.is_match("AAA"));
    assert!(!frame_id.is_match("0AA"));
    assert!(!frame_id.is_match("AA"));
    assert!(frame_id.is_match("COM"));
}

#[test]
fn metadata_empty() {
    let _ = env_logger::init();

    for m in MetadataReader::new("./test-resources/empty-meta.mp3").unwrap() {
        match m {
            Unit::FrameV1(_) => assert!(false),
            Unit::FrameV2(_, _) => assert!(false),
            _ => (),
        }
    }
}

#[test]
fn metadata_v1() {
    let _ = env_logger::init();

    for m in MetadataReader::new("./test-resources/v1-v2.mp3").unwrap() {
        match m {
            Unit::FrameV1(frame) => {
                debug!("v1: {:?}", frame);
                assert_eq!("Artist", frame.artist);
                assert_eq!("!@#$", frame.comment);
                assert_eq!("1", frame.track);
                assert_eq!("137", frame.genre);
            }
            _ => (),
        }
    }

    let id3v1_tag = concat!("TITLETITLETITLETITLETITLETITLE",
                            "ARTISTARTISTARTISTARTISTARTIST",
                            "ALBUMALBUMALBUMALBUMALBUMALBUM",
                            "2017",
                            "COMMENTCOMMENTCOMMENTCOMMENTCO4");

    let mut readable = Cursor::new(id3v1_tag.to_string().into_bytes());
    let frame = Frame1::read(&mut readable).unwrap();
    assert_eq!(frame.title, "TITLETITLETITLETITLETITLETITLE");
    assert_eq!(frame.artist, "ARTISTARTISTARTISTARTISTARTIST");
    assert_eq!(frame.album, "ALBUMALBUMALBUMALBUMALBUMALBUM");
    assert_eq!(frame.comment, "COMMENTCOMMENTCOMMENTCOMMENTCO");
    assert_eq!(frame.year, "2017");

    let id3v1_tag = concat!("TITLE                         ",
                            "ARTIST                        ",
                            "ALBUM                         ",
                            "2017",
                            "COMMENT                        ");

    let mut readable = Cursor::new(id3v1_tag.to_string().into_bytes());
    let frame = Frame1::read(&mut readable).unwrap();
    assert_eq!(frame.title, "TITLE");
    assert_eq!(frame.artist, "ARTIST");
    assert_eq!(frame.album, "ALBUM");
    assert_eq!(frame.comment, "COMMENT");
    assert_eq!(frame.year, "2017");
}

#[test]
fn metadata_v1_no_id() {
    let _ = env_logger::init();

    for m in MetadataReader::new("./test-resources/230-no-id3.mp3").unwrap() {
        match m {
            Unit::FrameV1(_) => assert!(false),
            _ => (),
        }
    }
}

#[test]
fn metadata_header() {
    let _ = env_logger::init();

    for m in MetadataReader::new("./test-resources/230.mp3").unwrap() {
        match m {
            Unit::Header(header) => {
                assert_eq!(3, header.version);
                assert_eq!(0, header.minor_version);
                assert_eq!(header.has_flag(HeadFlag::Unsynchronisation), false);
                assert_eq!(header.has_flag(HeadFlag::ExtendedHeader), false);
                assert_eq!(header.has_flag(HeadFlag::ExperimentalIndicator), false);
                assert_eq!(header.size, 1171);
            }
            _ => (),
        }
    }

    for m in MetadataReader::new("./test-resources/240.mp3").unwrap() {
        match m {
            Unit::Header(header) => {
                assert_eq!(4, header.version);
                assert_eq!(0, header.minor_version);
                assert_eq!(header.has_flag(HeadFlag::Unsynchronisation), false);
                assert_eq!(header.has_flag(HeadFlag::ExtendedHeader), false);
                assert_eq!(header.has_flag(HeadFlag::ExperimentalIndicator), false);
                assert_eq!(header.size, 165126);
            }
            _ => (),
        }
    }

    for m in MetadataReader::new("./test-resources/230.mp3").unwrap() {
        match m {
            Unit::Header(header) => {
                let writer = MetadataWriter::new("").unwrap();
                let bytes = writer.head(header.clone()).unwrap();
                let mut readable = Cursor::new(bytes);
                assert_eq!(header, Head::read(&mut readable, 3, "").unwrap());
            }
            _ => (),
        }
    }

    for m in MetadataReader::new("./test-resources/240.mp3").unwrap() {
        match m {
            Unit::Header(header) => {
                let writer = MetadataWriter::new("").unwrap();
                let bytes = writer.head(header.clone()).unwrap();
                let mut readable = Cursor::new(bytes);
                assert_eq!(header, Head::read(&mut readable, 4, "").unwrap());
            }
            _ => (),
        }
    }
}

#[test]
fn metadata_frame_body() {
    let _ = env_logger::init();

    fn test(path: &str, mut data: Vec<&str>) {
        let mut copied = data.clone();

        for m in MetadataReader::new(path).unwrap() {
            match m {
                Unit::FrameV2(_, frame) => {
                    compare_frame(frame, &mut data);
                }
                _ => (),
            }
        }

        assert_eq!(0, data.len());

        let meta_writer = MetadataWriter::new("").unwrap();
        for meta in MetadataReader::new(path).unwrap() {
            match meta {
                Unit::FrameV2(head, frame) => {
                    let frame_bytes = meta_writer.frame((head.clone(), frame.clone()))
                        .unwrap();

                    let mut readable = Cursor::new(frame_bytes);

                    match &head {
                        &FrameHeader::V22(_) => {
                            FrameHeaderV2::read(&mut readable, 2, "").unwrap();
                        }
                        &FrameHeader::V23(_) => {
                            FrameHeaderV3::read(&mut readable, 3, "").unwrap();
                        }
                        &FrameHeader::V24(_) => {
                            FrameHeaderV4::read(&mut readable, 4, "").unwrap();
                        }
                    };

                    let mut frame_readable = Cursor::new(readable.all_bytes().unwrap());
                    compare_frame_bytes(&mut frame_readable, frame, &mut copied);
                }
                _ => (),
            }
        }

        assert_eq!(0, copied.len());
    }

    test("./test-resources/v1-v2.mp3",
         vec!["타이틀",
              "Artist",
              "アルバム",
              "Album Artist",
              "Heavy Metal",
              "eng::!@#$",
              "1",
              "0"]);

    test("./test-resources/230.mp3",
         vec!["앨범",
              "Rock",
              "Tㅏi틀",
              "0",
              "아티st",
              "1",
              "eng::!!!@@#$@$^#$%^\\n123",
              "2017"]);

    test("./test-resources/240.mp3",
         vec!["2017",
              "1",
              "1",
              "아티스트",
              "Album",
              "Artist/아티스트",
              "타이틀",
              "ABAB",
              "Alternative",
              "eng::~~"]);

    test("./test-resources/v2.2.mp3",
         vec!["Test v2.2.0", "Pudge", "2", "1998", "(37)", "eng::All Rights Reserved\u{0}"]);
}

#[test]
fn metadata_frame_etco() {
    let _ = env_logger::init();

    for m in MetadataReader::new("./test-resources/230-etco.mp3").unwrap() {
        match m {
            Unit::FrameV2(head, FrameBody::ETCO(frame)) => {
                assert_eq!(&TimestampFormat::Milliseconds, &frame.timestamp_format);

                match frame.event_timing_codes[0] {
                    EventTimingCode::MainPartStart(timestamp) => assert_eq!(timestamp, 152110),
                    _ => assert!(false),
                }

                let meta_writer = MetadataWriter::new("").unwrap();
                let frame_bytes = meta_writer.frame((head.clone(), FrameBody::ETCO(frame.clone())))
                    .unwrap();

                let mut readable = Cursor::new(frame_bytes);

                assert_eq!(head,
                           FrameHeader::V23(FrameHeaderV3::read(&mut readable, 3, "").unwrap()));
                assert_eq!(frame, ETCO::read(&mut readable, 3, "ETCO").unwrap());
            }
            _ => (),
        }
    }
}

#[test]
fn metadata_frame_pcnt() {
    let _ = env_logger::init();

    for m in MetadataReader::new("./test-resources/240-pcnt.mp3").unwrap() {
        match m {
            Unit::FrameV2(head, FrameBody::PCNT(frame)) => {
                assert_eq!(256, frame.counter);

                let meta_writer = MetadataWriter::new("").unwrap();
                let frame_bytes = meta_writer.frame((head.clone(), FrameBody::PCNT(frame.clone())))
                    .unwrap();

                let mut readable = Cursor::new(frame_bytes);

                assert_eq!(head,
                           FrameHeader::V24(FrameHeaderV4::read(&mut readable, 4, "").unwrap()));
                assert_eq!(frame, PCNT::read(&mut readable, 4, "PCNT").unwrap());
            }
            _ => (),
        }
    }
}

#[test]
fn metadata_frame_tbpm() {
    let _ = env_logger::init();

    for m in MetadataReader::new("./test-resources/230-tbpm.mp3").unwrap() {
        match m {
            Unit::FrameV2(head, FrameBody::TBPM(frame)) => {
                assert_eq!("0", frame.text);

                let meta_writer = MetadataWriter::new("").unwrap();
                let frame_bytes = meta_writer.frame((head.clone(), FrameBody::TBPM(frame.clone())))
                    .unwrap();

                let mut readable = Cursor::new(frame_bytes);

                let origin_frame_header = FrameHeader::V23(FrameHeaderV3 {
                    id: id::TBPM.to_string(),
                    size: 5,
                    status_flag: 0,
                    encoding_flag: 0,
                });
                let new_frame_header = FrameHeader::V23(FrameHeaderV3::read(&mut readable, 3, "")
                    .unwrap());

                assert_eq!(origin_frame_header, new_frame_header);
                assert_eq!(frame, TEXT::read(&mut readable, 3, id::TBPM).unwrap());
            }
            _ => (),
        }
    }
}

#[test]
fn metadata_encoding() {
    let _ = env_logger::init();

    for m in MetadataReader::new("./test-resources/v1-iso-8859-1.mp3").unwrap() {
        match m {
            Unit::FrameV1(frame) => {
                assert_eq!("räksmörgås", frame.title);
                assert_eq!("räksmörgås", frame.artist);
                assert_eq!("räksmörgås", frame.album);
                assert_eq!("räksmörgås", frame.comment);

                let meta_writer = MetadataWriter::new("").unwrap();
                let frame_bytes = meta_writer.frame1(frame.clone()).unwrap();

                let mut readable = Cursor::new(frame_bytes);

                assert_eq!(frame, Frame1::read(&mut readable).unwrap());
            }
            _ => (),
        }
    }

    for m in MetadataReader::new("./test-resources/v1-utf8.mp3").unwrap() {
        match m {
            Unit::FrameV1(frame) => {
                assert_eq!("rÃ¤ksmÃ¶rgÃ¥s", frame.title);
                assert_eq!("rÃ¤ksmÃ¶rgÃ¥s", frame.artist);
                assert_eq!("rÃ¤ksmÃ¶rgÃ¥s", frame.album);
                assert_eq!("rÃ¤ksmÃ¶rgÃ¥s", frame.comment);

                let meta_writer = MetadataWriter::new("").unwrap();
                let frame_bytes = meta_writer.frame1(frame.clone()).unwrap();

                let mut readable = Cursor::new(frame_bytes);

                assert_eq!(frame, Frame1::read(&mut readable).unwrap());
            }
            _ => (),
        }
    }

    for m in MetadataReader::new("./test-resources/v2.3-iso-8859-1.mp3").unwrap() {
        match m {
            Unit::FrameV2(head, FrameBody::TPE1(frame)) => {
                assert_eq!("Ester Koèièková a Lubomír Nohavica", frame.text);

                let meta_writer = MetadataWriter::new("").unwrap();
                let frame_bytes = meta_writer.frame((head, FrameBody::TPE1(frame.clone())))
                    .unwrap();

                let mut readable = Cursor::new(frame_bytes);

                let _ = readable.skip_bytes(10);

                assert_eq!(frame, TEXT::read(&mut readable, 3, id::TPE1).unwrap());
            }
            Unit::FrameV2(head, FrameBody::TALB(frame)) => {
                assert_eq!("Ester Koèièková a Lubomír Nohavica s klavírem",
                           frame.text);

                let meta_writer = MetadataWriter::new("").unwrap();
                let frame_bytes = meta_writer.frame((head, FrameBody::TALB(frame.clone())))
                    .unwrap();

                let mut readable = Cursor::new(frame_bytes);

                let _ = readable.skip_bytes(10);

                assert_eq!(frame, TEXT::read(&mut readable, 3, id::TPE1).unwrap());
            }
            Unit::FrameV2(head, FrameBody::TIT2(frame)) => {
                assert_eq!("Tøem sestrám", frame.text);

                let meta_writer = MetadataWriter::new("").unwrap();
                let frame_bytes = meta_writer.frame((head, FrameBody::TIT2(frame.clone())))
                    .unwrap();

                let mut readable = Cursor::new(frame_bytes);

                let _ = readable.skip_bytes(10);

                assert_eq!(frame, TEXT::read(&mut readable, 3, id::TPE1).unwrap());
            }
            _ => (),
        }
    }
}

#[test]
fn metadata_v220() {
    let _ = env_logger::init();

    for m in MetadataReader::new("./test-resources/v2.2-pic.mp3").unwrap() {
        match m {
            Unit::FrameV2(head, FrameBody::PIC(frame)) => {
                compare_frame(FrameBody::PIC(frame.clone()), &mut vec!["PNG:Other::61007"]);

                let meta_writer = MetadataWriter::new("").unwrap();
                let frame_bytes = meta_writer.frame((head.clone(), FrameBody::PIC(frame.clone())))
                    .unwrap();

                let mut readable = Cursor::new(frame_bytes[6..].to_vec());

                compare_frame_bytes(&mut readable,
                                    FrameBody::PIC(frame.clone()),
                                    &mut vec!["PNG:Other::61007"]);
            }
            _ => (),
        }
    }
}

#[test]
fn metadata_compressed() {
    let _ = env_logger::init();

    let path = "./test-resources/v2.3-compressed-frame.mp3";

    let mut i = MetadataReader::new(path)
        .unwrap()
        .filter(|m| match m {
            &Unit::FrameV2(ref header, _) => header.has_flag(FrameHeaderFlag::Compression),
            _ => false,
        });

    match i.next().unwrap() {
        Unit::FrameV2(origin_head, FrameBody::TIT2(origin_frame)) => {

            assert_eq!("Compressed TIT2 Frame", origin_frame.text);

            let frame_bytes = MetadataWriter::new("")
                .unwrap()
                .frame((origin_head.clone(), FrameBody::TIT2(origin_frame.clone())))
                .unwrap();

            let mut readable = Cursor::new(frame_bytes);
            let mut file = fs::File::open(path).unwrap();

            match file.frame3(&mut readable).unwrap() {
                Unit::FrameV2(new_frame_header, new_frame_body) => {
                    assert_eq!(origin_head, new_frame_header);
                    assert_eq!(FrameBody::TIT2(origin_frame), new_frame_body);
                }
                _ => assert!(false),
            };
        }
        _ => {
            assert!(false);
        }
    }

    assert!(i.next().is_none());

    let path = "./test-resources/v2.4-compressed-frame.mp3";
    let mut i = MetadataReader::new(path)
        .unwrap()
        .filter(|m| match m {
            &Unit::FrameV2(ref header, _) => header.has_flag(FrameHeaderFlag::Compression),
            _ => false,
        });

    match i.next().unwrap() {
        Unit::FrameV2(origin_frame_header, FrameBody::TIT2(origin_frame)) => {
            assert_eq!("Compressed TIT2 Frame", origin_frame.text);

            let frame_bytes = MetadataWriter::new("")
                .unwrap()
                .frame((origin_frame_header.clone(), FrameBody::TIT2(origin_frame.clone())))
                .unwrap();

            let mut readable = Cursor::new(frame_bytes);
            let mut file = fs::File::open(path).unwrap();

            match file.frame4(&mut readable).unwrap() {
                Unit::FrameV2(new_frame_header, new_frame_body) => {
                    assert_eq!(origin_frame_header, new_frame_header);
                    assert_eq!(FrameBody::TIT2(origin_frame), new_frame_body);
                }
                _ => {
                    assert!(false);
                }
            };
        }
        _ => {
            assert!(false);
        }
    }

    assert!(i.next().is_none());
}

#[test]
fn metadata_encrypted() {
    let _ = env_logger::init();

    let path = "./test-resources/v2.3-encrypted-frame.mp3";
    let mut i = MetadataReader::new(path)
        .unwrap()
        .filter(|m| match m {
            &Unit::FrameV2(ref header, _) => header.has_flag(FrameHeaderFlag::Encryption),
            _ => false,
        });

    match i.next().unwrap() {
        Unit::FrameV2(orig_frame_header, FrameBody::SKIP(_, orig_frame)) => {
            let frame_header = match orig_frame_header.clone() {
                FrameHeader::V23(frame_header) => Some(frame_header),
                _ => None,
            };

            let frame_bytes = MetadataWriter::new("")
                .unwrap()
                .frame3(&mut frame_header.unwrap(),
                        FrameBody::OBJECT(OBJECT { data: orig_frame.clone() }))
                .unwrap();

            let mut readable = Cursor::new(frame_bytes);
            let mut file = fs::File::open(path).unwrap();

            match file.frame3(&mut readable).unwrap() {
                Unit::FrameV2(new_frame_header, FrameBody::SKIP(_, new_frame)) => {
                    assert_eq!(orig_frame_header, new_frame_header);
                    assert_eq!(orig_frame, new_frame);
                }
                _ => {
                    assert!(false);
                }
            };
        }
        _ => {
            assert!(false);
        }
    }

    assert!(i.next().is_none());

    let path = "./test-resources/v2.4-encrypted-frame.mp3";
    let mut i = MetadataReader::new(path)
        .unwrap()
        .filter(|m| match m {
            &Unit::FrameV2(ref header, _) => header.has_flag(FrameHeaderFlag::Encryption),
            _ => false,
        });

    match i.next().unwrap() {
        Unit::FrameV2(orig_frame_header, FrameBody::SKIP(_, orig_frame)) => {

            let frame_header = match orig_frame_header.clone() {
                FrameHeader::V24(frame_header) => Some(frame_header),
                _ => None,
            };

            let frame_bytes = MetadataWriter::new("")
                .unwrap()
                .frame4(&mut frame_header.unwrap(),
                        FrameBody::OBJECT(OBJECT { data: orig_frame.clone() }))
                .unwrap();

            let mut readable = Cursor::new(frame_bytes);
            let mut file = fs::File::open(path).unwrap();

            match file.frame4(&mut readable).unwrap() {
                Unit::FrameV2(new_frame_header, FrameBody::SKIP(_, new_frame)) => {
                    assert_eq!(orig_frame_header, new_frame_header);
                    assert_eq!(orig_frame, new_frame);
                }
                _ => {
                    assert!(false);
                }
            };
        }
        _ => {
            assert!(false);
        }
    };

    assert!(i.next().is_none());
}

#[test]
fn metadata_v230_ext_header() {
    let _ = env_logger::init();

    // file with extend header bit set but no extended header
    {
        let path = "./test-resources/v2.3-ext-header-invalid.mp3";
        let i = MetadataReader::new(path)
            .unwrap()
            .filter(|m| match m {
                &Unit::Header(ref header) => header.has_flag(HeadFlag::ExtendedHeader),
                _ => false,
            });

        assert!(i.count() == 1);

        let i = MetadataReader::new(path)
            .unwrap()
            .filter(|m| match m {
                &Unit::ExtendedHeader(_) => true,
                _ => false,
            });

        assert!(i.count() == 0);
    }

    {
        let path = "./test-resources/v2.3-ext-header.mp3";
        let i = MetadataReader::new(path)
            .unwrap()
            .filter(|m| match m {
                &Unit::ExtendedHeader(_) => true,
                _ => false,
            });

        assert!(i.count() == 1);

        for m in MetadataReader::new(path).unwrap() {
            match m {
                Unit::FrameV2(_, FrameBody::TCON(frame)) => assert_eq!("(0)Blues", frame.text),
                _ => (),
            }
        }
    }
}

// invalid frame is ingnored.
#[test]
fn metadata_v230_invalid_aenc() {
    let _ = env_logger::init();

    let i = MetadataReader::new("./test-resources/v2.3-invalid-aenc.mp3")
        .unwrap()
        .filter(|m| match m {
            &Unit::FrameV2(_, FrameBody::AENC(_)) => true,
            _ => false,
        });

    assert!(i.count() == 0);
}

#[test]
fn metadata_v230_link() {
    let _ = env_logger::init();

    let re = regex::Regex::new(r"^http://www\.emusic\.com").unwrap();

    let path = "./test-resources/v2.3-link-frame.mp3";
    for m in MetadataReader::new(path).unwrap() {
        match m {
            Unit::FrameV2(orig_frame_header, FrameBody::LINK(orig_frame)) => {

                assert_eq!("WCO", orig_frame.frame_identifier);
                assert!(re.is_match(orig_frame.url.as_str()));

                let frame_header = match orig_frame_header.clone() {
                    FrameHeader::V23(fh) => Some(fh),
                    _ => None,
                };

                let frame_bytes = MetadataWriter::new("")
                    .unwrap()
                    .frame3(&mut frame_header.unwrap(),
                            FrameBody::LINK(orig_frame.clone()))
                    .unwrap();

                let mut readable = Cursor::new(frame_bytes);
                let mut file = fs::File::open(path).unwrap();

                match file.frame3(&mut readable).unwrap() {
                    Unit::FrameV2(_, FrameBody::LINK(new_frame)) => {
                        assert_eq!(orig_frame, new_frame)
                    }
                    _ => {
                        assert!(false);
                    }
                };
            }
            _ => (),
        }
    }
}

#[test]
fn metadata_v230_mcdi() {
    let _ = env_logger::init();

    let path = "./test-resources/v2.3-mcdi.mp3";
    for m in MetadataReader::new(path).unwrap() {
        match m {
            Unit::FrameV2(orig_frame_header, FrameBody::MCDI(orig_frame)) => {
                assert_eq!(804, orig_frame.cd_toc.len());

                let frame_header = match orig_frame_header.clone() {
                    FrameHeader::V23(fh) => Some(fh),
                    _ => None,
                };

                let frame_bytes = MetadataWriter::new("")
                    .unwrap()
                    .frame3(&mut frame_header.unwrap(),
                            FrameBody::MCDI(orig_frame.clone()))
                    .unwrap();

                let mut readable = Cursor::new(frame_bytes);
                let mut file = fs::File::open(path).unwrap();

                match file.frame3(&mut readable).unwrap() {
                    Unit::FrameV2(new_frame_header, FrameBody::MCDI(new_frame)) => {
                        assert_eq!(orig_frame_header, new_frame_header);
                        assert_eq!(orig_frame, new_frame);
                    }
                    _ => {
                        assert!(false);
                    }
                };
            }
            _ => (),
        }
    }
}

#[test]
fn metadata_v240_geob() {
    let _ = env_logger::init();

    let path = "./test-resources/v2.4-geob-multiple.mp3";
    let mut i = MetadataReader::new(path).unwrap().filter(|m| match m {
        &Unit::FrameV2(_, FrameBody::GEOB(_)) => true,
        _ => false,
    });

    match i.next().unwrap() {
        Unit::FrameV2(orig_frame_header, FrameBody::GEOB(orig_frame)) => {
            assert_eq!("text/plain", orig_frame.mime_type);
            assert_eq!("eyeD3.txt", orig_frame.filename);
            assert_eq!("eyeD3 --help output", orig_frame.content_description);
            assert_eq!(6207, orig_frame.encapsulation_object.len());

            let frame_header = match orig_frame_header.clone() {
                FrameHeader::V24(fh) => Some(fh),
                _ => None,
            };

            let frame_bytes = MetadataWriter::new("")
                .unwrap()
                .frame4(&mut frame_header.unwrap(),
                        FrameBody::GEOB(orig_frame.clone()))
                .unwrap();

            let mut readable = Cursor::new(frame_bytes);
            let mut file = fs::File::open(path).unwrap();

            match file.frame4(&mut readable).unwrap() {
                Unit::FrameV2(new_frame_header, FrameBody::GEOB(new_frame)) => {
                    assert_eq!(orig_frame_header, new_frame_header);
                    assert_eq!(orig_frame, new_frame);
                } 
                _ => {
                    assert!(false);
                }
            };

        }
        _ => {
            assert!(false);
        }
    };

    match i.next().unwrap() {
        Unit::FrameV2(orig_frame_header, FrameBody::GEOB(orig_frame)) => {
            assert_eq!("text/plain", orig_frame.mime_type);
            assert_eq!("genres.txt", orig_frame.filename);
            assert_eq!("eyeD3 --list-genres output", orig_frame.content_description);
            assert_eq!(4087, orig_frame.encapsulation_object.len());

            let frame_header = match orig_frame_header.clone() {
                FrameHeader::V24(fh) => Some(fh),
                _ => None,
            };

            let frame_bytes = MetadataWriter::new("")
                .unwrap()
                .frame4(&mut frame_header.unwrap(),
                        FrameBody::GEOB(orig_frame.clone()))
                .unwrap();

            let mut readable = Cursor::new(frame_bytes);
            let mut file = fs::File::open(path).unwrap();

            match file.frame4(&mut readable).unwrap() {
                Unit::FrameV2(new_frame_header, FrameBody::GEOB(new_frame)) => {
                    assert_eq!(orig_frame_header, new_frame_header);
                    assert_eq!(orig_frame, new_frame);
                }
                _ => {
                    assert!(false);
                }
            };

        }
        _ => {
            assert!(false);
        }
    };

}


#[test]
fn metadata_unsync() {
    let _ = env_logger::init();

    fn test(path: &str, mut data: Vec<&str>) {
        let mut copied = data.clone();

        for m in MetadataReader::new(path).unwrap() {
            match m {
                Unit::FrameV2(_, frame) => {
                    compare_frame(frame.clone(), &mut data);
                }
                _ => (),
            }
        }

        assert_eq!(data.len(), 0);

        let meta_reader = MetadataReader::new(path).unwrap().collect::<Vec<Unit>>();

        let tmp_dir = TempDir::new("rtag").unwrap();
        let tmp_path = tmp_dir.path().join("metadata_unsync.txt");
        let _ = fs::remove_file(&tmp_path);
        let _ = fs::File::create(tmp_path.as_path()).unwrap();

        let path = tmp_path.to_str().unwrap();

        MetadataWriter::new(path).unwrap().write(meta_reader, false).unwrap();

        let mut i = MetadataReader::new(path)
            .unwrap()
            .filter(|m| match m {
                &Unit::FrameV2(_, _) => true,
                _ => false,
            });

        while let Some(Unit::FrameV2(_, frame)) = i.next() {
            compare_frame(frame.clone(), &mut copied);
        }

        assert_eq!(copied.len(), 0);
    }

    test("./test-resources/v2.3-unsync.mp3",
         vec!["ENG:Comment:http://www.mp3sugar.com/",
              "Carbon Based Lifeforms",
              "Carbon Based Lifeforms",
              "Hydroponic Garden",
              "Silent Running",
              "4",
              "2003",
              "(26)"]);

    test("./test-resources/v2.4-unsync.mp3",
         vec!["2009",
              "Album",
              "Artist",
              "Title",
              "replaygain_track_gain:+0.00 dB\u{0}",
              "replaygain_track_peak:0.000715\u{0}"]);
}

#[test]
fn metadata_writer() {
    let _ = env_logger::init();

    let tmp_dir = TempDir::new("rtag").unwrap();
    let tmp_path = tmp_dir.path().join("240.mp3");
    let path = tmp_path.to_str().unwrap();

    fs::copy("./test-resources/240.mp3", path).unwrap();

    let new_data = MetadataReader::new(path)
        .unwrap()
        .fold(Vec::new(), |mut vec, unit| {
            if let Unit::FrameV2(frame_head, frame_body) = unit {
                let new_frame_body = match frame_body {
                    FrameBody::TALB(ref frame) => {
                        let mut new_frame = frame.clone();
                        new_frame.text = "Album!".to_string();
                        FrameBody::TALB(new_frame)
                    }
                    _ => frame_body.clone(),
                };

                vec.push(Unit::FrameV2(frame_head, new_frame_body));
            } else {
                vec.push(unit);
            }

            vec
        });

    let writer = MetadataWriter::new(path).unwrap();
    let _ = writer.write(new_data, false);

    let mut i = MetadataReader::new(path)
        .unwrap()
        .filter(|unit| if let &Unit::FrameV2(_, FrameBody::TALB(_)) = unit {
            true
        } else {
            false
        });

    if let Unit::FrameV2(_, FrameBody::TALB(frame)) = i.next().unwrap() {
        assert_eq!(frame.text, "Album!");
    }

    assert!(i.next().is_none());

    let tmp_path = tmp_dir.path().join("220.mp3");
    let path = tmp_path.to_str().unwrap();

    fs::copy("./test-resources/v2.2.mp3", path).unwrap();

    let frames2_2 = MetadataReader::new(path).unwrap().collect::<Vec<Unit>>();
    let _ = MetadataWriter::new(path).unwrap().write(frames2_2, true);
    let i = MetadataReader::new(path)
        .unwrap()
        .filter(|unit| match unit {
            &Unit::FrameV2(FrameHeader::V22(_), _) => true,
            _ => false,
        });

    assert_eq!(i.count(), 0);

    let mut data = vec!["Test v2.2.0", "Pudge", "2", "(37)", "eng::All Rights Reserved\u{0}"];

    for unit in MetadataReader::new(path).unwrap() {
        match unit {
            Unit::FrameV2(FrameHeader::V24(_), frame_body) => compare_frame(frame_body, &mut data),
            _ => (),
        }
    }

    assert_eq!(data.len(), 0);

}

#[test]
fn json_test() {
    extern crate serde_json;

    let iter = MetadataReader::new("./test-resources/v2.3-unsync.mp3")
        .unwrap()
        .filter(|unit| match unit {
            &Unit::FrameV2(_, _) => true,
            _ => false,
        });

    let mut data = vec!["{\"FrameV2\":[{\"V23\":{\"id\":\"COMM\",\"size\":36,\"status_flag\":0,\
                         \"encoding_flag\":0}},{\"COMM\":{\"text_encoding\":\"ISO88591\",\
                         \"language\":\"ENG\",\"short_description\":\"Comment\",\"actual_text\":\
                         \"http://www.mp3sugar.com/\"}}]}",
                        "{\"FrameV2\":[{\"V23\":{\"id\":\"TPE2\",\"size\":47,\"status_flag\":0,\
                         \"encoding_flag\":0}},{\"TPE2\":{\"text_encoding\":\"UTF16LE\",\"text\":\
                         \"Carbon Based Lifeforms\"}}]}",
                        "{\"FrameV2\":[{\"V23\":{\"id\":\"TPE1\",\"size\":23,\"status_flag\":0,\
                         \"encoding_flag\":0}},{\"TPE1\":{\"text_encoding\":\"ISO88591\",\
                         \"text\":\"Carbon Based Lifeforms\"}}]}",
                        "{\"FrameV2\":[{\"V23\":{\"id\":\"TALB\",\"size\":18,\"status_flag\":0,\
                         \"encoding_flag\":0}},{\"TALB\":{\"text_encoding\":\"ISO88591\",\
                         \"text\":\"Hydroponic Garden\"}}]}",
                        "{\"FrameV2\":[{\"V23\":{\"id\":\"TIT2\",\"size\":15,\"status_flag\":0,\
                         \"encoding_flag\":0}},{\"TIT2\":{\"text_encoding\":\"ISO88591\",\
                         \"text\":\"Silent Running\"}}]}",
                        "{\"FrameV2\":[{\"V23\":{\"id\":\"TRCK\",\"size\":2,\"status_flag\":0,\
                         \"encoding_flag\":0}},{\"TRCK\":{\"text_encoding\":\"ISO88591\",\
                         \"text\":\"4\"}}]}",
                        "{\"FrameV2\":[{\"V23\":{\"id\":\"TYER\",\"size\":5,\"status_flag\":0,\
                         \"encoding_flag\":0}},{\"TYER\":{\"text_encoding\":\"ISO88591\",\
                         \"text\":\"2003\"}}]}",
                        "{\"FrameV2\":[{\"V23\":{\"id\":\"TCON\",\"size\":5,\"status_flag\":0,\
                         \"encoding_flag\":0}},{\"TCON\":{\"text_encoding\":\"ISO88591\",\
                         \"text\":\"(26)\"}}]}"];

    data.reverse();

    for m in iter {
        let j = serde_json::to_string(&m).unwrap();
        assert_eq!(j, data.pop().unwrap());
    }
    assert_eq!(0, data.len());
}

#[test]
fn frame_to_map_test() {

    let frame = ENCR {
        owner_identifier: "owner_identifier".to_string(),
        method_symbol: 1,
        encryption_data: vec![1, 2, 3],
    };

    let mut map = HashMap::new();
    map.insert("owner_identifier", "owner_identifier".to_string());
    map.insert("method_symbol", "1".to_string());
    map.insert("encryption_data", "".to_string());

    assert_eq!(frame.to_map().unwrap(), map);

    let frame_body = FrameBody::TIT2(TEXT {
        text_encoding: TextEncoding::ISO88591,
        text: "text".to_string()
    });

    let mut map = HashMap::new();
    map.insert("text_encoding", "ISO88591".to_string());
    map.insert("text", "text".to_string());

    assert_eq!(frame_body.to_map().unwrap(), map);
}

#[test]
fn frame_scan_test() {
    let frame = ENCR {
        owner_identifier: "owner_identifier".to_string(),
        method_symbol: 1,
        encryption_data: vec![1, 2, 3],
    };

    frame.inside(|key, value| {
        assert_eq!(key, "owner_identifier");
        assert_eq!(value, "owner_identifier".to_string());

        false
    });

    let frame_body = FrameBody::TIT2(TEXT {
        text_encoding: TextEncoding::ISO88591,
        text: "text".to_string()
    });

    frame_body.inside(|key, value| {
        assert_eq!(key, "text_encoding");
        assert_eq!(value, "ISO88591".to_string());

        false
    });
}

#[test]
fn frame_header_default() {
    let header = FrameHeader::V22(FrameHeaderV2 {
        id: "ABC".to_string(),
        size: 1
    });

    assert_eq!("ABC".to_string(), header.id());
    assert_eq!(1, header.size());
}

macro_rules! define_compare_frame {
    (
        $( $id:ident ),*
    ) => (

        fn compare_frame(frame_body: FrameBody, data: &mut Vec<&str>) {
            
            data.reverse();

            match frame_body {
                FrameBody::COMM(frame) => {
                    assert_eq!(data.pop().unwrap(),
                       format!("{}:{}:{}",
                               frame.language,
                               frame.short_description,
                               frame.actual_text))
                },
                FrameBody::PIC(frame) => {
                    assert_eq!(data.pop().unwrap(),
                       format!("{}:{:?}:{}:{}",
                               frame.image_format,
                               frame.picture_type,
                               frame.description,
                               frame.picture_data.len()))
                },
                FrameBody::APIC(frame) => {
                    assert_eq!(data.pop().unwrap(),
                            format!("{}{:?}{}{}",
                                    frame.mime_type,
                                    frame.picture_type,
                                    frame.description,
                                    frame.picture_data.len()))
                },
                FrameBody::TXXX(frame) => {
                    assert_eq!(data.pop().unwrap(), 
                            format!("{}:{}", 
                                    frame.description, 
                                    frame.value))
                },

                $(  FrameBody::$id(frame) => assert_eq!(data.pop().unwrap(), frame.text) ),*

                , 
                _ => ()
            }

            data.reverse();

        }

        fn compare_frame_bytes(readable: &mut Cursor<Vec<u8>>, frame_body: FrameBody, data: &mut Vec<&str>) {

            match frame_body {

                FrameBody::COMM(_) => compare_frame(
                                            FrameBody::COMM(COMM::read(readable, 0, "").unwrap()), 
                                            data),

                FrameBody::PIC(_) => compare_frame(
                                            FrameBody::PIC(PIC::read(readable, 0, "").unwrap()), 
                                            data),

                FrameBody::APIC(_) => compare_frame(
                                            FrameBody::APIC(APIC::read(readable, 0, "").unwrap()), 
                                            data),

                FrameBody::TXXX(_) => compare_frame(
                                            FrameBody::TXXX(TXXX::read(readable, 0, "").unwrap()), 
                                            data),

                $( 
                    FrameBody::$id(_) => compare_frame(
                                            FrameBody::$id(TEXT::read(readable, 0, id::$id).unwrap()),
                                            data)
                ),*

                ,
                _ => (),
            }
        }

    )
}

define_compare_frame!(
    TALB, TBPM, TCOM, TCON, TCOP, TDAT, TDEN, TDLY, TDOR, TDRC,
    TDRL, TDTG, TENC, TEXT, TIME, TFLT, TIPL, TIT1, TIT2, TIT3,
    TKEY, TLAN, TLEN, TMCL, TMED, TMOO, TOAL, TOFN, TOLY, TOPE,
    TORY, TOWN, TPE1, TPE2, TPE3, TPE4, TPOS, TPRO, TPUB, TRCK,
    TRDA, TRSN, TSIZ, TRSO, TSOA, TSOP, TSOT, TSRC, TSSE, TYER,
    TSST
);