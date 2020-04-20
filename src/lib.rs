//! nbtrs is a crate for parsing (reading) NBT and McRegion files.  There is no write support.
//!
//! # Examples
//!
//! Put some here

mod error;
mod nbt;
mod region;

pub use error::Error;
pub use nbt::{Tag, Taglike};
pub use region::RegionFile;
