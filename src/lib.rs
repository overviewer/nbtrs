#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate byteorder;
extern crate flate2;
extern crate serde;

pub use de::{read, read_named};
pub use value::{Tag};

pub mod de;
pub mod value;
