#[macro_use]
extern crate log;
extern crate env_logger;
extern crate tempdir;

extern crate rtag;

use tempdir::TempDir;

use std::fs;
use std::io::Cursor;
use std::vec::Vec;

use rtag::frame::*;
use rtag::metadata::*;
use rtag::readable::{Readable, ReadableFactory};

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

    let id3v1_tag = concat!("TAGTITLETITLETITLETITLETITLETITLE",
                            "ARTISTARTISTARTISTARTISTARTIST",
                            "ALBUMALBUMALBUMALBUMALBUMALBUM",
                            "2017",
                            "COMMENTCOMMENTCOMMENTCOMMENTCO4");

    let mut readable = Cursor::new(id3v1_tag.to_string().into_bytes()).to_readable();
    let frame = Frame1::read(&mut readable).unwrap();
    assert_eq!(frame.title, "TITLETITLETITLETITLETITLETITLE");
    assert_eq!(frame.artist, "ARTISTARTISTARTISTARTISTARTIST");
    assert_eq!(frame.album, "ALBUMALBUMALBUMALBUMALBUMALBUM");
    assert_eq!(frame.comment, "COMMENTCOMMENTCOMMENTCOMMENTCO");
    assert_eq!(frame.year, "2017");

    let id3v1_tag = concat!("TAGTITLE                         ",
                            "ARTIST                        ",
                            "ALBUM                         ",
                            "2017",
                            "COMMENT                        ");

    let mut readable = Cursor::new(id3v1_tag.to_string().into_bytes()).to_readable();
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
                let mut readable = Readable::new(Cursor::new(bytes));
                assert_eq!(header, Head::read(&mut readable).unwrap());
            }
            _ => (),
        }
    }

    for m in MetadataReader::new("./test-resources/240.mp3").unwrap() {
        match m {
            Unit::Header(header) => {
                let writer = MetadataWriter::new("").unwrap();
                let bytes = writer.head(header.clone()).unwrap();
                let mut readable = Readable::new(Cursor::new(bytes));
                assert_eq!(header, Head::read(&mut readable).unwrap());
            }
            _ => (),
        }
    }
}

