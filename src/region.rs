use std::io::{Read, Seek,Cursor, SeekFrom};
use byteorder;
use byteorder::{BigEndian, ReadBytesExt};
use ::nbt;
use flate2;
use std::convert::From;
use std::error;
use std::fmt;

pub struct Region<T>
where T: Read + Seek {
    /// Offsets (in bytes, from the beginning of the file) of each chunk.  
    ///
    /// An offset of zero means the chunk does not exist
    offsets: Vec<u32>,

    /// Timestamps, indexed by chunk.  If the chunk doesn't exist, the value will be zero
    timestamps: Vec<u32>,

    /// Size of each chunk, in number of 4096-byte sectors
    chunk_size: Vec<u8>,

    cursor: Box<T>
}

#[derive(Debug)]
pub struct RegionError {
    msg: &'static str,
    cause: Option<Box<error::Error>>
}

impl ::std::error::Error for RegionError {
    fn description(&self) -> &str { self.msg }
    fn cause(&self) -> Option<&error::Error> { 
        if let Some(ref cause) = self.cause {
            Some(cause.as_ref())
        } else { None }
    }
}

impl fmt::Display for RegionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if let Some(ref cause) = self.cause {
            writeln!(f, "RegionError: {}: {}", self.msg, cause.description())
        } else {
            writeln!(f, "RegionError: {}", self.msg)
        }
    }
}

impl From<byteorder::Error> for RegionError {
    fn from(e: byteorder::Error) -> RegionError {
        RegionError {msg: "Failed to read bytes", cause: Some(Box::new(e))}
    }
}

impl From<::std::io::Error> for RegionError {
    fn from(e: ::std::io::Error) -> RegionError {
        RegionError {msg: "IO Failure", cause: Some(Box::new(e))}
    }
}


impl<R> Region<R>
where R: Read + Seek {
    pub fn new(mut r: R) -> Result<Region<R>, RegionError> {


        let mut offsets  = Vec::with_capacity(1024);
        let mut timestamps = Vec::with_capacity(1024);
        let mut chunk_size = Vec::with_capacity(1024);

        for _ in 0..1024 {
            let v = try!(r.read_u32::<BigEndian>());

            // upper 3 bytes are an offset
            let offset = v  >> 8;
            let sector_count = (v & 0xff) as u8;

            offsets.push(offset * 4096);
            chunk_size.push(sector_count);

        }

        for _ in 0..1024 {
            let ts = try!(r.read_u32::<BigEndian>());
            timestamps.push(ts);
        }

        Ok(Region{
            offsets: offsets,
            timestamps: timestamps,
            chunk_size: chunk_size,
            cursor: Box::new(r)
        })
    }

    pub fn get_chunk_timestamp(&self, x: u32, z: u32) -> Option<u32> {
        let idx = (x%32 + (z%32) *32 ) as usize;
        if idx < self.timestamps.len() {
            Some(self.timestamps[idx])
        } else {
            None
        }
    }

    fn get_chunk_offset(&self, x: u32, z: u32) -> u32 {
        let idx = (x%32 + (z%32) *32 ) as usize;
        self.offsets[idx]
    }

    pub fn chunk_exists(&self, x: u32, z: u32) -> bool {
        let idx = (x%32 + (z%32) *32 ) as usize;
        self.offsets.get(idx).map_or(false, |v| *v > 0)
    }

    pub fn load_chunk(&mut self, x: u32, z: u32) -> Result<nbt::Tag, RegionError> {
        let idx = (x%32 + (z%32) *32 ) as usize;
        
        let offset = self.get_chunk_offset(x, z);

        try!(self.cursor.seek(SeekFrom::Start(offset as u64)));
        let total_len = try!(self.cursor.read_u32::<BigEndian>()) as usize;
        let compression_type = try!(self.cursor.read_u8());
       
        println!("Compresion type: {:?}", compression_type);
        if compression_type != 2 { panic!("Compression types other than zlib are not supported right now"); }

        let compressed_data = {
            let mut v: Vec<u8> = Vec::with_capacity(total_len- 1);
            v.resize(total_len-1, 0);
            self.cursor.read_exact(&mut v);
            v
        };

        let mut decoder = flate2::read::ZlibDecoder::new(Cursor::new(compressed_data));

        let (_, tag) = nbt::Tag::parse_file(&mut decoder).unwrap();
        Ok(tag)

    }
}


#[test]
fn test_region() {
    // The values used in the assertions in this test were gotten from the nbt.py impl in
    // Minecraft-Overviewer
    use std::fs::File;
    
    let f = File::open("tests/data/r.0.0.mca").unwrap();
    let mut region = Region::new(f).unwrap();

    let ts = region.get_chunk_timestamp(0, 0).unwrap();
    assert_eq!(ts, 1383443712);
    
    let ts = region.get_chunk_timestamp(13, 23).unwrap();
    assert_eq!(ts, 0);
    
    let ts = region.get_chunk_timestamp(14, 10).unwrap();
    assert_eq!(ts, 1383443713);


    assert!(region.chunk_exists(14, 10));
    assert!(! region.chunk_exists(15, 15));

    assert_eq!(region.get_chunk_offset(0, 0), 180224);

    let tag = region.load_chunk(0, 0).unwrap();
    tag.pretty_print(0, None);
}
