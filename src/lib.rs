//! nbtrs is a crate for parsing (reading) NBT and McRegion files.  There is no write support.
//!
//! # Examples
//!
//! Put some here

extern crate flate2;
extern crate byteorder;

mod error;
mod nbt;
mod region;

pub use error::Error;
pub use nbt::{Tag, Taglike};
pub use region::RegionFile;
