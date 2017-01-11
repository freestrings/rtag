//MIT License
//
//Copyright (c) [2017] [Mark Han]
//
//Permission is hereby granted, free of charge, to any person obtaining a copy
//of this software and associated documentation files (the "Software"), to deal
//in the Software without restriction, including without limitation the rights
//to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//copies of the Software, and to permit persons to whom the Software is
//furnished to do so, subject to the following conditions:
//
//The above copyright notice and this permission notice shall be included in all
//copies or substantial portions of the Software.
//
//THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//SOFTWARE.

mod bytes;
mod reader;
mod tag;

#[cfg(test)]
mod tests {
    extern crate env_logger;

    use readable;
    use std::vec;
    use super::reader::FrameIterator;

    #[test]
    fn idv3_230_header() {
        let _ = env_logger::init();
        let mut readable = readable::factory::from_path("./resources/230.mp3").unwrap();
        let bytes = readable.as_bytes(10).unwrap();
        let header = super::tag::header::Header::new(bytes);
        assert_eq!(header.get_version(), 3);
        assert_eq!(header.get_minor_version(), 0);
        assert_eq!(header.has_flag(super::tag::header::HeaderFlag::Unsynchronisation), false);
        assert_eq!(header.has_flag(super::tag::header::HeaderFlag::ExtendedHeader), false);
        assert_eq!(header.has_flag(super::tag::header::HeaderFlag::ExperimentalIndicator), false);
        assert_eq!(header.get_size(), 1171);
    }

    #[test]
    fn idv3_240_header() {
        let _ = env_logger::init();

        let mut readable = readable::factory::from_path("./resources/240.mp3").unwrap();
        let bytes = readable.as_bytes(10).unwrap();
        let header = super::tag::header::Header::new(bytes);
        assert_eq!(header.get_version(), 4);
        assert_eq!(header.get_minor_version(), 0);
        assert_eq!(header.has_flag(super::tag::header::HeaderFlag::Unsynchronisation), false);
        assert_eq!(header.has_flag(super::tag::header::HeaderFlag::ExtendedHeader), false);
        assert_eq!(header.has_flag(super::tag::header::HeaderFlag::ExperimentalIndicator), false);
        assert_eq!(header.get_size(), 165126);
    }

    fn _id_compare(file_path: &'static str, mut ids: vec::Vec<&str>) {
        let _ = env_logger::init();

        let mut readable = readable::factory::from_path(file_path).unwrap();
        let mut frame_reader = super::reader::FrameReader::new(&mut readable).unwrap();
        ids.reverse();
        loop {
            if frame_reader.has_next_frame() {
                if let Ok(frame) = frame_reader.next_frame() {
                    assert_eq!(ids.pop().unwrap(), frame.get_id())
                }
            } else {
                break;
            }
        }
    }

    fn _data_compare(file_path: &'static str, mut data: vec::Vec<&str>) {
        let _ = env_logger::init();

        let mut readable = readable::factory::from_path(file_path).unwrap();
        let mut frame_reader = super::reader::FrameReader::new(&mut readable).unwrap();
        data.reverse();
        loop {
            if frame_reader.has_next_frame() {
                if let Ok(frame) = frame_reader.next_frame() {
                    debug!("{}: {:?}", frame.get_id(), frame.get_data());
                    assert_eq!(data.pop().unwrap(), frame.get_data().unwrap())
                }
            } else {
                break;
            }
        }
    }

    #[test]
    fn idv3_230_frame_id() {
        _id_compare("./resources/id3v1-id3v2.mp3", vec!["TIT2", "TPE1", "TALB", "TPE2", "TCON", "COMM", "TRCK", "TPOS"]);
    }

    #[test]
    fn idv3_230_frame_id2() {
        _id_compare("./resources/230.mp3", vec!["TALB", "TCON", "TIT2", "TLEN", "TPE1", "TRCK", "COMM", "TYER"]);
    }

    #[test]
    fn idv3_240_frame_id() {
        _id_compare("./resources/240.mp3", vec!["TDRC", "TRCK", "TPOS", "TPE1", "TALB", "TPE2", "TIT2", "TSRC", "TCON", "COMM"]);
    }

    #[test]
    fn idv3_230_frame_data() {
        _data_compare("./resources/id3v1-id3v2.mp3", vec!["타이틀", "Artist", "アルバム", "Album Artist", "Heavy Metal", "eng\u{0}!@#$", "1", "0"]);
    }

    #[test]
    fn idv3_230_frame_data2() {
        _data_compare("./resources/230.mp3", vec!["\u{feff}앨범", "Rock", "\u{feff}Tㅏi틀", "0", "\u{feff}아티st", "1", "eng\u{0}!!!@@#$@$^#$%^\\n123", "2017"]);
    }

    #[test]
    fn idv3_240_frame_data() {
        _data_compare("./resources/240.mp3", vec!["2017", "1", "1", "아티스트", "Album", "Artist/아티스트", "타이틀", "ABAB", "Alternative", "eng\u{0}~~"]);
    }
}