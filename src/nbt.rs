use std::io::Read;
use std::collections::HashMap;
use byteorder::{ReadBytesExt, BigEndian};

use super::error::Error;

/// An NBT Tag
#[derive(Debug, PartialEq, Clone)]
pub enum Tag {
    TagEnd,
    TagByte(i8),
    TagShort(i16),
    TagInt(i32),
    TagLong(i64),
    TagFloat(f32),
    TagDouble(f64),
    TagByteArray(Vec<u8>),
    TagString(String),
    TagList(Vec<Tag>),
    TagCompound(HashMap<String, Tag>),
    TagIntArray(Vec<u32>),
}

// trait to simplify grabbing nested NBT data
pub trait Taglike<'t> : Sized {
    fn map_tag<F, T>(self, f: F) -> Result<T, Error> where F: FnOnce(&'t Tag) -> Result<T, Error>;

    // the rest of these are defaults that work, relying on
    // the implementation for Tag
    fn as_i8(self) -> Result<i8, Error> { self.map_tag(|t| t.as_i8()) }
    fn as_i16(self) -> Result<i16, Error> { self.map_tag(|t| t.as_i16()) }
    fn as_i32(self) -> Result<i32, Error> { self.map_tag(|t| t.as_i32()) }
    fn as_i64(self) -> Result<i64, Error> { self.map_tag(|t| t.as_i64()) }
    fn as_f32(self) -> Result<f32, Error> { self.map_tag(|t| t.as_f32()) }
    fn as_f64(self) -> Result<f64, Error> { self.map_tag(|t| t.as_f64()) }
    fn as_bytes(self) -> Result<&'t Vec<u8>, Error> { self.map_tag(|t| t.as_bytes()) }
    fn as_string(self) -> Result<&'t String, Error> { self.map_tag(|t| t.as_string()) }
    fn as_list(self) -> Result<&'t Vec<Tag>, Error> { self.map_tag(|t| t.as_list()) }
    fn as_map(self) -> Result<&'t HashMap<String, Tag>, Error> { self.map_tag(|t| t.as_map()) }
    fn as_ints(self) -> Result<&'t Vec<u32>, Error> { self.map_tag(|t| t.as_ints()) }

    // and now everything below this is defined in terms of the above
    fn index(self, index: usize) -> Result<&'t Tag, Error> {
        self.as_list().and_then(|v| {
            v.get(index).ok_or_else(|| Error::InvalidIndex(index))
        })
    }
    fn key(self, key: &str) -> Result<&'t Tag, Error> {
        self.as_map().and_then(|m| {
            m.get(key).ok_or_else(|| Error::InvalidKey(key.to_string()))
        })
    }
}

// a helper to define as_i8, etc.
macro_rules! simple_getter {
    ($name:ident, $r:ty, $pat:path) => {
        fn $name(self) -> Result<$r, Error> {
            if let &$pat(v) = self {
                Ok(v)
            } else {
                Err(Error::UnexpectedType)
            }
        }
    };
}

// &Tags are taglike
impl<'t> Taglike<'t> for &'t Tag {
    fn map_tag<F, T>(self, f: F) -> Result<T, Error> where F: FnOnce(&'t Tag) -> Result<T, Error> {
        f(self)
    }
    
    simple_getter!(as_i8, i8, Tag::TagByte);
    simple_getter!(as_i16, i16, Tag::TagShort);
    simple_getter!(as_i32, i32, Tag::TagInt);
    simple_getter!(as_i64, i64, Tag::TagLong);
    simple_getter!(as_f32, f32, Tag::TagFloat);
    simple_getter!(as_f64, f64, Tag::TagDouble);

    fn as_bytes(self) -> Result<&'t Vec<u8>, Error> {
        if let &Tag::TagByteArray(ref v) = self {
            Ok(v)
        } else {
            Err(Error::UnexpectedType)
        }
    }
    fn as_string(self) -> Result<&'t String, Error> {
        if let &Tag::TagString(ref v) = self {
            Ok(v)
        } else {
            Err(Error::UnexpectedType)
        }
    }
    fn as_list(self) -> Result<&'t Vec<Tag>, Error> {
        if let &Tag::TagList(ref v) = self {
            Ok(v)
        } else {
            Err(Error::UnexpectedType)
        }
    }
    fn as_map(self) -> Result<&'t HashMap<String, Tag>, Error> {
        if let &Tag::TagCompound(ref v) = self {
            Ok(v)
        } else {
            Err(Error::UnexpectedType)
        }
    }
    fn as_ints(self) -> Result<&'t Vec<u32>, Error> {
        if let &Tag::TagIntArray(ref v) = self {
            Ok(v)
        } else {
            Err(Error::UnexpectedType)
        }
    }
}

