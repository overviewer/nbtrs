extern crate flate2;
extern crate nbtrs;

use flate2::read::GzDecoder;
use std::fs;
use std::io::Read;
use std::path::Path;

#[test]
fn load_level_dat_flate_test() {
    let level_dat_path = Path::new("tests/data/level.dat");
    let level_dat = fs::File::open(&level_dat_path).unwrap();

    let decoder: GzDecoder<fs::File> = GzDecoder::new(level_dat);

    // check the first 4 bytes
    let bytes: Vec<u8> = decoder.bytes().take(4).map(|x| x.unwrap()).collect();
    assert_eq!(bytes, vec!(0x0a, 0x00, 0x00, 0x0a));
}
