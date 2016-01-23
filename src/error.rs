use std::io;
use std::convert::From;
use std::string;
use std::error;
use std::fmt;
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

    UnexpectedType, // when trying to access data of the wrong type on a tag
    InvalidKey(String), // when trying to get a bad key
    InvalidIndex(usize), // when trying to get a bad index
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

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::Io(..) => "IO Error",
            &Error::BadEncoding(..) => "Bad Encoding",
            &Error::UnexpectedEOF => "Unexpected EOF",
            &Error::UnexpectedTag(..) => "Unexpected Tag",
            &Error::UnexpectedType => "Unexpected Type",
            &Error::InvalidKey(..) => "Invalid Key",
            &Error::InvalidIndex(..) => "Invalid Index"
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use std::error::Error;
        write!(f, "NBTError: {}", self.description())
    }
}
