use std::io;
use std::convert::From;
use std::string;
use std::error;
use std::fmt;

/// Things that can go wrong during NBT or Region parsing
#[derive(Debug)]
pub enum Error {
    /// There was an error during IO
    Io(io::Error),
    BadEncoding(string::FromUtf8Error),
    /// Currently, only zlib is implemented.
    UnsupportedCompressionFormat{
        /// Compression type byte from the format.
        compression_type: u8
    },
    UnexpectedEOF,

    /// An unexpected tag was found while NBT Parsing
    UnexpectedTag(u8),
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
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::Io(..) => write!(f, "IO Error"),
            Error::BadEncoding(..) => write!(f, "Bad Encoding"),
            Error::UnexpectedEOF => write!(f, "Unexpected EOF"),
            Error::UnexpectedTag(..) => write!(f, "Unexpected Tag"),
            Error::UnsupportedCompressionFormat{ compression_type: _ } => write!(f, "Unsupported Compression"),
        }
    }
}