#[test]
fn metadata_frame_data() {
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
                    let mut readable = Cursor::new(frame_bytes).to_readable();
                    match &head {
                        &FrameHeader::V22(_) => {
                            FrameHeaderV2::read(&mut readable).unwrap();
                        }
                        &FrameHeader::V23(_) => {
                            FrameHeaderV3::read(&mut readable).unwrap();
                        }
                        &FrameHeader::V24(_) => {
                            FrameHeaderV4::read(&mut readable).unwrap();
                        }
                    };
                    compare_frame_bytes(&mut readable, frame, &mut copied);
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
         vec!["Test v2.2.0", "Pudge", "2", "1998", "(37)", "eng::All Rights Reserved"]);
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

                let mut readable = Cursor::new(frame_bytes).to_readable();

                assert_eq!(head,
                           FrameHeader::V23(FrameHeaderV3::read(&mut readable).unwrap()));
                assert_eq!(frame, ETCO::read(&mut readable).unwrap());
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

                let mut readable = Cursor::new(frame_bytes).to_readable();

                assert_eq!(head,
                           FrameHeader::V24(FrameHeaderV4::read(&mut readable).unwrap()));
                assert_eq!(frame, PCNT::read(&mut readable).unwrap());
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

                let mut readable = Cursor::new(frame_bytes).to_readable();

                let origin_frame_header = FrameHeader::V23(FrameHeaderV3 {
                    id: id::TBPM_STR.to_string(),
                    size: 5,
                    status_flag: 0,
                    encoding_flag: 0,
                });
                let new_frame_header = FrameHeader::V23(FrameHeaderV3::read(&mut readable)
                    .unwrap());

                assert_eq!(origin_frame_header, new_frame_header);
                assert_eq!(frame, TEXT::read(&mut readable, id::TBPM_STR).unwrap());
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

                let mut readable = Cursor::new(frame_bytes).to_readable();

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

                let mut readable = Cursor::new(frame_bytes).to_readable();

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

                let mut readable = Cursor::new(frame_bytes).to_readable();

                let _ = readable.skip(10);

                assert_eq!(frame, TEXT::read(&mut readable, id::TPE1_STR).unwrap());
            }
            Unit::FrameV2(head, FrameBody::TALB(frame)) => {
                assert_eq!("Ester Koèièková a Lubomír Nohavica s klavírem",
                           frame.text);

                let meta_writer = MetadataWriter::new("").unwrap();
                let frame_bytes = meta_writer.frame((head, FrameBody::TALB(frame.clone())))
                    .unwrap();

                let mut readable = Cursor::new(frame_bytes).to_readable();

                let _ = readable.skip(10);

                assert_eq!(frame, TEXT::read(&mut readable, id::TPE1_STR).unwrap());
            }
            Unit::FrameV2(head, FrameBody::TIT2(frame)) => {
                assert_eq!("Tøem sestrám", frame.text);

                let meta_writer = MetadataWriter::new("").unwrap();
                let frame_bytes = meta_writer.frame((head, FrameBody::TIT2(frame.clone())))
                    .unwrap();

                let mut readable = Cursor::new(frame_bytes).to_readable();

                let _ = readable.skip(10);

                assert_eq!(frame, TEXT::read(&mut readable, id::TPE1_STR).unwrap());
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

                let mut readable = Cursor::new(frame_bytes).to_readable();

                let _ = readable.skip(6);

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
    let mut i = MetadataReader::new(path).unwrap().filter(|m| match m {
        &Unit::FrameV2(FrameHeader::V23(ref header), _) => {
            header.has_flag(FrameHeaderFlag::Compression)
        }
        _ => false,
    });

    if let Unit::FrameV2(origin_head, FrameBody::TIT2(origin_frame)) = i.next().unwrap() {
        assert_eq!("Compressed TIT2 Frame", origin_frame.text);

        let frame_bytes = MetadataWriter::new("")
            .unwrap()
            .frame((origin_head.clone(), FrameBody::TIT2(origin_frame.clone())))
            .unwrap();

        let mut readable = Cursor::new(frame_bytes).to_readable();

        if let Unit::FrameV2(new_frame_header, new_frame_data) =
            MetadataReader::new(path)
                .unwrap()
                .frame3(&mut readable)
                .unwrap() {
            assert_eq!(origin_head, new_frame_header);
            assert_eq!(FrameBody::TIT2(origin_frame), new_frame_data);
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }
    assert!(i.next().is_none());

    let path = "./test-resources/v2.4-compressed-frame.mp3";
    let mut i = MetadataReader::new(path).unwrap().filter(|m| match m {
        &Unit::FrameV2(FrameHeader::V24(ref header), _) => {
            header.has_flag(FrameHeaderFlag::Compression)
        }
        _ => false,
    });

    if let Unit::FrameV2(origin_frame_header, FrameBody::TIT2(origin_frame)) = i.next().unwrap() {
        assert_eq!("Compressed TIT2 Frame", origin_frame.text);

        let frame_bytes = MetadataWriter::new("")
            .unwrap()
            .frame((origin_frame_header.clone(), FrameBody::TIT2(origin_frame.clone())))
            .unwrap();

        let mut readable = Cursor::new(frame_bytes).to_readable();

        if let Unit::FrameV2(new_frame_header, new_frame_data) =
            MetadataReader::new(path)
                .unwrap()
                .frame4(&mut readable)
                .unwrap() {
            assert_eq!(origin_frame_header, new_frame_header);
            assert_eq!(FrameBody::TIT2(origin_frame), new_frame_data);
        } else {
            assert!(false);
        }
    } else {
        assert!(false);
    }

    assert!(i.next().is_none());
}

#[test]
fn metadata_encrypted() {
    let _ = env_logger::init();

    let path = "./test-resources/v2.3-encrypted-frame.mp3";
    let mut i = MetadataReader::new(path).unwrap().filter(|m| match m {
        &Unit::FrameV2(FrameHeader::V23(ref head), _) => head.has_flag(FrameHeaderFlag::Encryption),
        _ => false,
    });

    if let Unit::FrameV2(orig_frame_header, FrameBody::SKIP(_, orig_frame)) = i.next().unwrap() {
        assert!(true);

        let frame_header = if let FrameHeader::V23(frame_header) = orig_frame_header.clone() {
            Some(frame_header)
        } else {
            None
        };

        let frame_bytes = MetadataWriter::new("")
            .unwrap()
            .frame3(&mut frame_header.unwrap(),
                    FrameBody::OBJECT(OBJECT { data: orig_frame.clone() }))
            .unwrap();

        let mut readable = Cursor::new(frame_bytes).to_readable();

        let meta_reader = MetadataReader::new(path).unwrap().frame3(&mut readable).unwrap();
        if let Unit::FrameV2(new_frame_header, FrameBody::SKIP(_, new_frame)) = meta_reader {
            assert_eq!(orig_frame_header, new_frame_header);
            assert_eq!(orig_frame, new_frame);
        }
    } else {
        assert!(false);
    }

    assert!(i.next().is_none());

    let path = "./test-resources/v2.4-encrypted-frame.mp3";
    let mut i = MetadataReader::new(path).unwrap().filter(|m| match m {
        &Unit::FrameV2(FrameHeader::V24(ref head), _) => head.has_flag(FrameHeaderFlag::Encryption),
        _ => false,
    });

    if let Unit::FrameV2(orig_frame_header, FrameBody::SKIP(_, orig_frame)) = i.next().unwrap() {
        assert!(true);

        let frame_header = if let FrameHeader::V24(frame_header) = orig_frame_header.clone() {
            Some(frame_header)
        } else {
            None
        };

        let frame_bytes = MetadataWriter::new("")
            .unwrap()
            .frame4(&mut frame_header.unwrap(),
                    FrameBody::OBJECT(OBJECT { data: orig_frame.clone() }))
            .unwrap();

        let mut readable = Cursor::new(frame_bytes).to_readable();

        let meta_reader = MetadataReader::new(path).unwrap().frame4(&mut readable).unwrap();
        if let Unit::FrameV2(new_frame_header, FrameBody::SKIP(_, new_frame)) = meta_reader {
            assert_eq!(orig_frame_header, new_frame_header);
            assert_eq!(orig_frame, new_frame);
        }
    } else {
        assert!(false);
    }

    assert!(i.next().is_none());
}

#[test]
fn metadata_v230_ext_header() {
    let _ = env_logger::init();

    // file with extend header bit set but no extended header
    {
        let path = "./test-resources/v2.3-ext-header-invalid.mp3";
        let i = MetadataReader::new(path).unwrap().filter(|m| match m {
            &Unit::Header(ref header) => header.has_flag(HeadFlag::ExtendedHeader),
            _ => false,
        });

        assert!(i.count() == 1);

        let i = MetadataReader::new(path).unwrap().filter(|m| match m {
            &Unit::ExtendedHeader(_) => true,
            _ => false,
        });

        assert!(i.count() == 0);
    }

    {
        let path = "./test-resources/v2.3-ext-header.mp3";
        let i = MetadataReader::new(path).unwrap().filter(|m| match m {
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

                let frame_header = if let FrameHeader::V23(fh) = orig_frame_header.clone() {
                    Some(fh)
                } else {
                    None
                };

                let frame_bytes = MetadataWriter::new("")
                    .unwrap()
                    .frame3(&mut frame_header.unwrap(),
                            FrameBody::LINK(orig_frame.clone()))
                    .unwrap();

                let mut readable = Cursor::new(frame_bytes).to_readable();

                let meta_reader = MetadataReader::new(path).unwrap().frame3(&mut readable).unwrap();
                if let Unit::FrameV2(_, FrameBody::LINK(new_frame)) = meta_reader {
                    assert_eq!(orig_frame, new_frame);
                }
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

                let frame_header = if let FrameHeader::V23(fh) = orig_frame_header.clone() {
                    Some(fh)
                } else {
                    None
                };

                let frame_bytes = MetadataWriter::new("")
                    .unwrap()
                    .frame3(&mut frame_header.unwrap(),
                            FrameBody::MCDI(orig_frame.clone()))
                    .unwrap();

                let mut readable = Cursor::new(frame_bytes).to_readable();

                let meta_reader = MetadataReader::new(path).unwrap().frame3(&mut readable).unwrap();
                if let Unit::FrameV2(new_frame_header, FrameBody::MCDI(new_frame)) = meta_reader {
                    assert_eq!(orig_frame_header, new_frame_header);
                    assert_eq!(orig_frame, new_frame);
                }
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

    if let Unit::FrameV2(orig_frame_header, FrameBody::GEOB(orig_frame)) = i.next().unwrap() {
        assert_eq!("text/plain", orig_frame.mime_type);
        assert_eq!("eyeD3.txt", orig_frame.filename);
        assert_eq!("eyeD3 --help output", orig_frame.content_description);
        assert_eq!(6207, orig_frame.encapsulation_object.len());

        let frame_header = if let FrameHeader::V24(fh) = orig_frame_header.clone() {
            Some(fh)
        } else {
            None
        };

        let frame_bytes = MetadataWriter::new("")
            .unwrap()
            .frame4(&mut frame_header.unwrap(),
                    FrameBody::GEOB(orig_frame.clone()))
            .unwrap();

        let mut readable = Cursor::new(frame_bytes).to_readable();

        let meta_reader = MetadataReader::new(path).unwrap().frame4(&mut readable).unwrap();
        if let Unit::FrameV2(new_frame_header, FrameBody::GEOB(new_frame)) = meta_reader {
            assert_eq!(orig_frame_header, new_frame_header);
            assert_eq!(orig_frame, new_frame);
        }
    }

    if let Unit::FrameV2(orig_frame_header, FrameBody::GEOB(orig_frame)) = i.next().unwrap() {
        assert_eq!("text/plain", orig_frame.mime_type);
        assert_eq!("genres.txt", orig_frame.filename);
        assert_eq!("eyeD3 --list-genres output", orig_frame.content_description);
        assert_eq!(4087, orig_frame.encapsulation_object.len());

        let frame_header = if let FrameHeader::V24(fh) = orig_frame_header.clone() {
            Some(fh)
        } else {
            None
        };

        let frame_bytes = MetadataWriter::new("")
            .unwrap()
            .frame4(&mut frame_header.unwrap(),
                    FrameBody::GEOB(orig_frame.clone()))
            .unwrap();

        let mut readable = Cursor::new(frame_bytes).to_readable();

        let meta_reader = MetadataReader::new(path).unwrap().frame4(&mut readable).unwrap();
        if let Unit::FrameV2(new_frame_header, FrameBody::GEOB(new_frame)) = meta_reader {
            assert_eq!(orig_frame_header, new_frame_header);
            assert_eq!(orig_frame, new_frame);
        }
    }
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

        MetadataWriter::new(path)
            .unwrap()
            .write(meta_reader)
            .unwrap();

        let mut i = MetadataReader::new(path).unwrap().filter(|m| match m {
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
            if let Unit::FrameV2(frame_head, frame_data) = unit {
                let new_frame_data = if let FrameBody::TALB(ref frame) = frame_data {
                    let mut new_frame = frame.clone();
                    new_frame.text = "Album!".to_string();
                    FrameBody::TALB(new_frame)
                } else {
                    frame_data.clone()
                };

                vec.push(Unit::FrameV2(frame_head, new_frame_data));
            } else {
                vec.push(unit);
            }

            vec
        });

    let writer = MetadataWriter::new(path).unwrap();
    let _ = writer.write(new_data);

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

}

fn compare_frame(frame_data: FrameBody, data: &mut Vec<&str>) {
    data.reverse();
    match frame_data {
        FrameBody::COMM(frame) => {
            assert_eq!(data.pop().unwrap(),
                       format!("{}:{}:{}",
                               frame.language,
                               frame.short_description,
                               frame.actual_text))
        }
        FrameBody::PIC(frame) => {
            assert_eq!(data.pop().unwrap(),
                       format!("{}:{:?}:{}:{}",
                               frame.image_format,
                               frame.picture_type,
                               frame.description,
                               frame.picture_data.len()))
        }
        FrameBody::APIC(frame) => {
            assert_eq!(data.pop().unwrap(),
                       format!("{}:{:?}:{}:{}",
                               frame.mime_type,
                               frame.picture_type,
                               frame.description,
                               frame.picture_data.len()))
        }
        FrameBody::TALB(frame) |
        FrameBody::TBPM(frame) |
        FrameBody::TCOM(frame) |
        FrameBody::TCON(frame) |
        FrameBody::TCOP(frame) |
        FrameBody::TDAT(frame) |
        FrameBody::TDEN(frame) |
        FrameBody::TDLY(frame) |
        FrameBody::TDOR(frame) |
        FrameBody::TDRC(frame) |
        FrameBody::TDRL(frame) |
        FrameBody::TDTG(frame) |
        FrameBody::TENC(frame) |
        FrameBody::TEXT(frame) |
        FrameBody::TIME(frame) |
        FrameBody::TFLT(frame) |
        FrameBody::TIPL(frame) |
        FrameBody::TIT1(frame) |
        FrameBody::TIT2(frame) |
        FrameBody::TIT3(frame) |
        FrameBody::TKEY(frame) |
        FrameBody::TLAN(frame) |
        FrameBody::TLEN(frame) |
        FrameBody::TMCL(frame) |
        FrameBody::TMED(frame) |
        FrameBody::TMOO(frame) |
        FrameBody::TOAL(frame) |
        FrameBody::TOFN(frame) |
        FrameBody::TOLY(frame) |
        FrameBody::TOPE(frame) |
        FrameBody::TORY(frame) |
        FrameBody::TOWN(frame) |
        FrameBody::TPE1(frame) |
        FrameBody::TPE2(frame) |
        FrameBody::TPE3(frame) |
        FrameBody::TPE4(frame) |
        FrameBody::TPOS(frame) |
        FrameBody::TPRO(frame) |
        FrameBody::TPUB(frame) |
        FrameBody::TRCK(frame) |
        FrameBody::TRDA(frame) |
        FrameBody::TRSN(frame) |
        FrameBody::TSIZ(frame) |
        FrameBody::TRSO(frame) |
        FrameBody::TSOA(frame) |
        FrameBody::TSOP(frame) |
        FrameBody::TSOT(frame) |
        FrameBody::TSRC(frame) |
        FrameBody::TSSE(frame) |
        FrameBody::TYER(frame) |
        FrameBody::TSST(frame) => assert_eq!(data.pop().unwrap(), frame.text),
        FrameBody::TXXX(frame) => {
            assert_eq!(data.pop().unwrap(),
                       format!("{}:{}", frame.description, frame.value))
        }
        _ => (),
    }
    data.reverse();
}

fn compare_frame_bytes(readable: &mut Readable<Cursor<Vec<u8>>>,
                       frame_data: FrameBody,
                       data: &mut Vec<&str>) {
    match frame_data {
        FrameBody::COMM(_) => compare_frame(FrameBody::COMM(COMM::read(readable).unwrap()), data),
        FrameBody::PIC(_) => compare_frame(FrameBody::PIC(PIC::read(readable).unwrap()), data),
        FrameBody::APIC(_) => compare_frame(FrameBody::APIC(APIC::read(readable).unwrap()), data),
        FrameBody::TALB(_) => {
            compare_frame(FrameBody::TALB(TEXT::read(readable, id::TALB_STR).unwrap()),
                          data)
        }
        FrameBody::TBPM(_) => {
            compare_frame(FrameBody::TBPM(TEXT::read(readable, id::TBPM_STR).unwrap()),
                          data)
        }
        FrameBody::TCOM(_) => {
            compare_frame(FrameBody::TCOM(TEXT::read(readable, id::TCOM_STR).unwrap()),
                          data)
        }
        FrameBody::TCON(_) => {
            compare_frame(FrameBody::TCON(TEXT::read(readable, id::TCON_STR).unwrap()),
                          data)
        }
        FrameBody::TCOP(_) => {
            compare_frame(FrameBody::TCOP(TEXT::read(readable, id::TCOP_STR).unwrap()),
                          data)
        }
        FrameBody::TDAT(_) => {
            compare_frame(FrameBody::TDAT(TEXT::read(readable, id::TDAT_STR).unwrap()),
                          data)
        }
        FrameBody::TDEN(_) => {
            compare_frame(FrameBody::TDEN(TEXT::read(readable, id::TDEN_STR).unwrap()),
                          data)
        }
        FrameBody::TDLY(_) => {
            compare_frame(FrameBody::TDLY(TEXT::read(readable, id::TDLY_STR).unwrap()),
                          data)
        }
        FrameBody::TDOR(_) => {
            compare_frame(FrameBody::TDOR(TEXT::read(readable, id::TDOR_STR).unwrap()),
                          data)
        }
        FrameBody::TDRC(_) => {
            compare_frame(FrameBody::TDRC(TEXT::read(readable, id::TDRC_STR).unwrap()),
                          data)
        }
        FrameBody::TDRL(_) => {
            compare_frame(FrameBody::TDRL(TEXT::read(readable, id::TDRL_STR).unwrap()),
                          data)
        }
        FrameBody::TDTG(_) => {
            compare_frame(FrameBody::TDTG(TEXT::read(readable, id::TDTG_STR).unwrap()),
                          data)
        }
        FrameBody::TENC(_) => {
            compare_frame(FrameBody::TENC(TEXT::read(readable, id::TENC_STR).unwrap()),
                          data)
        }
        FrameBody::TEXT(_) => {
            compare_frame(FrameBody::TEXT(TEXT::read(readable, id::TEXT_STR).unwrap()),
                          data)
        }
        FrameBody::TIME(_) => {
            compare_frame(FrameBody::TIME(TEXT::read(readable, id::TIME_STR).unwrap()),
                          data)
        }
        FrameBody::TFLT(_) => {
            compare_frame(FrameBody::TFLT(TEXT::read(readable, id::TFLT_STR).unwrap()),
                          data)
        }
        FrameBody::TIPL(_) => {
            compare_frame(FrameBody::TIPL(TEXT::read(readable, id::TIPL_STR).unwrap()),
                          data)
        }
        FrameBody::TIT1(_) => {
            compare_frame(FrameBody::TIT1(TEXT::read(readable, id::TIT1_STR).unwrap()),
                          data)
        }
        FrameBody::TIT2(_) => {
            compare_frame(FrameBody::TIT2(TEXT::read(readable, id::TIT2_STR).unwrap()),
                          data)
        }
        FrameBody::TIT3(_) => {
            compare_frame(FrameBody::TIT3(TEXT::read(readable, id::TIT3_STR).unwrap()),
                          data)
        }
        FrameBody::TKEY(_) => {
            compare_frame(FrameBody::TKEY(TEXT::read(readable, id::TKEY_STR).unwrap()),
                          data)
        }
        FrameBody::TLAN(_) => {
            compare_frame(FrameBody::TLAN(TEXT::read(readable, id::TLAN_STR).unwrap()),
                          data)
        }
        FrameBody::TLEN(_) => {
            compare_frame(FrameBody::TLEN(TEXT::read(readable, id::TLEN_STR).unwrap()),
                          data)
        }
        FrameBody::TMCL(_) => {
            compare_frame(FrameBody::TMCL(TEXT::read(readable, id::TMCL_STR).unwrap()),
                          data)
        }
        FrameBody::TMED(_) => {
            compare_frame(FrameBody::TMED(TEXT::read(readable, id::TMED_STR).unwrap()),
                          data)
        }
        FrameBody::TMOO(_) => {
            compare_frame(FrameBody::TMOO(TEXT::read(readable, id::TMOO_STR).unwrap()),
                          data)
        }
        FrameBody::TOAL(_) => {
            compare_frame(FrameBody::TOAL(TEXT::read(readable, id::TOAL_STR).unwrap()),
                          data)
        }
        FrameBody::TOFN(_) => {
            compare_frame(FrameBody::TOFN(TEXT::read(readable, id::TOFN_STR).unwrap()),
                          data)
        }
        FrameBody::TOLY(_) => {
            compare_frame(FrameBody::TOLY(TEXT::read(readable, id::TOLY_STR).unwrap()),
                          data)
        }
        FrameBody::TOPE(_) => {
            compare_frame(FrameBody::TOPE(TEXT::read(readable, id::TOPE_STR).unwrap()),
                          data)
        }
        FrameBody::TORY(_) => {
            compare_frame(FrameBody::TORY(TEXT::read(readable, id::TORY_STR).unwrap()),
                          data)
        }
        FrameBody::TOWN(_) => {
            compare_frame(FrameBody::TOWN(TEXT::read(readable, id::TOWN_STR).unwrap()),
                          data)
        }
        FrameBody::TPE1(_) => {
            compare_frame(FrameBody::TPE1(TEXT::read(readable, id::TPE1_STR).unwrap()),
                          data)
        }
        FrameBody::TPE2(_) => {
            compare_frame(FrameBody::TPE2(TEXT::read(readable, id::TPE2_STR).unwrap()),
                          data)
        }
        FrameBody::TPE3(_) => {
            compare_frame(FrameBody::TPE3(TEXT::read(readable, id::TPE3_STR).unwrap()),
                          data)
        }
        FrameBody::TPE4(_) => {
            compare_frame(FrameBody::TPE4(TEXT::read(readable, id::TPE4_STR).unwrap()),
                          data)
        }
        FrameBody::TPOS(_) => {
            compare_frame(FrameBody::TPOS(TEXT::read(readable, id::TPOS_STR).unwrap()),
                          data)
        }
        FrameBody::TPRO(_) => {
            compare_frame(FrameBody::TPRO(TEXT::read(readable, id::TPRO_STR).unwrap()),
                          data)
        }
        FrameBody::TPUB(_) => {
            compare_frame(FrameBody::TPUB(TEXT::read(readable, id::TPUB_STR).unwrap()),
                          data)
        }
        FrameBody::TRCK(_) => {
            compare_frame(FrameBody::TRCK(TEXT::read(readable, id::TRCK_STR).unwrap()),
                          data)
        }
        FrameBody::TRDA(_) => {
            compare_frame(FrameBody::TRDA(TEXT::read(readable, id::TRDA_STR).unwrap()),
                          data)
        }
        FrameBody::TRSN(_) => {
            compare_frame(FrameBody::TRSN(TEXT::read(readable, id::TRSN_STR).unwrap()),
                          data)
        }
        FrameBody::TSIZ(_) => {
            compare_frame(FrameBody::TSIZ(TEXT::read(readable, id::TSIZ_STR).unwrap()),
                          data)
        }
        FrameBody::TRSO(_) => {
            compare_frame(FrameBody::TRSO(TEXT::read(readable, id::TRSO_STR).unwrap()),
                          data)
        }
        FrameBody::TSOA(_) => {
            compare_frame(FrameBody::TSOA(TEXT::read(readable, id::TSOA_STR).unwrap()),
                          data)
        }
        FrameBody::TSOP(_) => {
            compare_frame(FrameBody::TSOP(TEXT::read(readable, id::TSOP_STR).unwrap()),
                          data)
        }
        FrameBody::TSOT(_) => {
            compare_frame(FrameBody::TSOT(TEXT::read(readable, id::TSOT_STR).unwrap()),
                          data)
        }
        FrameBody::TSRC(_) => {
            compare_frame(FrameBody::TSRC(TEXT::read(readable, id::TSRC_STR).unwrap()),
                          data)
        }
        FrameBody::TSSE(_) => {
            compare_frame(FrameBody::TSSE(TEXT::read(readable, id::TSSE_STR).unwrap()),
                          data)
        }
        FrameBody::TYER(_) => {
            compare_frame(FrameBody::TYER(TEXT::read(readable, id::TYER_STR).unwrap()),
                          data)
        }
        FrameBody::TSST(_) => {
            compare_frame(FrameBody::TSST(TEXT::read(readable, id::TSST_STR).unwrap()),
                          data)
        }
        FrameBody::TXXX(_) => compare_frame(FrameBody::TXXX(TXXX::read(readable).unwrap()), data),
        _ => (),
    }
}