// Results containing Taglike things are Taglike
impl<'t, T> Taglike<'t> for Result<T, Error> where T: Taglike<'t> {
    fn map_tag<F, R>(self, f: F) -> Result<R, Error> where F: FnOnce(&'t Tag) -> Result<R, Error> {
        self.and_then(|t| t.map_tag(f))
    }
}

// now, on to actually parsing the things
impl Tag {
    /// Attempts to parse some data as a NBT
    pub fn parse<R>(r: &mut R) -> Result<(String, Tag), Error> where R: Read {
        let ty = try!(r.read_u8());
        let name = try!(Tag::read_string(r));
        let tag = try!(Tag::parse_tag(r, Some(ty)));
        Ok((name, tag))
    }

    pub fn parse_tag<R>(r: &mut R, tag_type: Option<u8>) -> Result<Tag, Error> where R: Read {
        let tag_type = try!(tag_type.map_or_else(|| r.read_u8(), Ok));
        Ok(match tag_type {
            0 => Tag::TagEnd,
            1 => Tag::TagByte(try!(r.read_i8())),
            2 => Tag::TagShort(try!(r.read_i16::<BigEndian>())),
            3 => Tag::TagInt(try!(r.read_i32::<BigEndian>())),
            4 => Tag::TagLong(try!(r.read_i64::<BigEndian>())),
            5 => Tag::TagFloat(try!(r.read_f32::<BigEndian>())),
            6 => Tag::TagDouble(try!(r.read_f64::<BigEndian>())),
            7 => { // TAG_Byte_Array 
                let len = try!(r.read_u32::<BigEndian>());
                let mut buf = vec![0; len as usize];
                try!(r.read_exact(&mut buf));
                Tag::TagByteArray(buf)
            }
            8 => { // TAG_String
                let s = try!(Tag::read_string(r));
                Tag::TagString(s)
            }
            9 => { // TAG_List
                let ty = try!(r.read_u8());
                let len = try!(r.read_u32::<BigEndian>());
                let mut v = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let t = try!(Tag::parse_tag(r, Some(ty)));
                    v.push(t)
                }
                Tag::TagList(v)
            }
            10 => { // TAG_Compound
                let mut v = HashMap::new();
                loop {
                    let ty = try!(r.read_u8());
                    if ty == 0 {
                        break;
                    }
                    let name = try!(Tag::read_string(r));
                    let value = try!(Tag::parse_tag(r, Some(ty)));
                    v.insert(name, value);
                }
                Tag::TagCompound(v)
            }
            11 => { // TAG_IntArray
                let len = try!(r.read_u32::<BigEndian>());
                let mut v = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let i = try!(r.read_u32::<BigEndian>());
                    v.push(i)
                }
                Tag::TagIntArray(v)
            }
            x => return Err(Error::UnexpectedTag(x))
        })
    }

    fn read_string<R>(r: &mut R) -> Result<String, Error> where R: Read {
        let len = try!(r.read_u16::<BigEndian>());
        let mut buf = vec![0; len as usize];
        try!(r.read_exact(&mut buf));
        Ok(try!(String::from_utf8(buf)))
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            &Tag::TagEnd => "TAG_End",
            &Tag::TagByte(_) => "TAG_Byte",
            &Tag::TagShort(_) => "TAG_Short",
            &Tag::TagInt(_) => "TAG_Int",
            &Tag::TagLong(_) => "TAG_Long",
            &Tag::TagFloat(_) => "TAG_Float",
            &Tag::TagDouble(_) => "TAG_Double",
            &Tag::TagByteArray(_) => "TAG_ByteArray",
            &Tag::TagString(_) => "TAG_String",
            &Tag::TagList(_) => "TAG_List",
            &Tag::TagCompound(_) => "TAG_Compound",
            &Tag::TagIntArray(_) => "TAG_IntArray",
        }
    }

    pub fn pretty_print(&self, indent: usize, name: Option<&String>) {
        let name_s = name.map_or("".to_string(), |s| format!("(\"{}\")", s));

        match self {
            &Tag::TagCompound(ref v) => { println!("{1:0$}{2}{3} : {4} entries\n{1:0$}{{", indent,"", self.get_name(), name_s, v.len()); 
                for (name, val) in v.iter() {
                    val.pretty_print(indent + 4, Some(name));
                }
                println!("{1:0$}}}", indent, "");
            }
            &Tag::TagList(ref data) => {
                let end = Tag::TagEnd;
                let ex = data.get(0).unwrap_or(&end);
                println!("{1:0$}{2}{3} : {4} entries of type {5}\n{1:0$}{{",
                         indent,"", self.get_name(), name_s, data.len(), ex.get_name());
                for item in data.iter() {
                    item.pretty_print(indent + 4, None);
                }
                println!("{1:0$}}}", indent, "");
            }
            &Tag::TagString(ref s) => { println!("{1:0$}{2}{3} : {4}", indent, "", self.get_name(), name_s, s) }
            &Tag::TagByteArray(ref data) => { println!("{1:0$}{2}{3} : Length of {4}", indent, "", self.get_name(), name_s, data.len()); }
            &Tag::TagDouble(d) => { println!("{1:0$}{2}{3} : {4}", indent, "", self.get_name(), name_s, d); }
            &Tag::TagFloat(d) => { println!("{1:0$}{2}{3} : {4}", indent, "", self.get_name(), name_s, d); }
            &Tag::TagLong(d) => { println!("{1:0$}{2}{3} : {4}", indent, "", self.get_name(), name_s, d); }
            &Tag::TagInt(d) => { println!("{1:0$}{2}{3} : {4}", indent, "", self.get_name(), name_s, d); }
            &Tag::TagShort(d) => { println!("{1:0$}{2}{3} : {4}", indent, "", self.get_name(), name_s, d); }
            &Tag::TagByte(d) => { println!("{1:0$}{2}{3} : {4}", indent, "", self.get_name(), name_s, d); }
            &Tag::TagEnd => { println!("{1:0$}{2}{3}", indent, "", self.get_name(), name_s); }
            &Tag::TagIntArray(ref data) => { println!("{1:0$}{2}{3} : Length of {4}", indent, "", self.get_name(), name_s, data.len()); }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_tag(data: Vec<u8>, name: &str, tag: Tag) {
        use std::io::Cursor;

        let mut cur = Cursor::new(data);
        let (n, t) = Tag::parse(&mut cur).unwrap();
        assert_eq!(n, name);
        assert_eq!(t, tag);
    }
    
    #[test]
    fn test_level_dat() {
        use flate2::read::{GzDecoder};
        use std::fs;

        let level_dat = fs::File::open("tests/data/level.dat").unwrap();

        let mut decoder = GzDecoder::new(level_dat).unwrap();
        let (_, tag) = Tag::parse(&mut decoder).unwrap();
        tag.pretty_print(0, None);
        println!("{}", tag.key("Data").key("thundering").as_i8().unwrap());
    }

    #[test]
    fn test_tag_byte() {
        let data = vec!(1, 0, 5, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 69);
        test_tag(data, "hello", Tag::TagByte(69));
    }

    #[test]
    fn test_tag_byte_array() {
        let data = vec!(7, 0, 5, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0, 0, 0, 3, 69, 250, 123);
        test_tag(data, "hello", Tag::TagByteArray(vec![69, 250, 123]));
    }

    #[test]
    fn test_tag_string() {
        let data = vec!(8, 0, 5, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0, 3, 'c' as u8, 'a' as u8, 't' as u8);
        test_tag(data, "hello", Tag::TagString("cat".to_string()));
    }

    #[test]
    fn test_tag_list() {
        let data = vec!(9, 0, 2, 'h' as u8, 'i' as u8, 1, 0, 0, 0, 3, 1, 2, 3);
        test_tag(data, "hi", Tag::TagList(vec![Tag::TagByte(1), Tag::TagByte(2), Tag::TagByte(3)]));
    }
}
