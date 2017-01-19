#[derive(Debug)]
pub enum ParsingError {
    Id1TagNotFound,
    Id2TagNotFound,
    BadData(String),
    EncodeDecodeError(::std::borrow::Cow<'static, str>),
    IoError(::std::io::Error)
}

impl ::std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            ParsingError::Id1TagNotFound => ::std::fmt::Display::fmt(&ParsingError::Id1TagNotFound, f),
            ParsingError::Id2TagNotFound => ::std::fmt::Display::fmt(&ParsingError::Id2TagNotFound, f),
            ParsingError::BadData(ref err) => ::std::fmt::Display::fmt(err, f),
            ParsingError::EncodeDecodeError(ref err) => ::std::fmt::Display::fmt(err, f),
            ParsingError::IoError(ref err) => ::std::fmt::Display::fmt(err, f)
        }
    }
}

impl From<String> for ParsingError {
    fn from(err: String) -> ParsingError {
        ParsingError::BadData(err)
    }
}

impl From<::std::borrow::Cow<'static, str>> for ParsingError {
    fn from(err: ::std::borrow::Cow<'static, str>) -> ParsingError {
        ParsingError::EncodeDecodeError(err)
    }
}

impl From<::std::io::Error> for ParsingError {
    fn from(err: ::std::io::Error) -> ParsingError {
        ParsingError::IoError(err)
    }
}

impl ::std::error::Error for ParsingError {
    fn description(&self) -> &str {
        match *self {
            ParsingError::Id1TagNotFound => "Not found id1 tag",
            ParsingError::Id2TagNotFound => "Not found id2 tag",
            ParsingError::BadData(ref err) => err.as_str(),
            ParsingError::EncodeDecodeError(ref err) => err,
            ParsingError::IoError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        match *self {
            ParsingError::IoError(ref err) => Some(err as &::std::error::Error),
            _ => None,
        }
    }
}