use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::error as nbt_error;
use crate::nbt;

/// A region file
///
/// These normally have a .mca extension on disk.  They contain up to 1024 chunks, each containing
/// a 32-by-32 column of blocks.
#[allow(dead_code)]
pub struct RegionFile<T> {
    /// Offsets (in bytes, from the beginning of the file) of each chunk.
    /// An offset of zero means the chunk does not exist
    offsets: Vec<u32>,

    /// Timestamps, indexed by chunk.  If the chunk doesn't exist, the value will be zero
    timestamps: Vec<u32>,

    /// Size of each chunk, in number of 4096-byte sectors
    chunk_size: Vec<u8>,

    cursor: Box<T>,
}

impl<R> RegionFile<R>
where
    R: Read + Seek,
{
    /// Parses a region file
    pub fn new(mut r: R) -> Result<RegionFile<R>, nbt_error::Error> {
        let mut offsets = Vec::with_capacity(1024);
        let mut timestamps = Vec::with_capacity(1024);
        let mut chunk_size = Vec::with_capacity(1024);

        for _ in 0..1024 {
            let v = r.read_u32::<BigEndian>()?;

            // upper 3 bytes are an offset
            let offset = v >> 8;
            let sector_count = (v & 0xff) as u8;

            offsets.push(offset * 4096);
            chunk_size.push(sector_count);
        }

        for _ in 0..1024 {
            let ts = r.read_u32::<BigEndian>()?;
            timestamps.push(ts);
        }

        Ok(RegionFile {
            offsets,
            timestamps,
            chunk_size,
            cursor: Box::new(r),
        })
    }

    /// Returns a unix timestamp of when a given chunk was last modified.  If the chunk does not
    /// exist in this Region, return `None`.
    ///
    /// # Panics
    ///
    /// x and z must be between 0 and 31 (inclusive).  If not, panics.
    pub fn get_chunk_timestamp(&self, x: u8, z: u8) -> Option<u32> {
        assert!(x < 32);
        assert!(z < 32);
        let idx = x as usize % 32 + (z as usize % 32) * 32;
        self.timestamps
            .get(idx)
            .and_then(|&ts| if ts == 0 { None } else { Some(ts) })
    }

    /// Returns the byte-offset for a given chunk (as measured from the start of the file).
    ///
    /// # Panics
    ///
    /// x and z must be between 0 and 31 (inclusive).  If not, panics.
    fn get_chunk_offset(&self, x: u8, z: u8) -> u32 {
        assert!(x < 32);
        assert!(z < 32);
        let idx = x as usize % 32 + (z as usize % 32) * 32;
        self.offsets[idx]
    }

    /// Does the given chunk exist in the Region
    ///
    /// # Panics
    ///
    /// x and z must be between 0 and 31 (inclusive).  If not, panics.
    pub fn chunk_exists(&self, x: u8, z: u8) -> bool {
        assert!(x < 32);
        assert!(z < 32);
        let idx = x as usize % 32 + (z as usize % 32) * 32;
        self.offsets.get(idx).map_or(false, |v| *v > 0)
    }

    /// Loads a chunk into a parsed NBT Tag structure.
    ///
    /// # Panics
    ///
    /// x and z must be between 0 and 31 (inclusive).  If not, panics.
    pub fn load_chunk(&mut self, x: u8, z: u8) -> Result<nbt::Tag, nbt_error::Error> {
        let offset = self.get_chunk_offset(x, z); // might panic

        self.cursor.seek(SeekFrom::Start(offset as u64))?;
        let total_len = self.cursor.read_u32::<BigEndian>()? as usize;
        let compression_type = self.cursor.read_u8()?;

        if compression_type != 2 {
            return Err(nbt_error::Error::UnsupportedCompressionFormat { compression_type });
        }

        let compressed_data = {
            let mut v = vec![0; total_len - 1];
            self.cursor.read_exact(&mut v)?;
            v
        };

        let mut decoder = flate2::read::ZlibDecoder::new(Cursor::new(compressed_data));

        let (_, tag) = nbt::Tag::parse(&mut decoder).unwrap();
        Ok(tag)
    }
}

#[test]
fn test_region() {
    // The values used in the assertions in this test were gotten from the nbt.py impl in
    // Minecraft-Overviewer
    use nbt::Taglike;
    use std::fs::File;

    let f = File::open("tests/data/r.0.0.mca").unwrap();
    let mut region = RegionFile::new(f).unwrap();

    let ts = region.get_chunk_timestamp(0, 0).unwrap();
    assert_eq!(ts, 1383443712);

    let ts = region.get_chunk_timestamp(13, 23);
    assert_eq!(ts, None);
    assert!(!region.chunk_exists(13, 23));

    let ts = region.get_chunk_timestamp(14, 10).unwrap();
    assert_eq!(ts, 1383443713);

    assert!(region.chunk_exists(14, 10));
    assert!(!region.chunk_exists(15, 15));

    assert_eq!(region.get_chunk_offset(0, 0), 180224);

    let tag = region.load_chunk(0, 0).unwrap();
    // tag.pretty_print(0, None);

    let level = tag.key("Level").unwrap();
    let last_update = level.key("LastUpdate").as_i64().unwrap();
    let z_pos = level.key("zPos").as_i32().unwrap();
    assert_eq!(last_update, 137577);
    assert_eq!(z_pos, 0);
    level.pretty_print(0, None);
}
