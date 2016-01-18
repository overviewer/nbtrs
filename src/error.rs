use std::io;
use std::convert::From;
use std::string;
use byteorder;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    BadEncoding(string::FromUtf8Error),
    UnexpectedEOF,
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
    fn from(err: string::FromUtf8Error) -> Error { Error::BadEncoding(err) }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error { Error::Io(err) }
}
