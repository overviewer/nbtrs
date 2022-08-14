extern crate flate2;
extern crate nbtrs;

use nbtrs::Tag;

use flate2::read::GzDecoder;
use std::env::args;
use std::fs::File;

fn load_and_print(s: &str) {
    println!("Dumping... {}", s);
    let f = File::open(s).unwrap();
    let mut decoder = GzDecoder::new(f);
    let (name, tag) = Tag::parse(&mut decoder).unwrap();

    tag.pretty_print(0, Some(&name));
}

fn main() {
    for arg in args().skip(1) {
        load_and_print(&arg);
    }
}
