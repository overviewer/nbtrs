extern crate nbtrs;
extern crate flate2;

use nbtrs::{Tag, Taglike};

use std::env::args;
use std::fs::File;
use flate2::read::GzDecoder;

fn load_and_print(s: &str) {
    println!("Dumping... {}", s);
    let f = File::open(s).unwrap();
    let mut decoder = GzDecoder::new(f).unwrap();
    let (name, tag) = Tag::parse(&mut decoder).unwrap();

    let pos = tag.key("Data").key("Player").key("Pos").as_list().unwrap().clone();
    tag.key("Data").key("Player").key("Attributes").unwrap().pretty_print(0, Some("Player"));
}

fn main() {

    for arg in args().skip(1) {
        load_and_print(&arg);
    }

}
