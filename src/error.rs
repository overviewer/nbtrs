use std::io;
use std::convert::From;
use std::string;
use byteorder;

/// Things that can go wrong during NBT or Region parsing
#[derive(Debug)]
pub enum Error {
    /// There was an error during IO
    Io(io::Error),
    BadEncoding(string::FromUtf8Error),
    UnexpectedEOF,

    /// An unexpected tag was found while NBT Parsing
    UnexpectedTag(u8),
}

impl From<byteorder::Error> for Error {
    fn from(err: byteorder::Error) -> Error {
        match err {
            byteorder::Error::UnexpectedEOF => Error::UnexpectedEOF,
            byteorder::Error::Io(e) => From::from(e),
        }
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Error {
        Error::BadEncoding(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}
