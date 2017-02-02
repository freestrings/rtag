use std::{
    error,
    io,
    fmt,
    mem
};

#[derive(Debug)]
pub enum ParsingErrorKind {
    InvalidV1FrameId,
    InvalidV2FrameId,
    InvalidFrameLength,
    EncodingError
}

#[derive(Debug)]
pub enum ParsingError {
    BadData(ParsingErrorKind),
    IoError(io::Error)
}

impl From<ParsingErrorKind> for ParsingError {
    fn from(err: ParsingErrorKind) -> ParsingError {
        ParsingError::BadData(err)
    }
}

impl From<io::Error> for ParsingError {
    fn from(err: io::Error) -> ParsingError {
        ParsingError::IoError(err)
    }
}

impl fmt::Display for ParsingErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParsingErrorKind::InvalidV1FrameId => write!(f, "Only 'TAG' is allowed)"),
            ParsingErrorKind::InvalidV2FrameId => write!(f, "Only 'ID3' is allowed)"),
            ParsingErrorKind::InvalidFrameLength => write!(f, "Invalid frame length"),
            ParsingErrorKind::EncodingError => write!(f, "Invalid text encoding")
        }
    }
}

impl fmt::Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParsingError::BadData(ref kind) => fmt::Display::fmt(kind, f),
            ParsingError::IoError(ref err) => fmt::Display::fmt(err, f)
        }
    }
}

impl error::Error for ParsingError {
    fn description(&self) -> &str {
        match *self {
            ParsingError::BadData(ref kind) => unsafe {
                mem::transmute(&format!("{:?}", kind) as &str)
            },
            ParsingError::IoError(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ParsingError::IoError(ref err) => Some(err as &error::Error),
            _ => None,
        }
    }
}
