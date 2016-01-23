extern crate nbtrs;
extern crate flate2;

use nbtrs::Tag;

use std::env::args;
use std::fs::File;
use flate2::read::GzDecoder;

fn load_and_print(s: &str) {
    println!("Dumping... {}", s);
    let f = File::open(s).unwrap();
    let mut decoder = GzDecoder::new(f).unwrap();
    let (name, tag) = Tag::parse(&mut decoder).unwrap();

    tag.pretty_print(0, Some(&name));
}

fn main() {

    for arg in args().skip(1) {
        load_and_print(&arg);
    }

}
