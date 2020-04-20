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
    TagLongArray(Vec<u64>),
}

// trait to simplify grabbing nested NBT data
pub trait Taglike<'t> : Sized {
    fn map_tag<F, T>(self, f: F) -> Option<T> where F: FnOnce(&'t Tag) -> Option<T>;

    // the rest of these are defaults that work, relying on
    // the implementation for Tag
    fn as_i8(self) -> Option<i8> {
        self.map_tag(|t| t.as_i8())
    }
    fn as_i16(self) -> Option<i16> {
        self.map_tag(|t| t.as_i16())
    }
    fn as_i32(self) -> Option<i32> {
        self.map_tag(|t| t.as_i32())
    }
    fn as_i64(self) -> Option<i64> {
        self.map_tag(|t| t.as_i64())
    }
    fn as_f32(self) -> Option<f32> {
        self.map_tag(|t| t.as_f32())
    }
    fn as_f64(self) -> Option<f64> {
        self.map_tag(|t| t.as_f64())
    }
    fn as_bytes(self) -> Option<&'t Vec<u8>> {
        self.map_tag(|t| t.as_bytes())
    }
    fn as_string(self) -> Option<&'t String> {
        self.map_tag(|t| t.as_string())
    }
    fn as_list(self) -> Option<&'t Vec<Tag>> {
        self.map_tag(|t| t.as_list())
    }
    fn as_map(self) -> Option<&'t HashMap<String, Tag>> {
        self.map_tag(|t| t.as_map())
    }
    fn as_ints(self) -> Option<&'t Vec<u32>> {
        self.map_tag(|t| t.as_ints())
    }
    fn as_longs(self) -> Option<&'t Vec<u64>> {
        self.map_tag(|t| t.as_longs())
    }

    // and now everything below this is defined in terms of the above
    fn index(self, index: usize) -> Option<&'t Tag> {
        self.as_list().and_then(|v| v.get(index))
    }
    fn key(self, key: &str) -> Option<&'t Tag> {
        self.as_map().and_then(|m| m.get(key))
    }
}

// a helper to define as_i8, etc.
macro_rules! simple_getter {
    (clone, $name:ident, $r:ty, $pat:path) => {
        fn $name(self) -> Option<$r> {
            if let &$pat(v) = self {
                Some(v)
            } else {
                None
            }
        }
    };
    (ref, $name:ident, $r:ty, $pat:path) => {
        fn $name(self) -> Option<$r> {
            if let &$pat(ref v) = self {
                Some(v)
            } else {
                None
            }
        }
    };
}

// &Tags are taglike
impl<'t> Taglike<'t> for &'t Tag {
    fn map_tag<F, T>(self, f: F) -> Option<T>
        where F: FnOnce(&'t Tag) -> Option<T>
    {
        f(self)
    }

    simple_getter!(clone, as_i8, i8, Tag::TagByte);
    simple_getter!(clone, as_i16, i16, Tag::TagShort);
    simple_getter!(clone, as_i32, i32, Tag::TagInt);
    simple_getter!(clone, as_i64, i64, Tag::TagLong);
    simple_getter!(clone, as_f32, f32, Tag::TagFloat);
    simple_getter!(clone, as_f64, f64, Tag::TagDouble);

    simple_getter!(ref, as_bytes, &'t Vec<u8>, Tag::TagByteArray);
    simple_getter!(ref, as_string, &'t String, Tag::TagString);
    simple_getter!(ref, as_list, &'t Vec<Tag>, Tag::TagList);
    simple_getter!(ref, as_map, &'t HashMap<String, Tag>, Tag::TagCompound);
    simple_getter!(ref, as_ints, &'t Vec<u32>, Tag::TagIntArray);
    simple_getter!(ref, as_longs, &'t Vec<u64>, Tag::TagLongArray);
}


// Options containing Taglike things are Taglike
impl<'t, T> Taglike<'t> for Option<T> where T: Taglike<'t>
{
    fn map_tag<F, R>(self, f: F) -> Option<R>
        where F: FnOnce(&'t Tag) -> Option<R>
    {
        self.and_then(|t| t.map_tag(f))
    }
}

// Results containing taglike things are Taglike
impl<'t, T, E> Taglike<'t> for Result<T, E> where T: Taglike<'t>
{
    fn map_tag<F, R>(self, f: F) -> Option<R>
        where F: FnOnce(&'t Tag) -> Option<R>
    {
        if let Ok(t) = self {
            t.map_tag(f)
        } else {
            None
        }
    }
}


// now, on to actually parsing the things
impl Tag {
    /// Attempts to parse some data as a NBT
    pub fn parse<R>(r: &mut R) -> Result<(String, Tag), Error>
        where R: Read
    {
        let ty = r.read_u8()?;
        let name = Tag::read_string(r)?;
        let tag = Tag::parse_tag(r, Some(ty))?;
        Ok((name, tag))
    }

    pub fn parse_tag<R>(r: &mut R, tag_type: Option<u8>) -> Result<Tag, Error>
        where R: Read
    {
        let tag_type = tag_type.map_or_else(|| r.read_u8(), Ok)?;
        Ok(match tag_type {
            0 => Tag::TagEnd,
            1 => Tag::TagByte(r.read_i8()?),
            2 => Tag::TagShort(r.read_i16::<BigEndian>()?),
            3 => Tag::TagInt(r.read_i32::<BigEndian>()?),
            4 => Tag::TagLong(r.read_i64::<BigEndian>()?),
            5 => Tag::TagFloat(r.read_f32::<BigEndian>()?),
            6 => Tag::TagDouble(r.read_f64::<BigEndian>()?),
            7 => {
                // TAG_Byte_Array
                let len = r.read_u32::<BigEndian>()?;
                let mut buf = vec![0; len as usize];
                r.read_exact(&mut buf)?;
                Tag::TagByteArray(buf)
            }
            8 => {
                // TAG_String
                let s = Tag::read_string(r)?;
                Tag::TagString(s)
            }
            9 => {
                // TAG_List
                let ty = r.read_u8()?;
                let len = r.read_u32::<BigEndian>()?;
                let mut v = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let t = Tag::parse_tag(r, Some(ty))?;
                    v.push(t)
                }
                Tag::TagList(v)
            }
            10 => {
                // TAG_Compound
                let mut v = HashMap::new();
                loop {
                    let ty = r.read_u8()?;
                    if ty == 0 {
                        break;
                    }
                    let name = Tag::read_string(r)?;
                    let value = Tag::parse_tag(r, Some(ty))?;
                    v.insert(name, value);
                }
                Tag::TagCompound(v)
            }
            11 => {
                // TAG_IntArray
                let len = r.read_u32::<BigEndian>()?;
                let mut v = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let i = r.read_u32::<BigEndian>()?;
                    v.push(i)
                }
                Tag::TagIntArray(v)
            },
            12 => {
                // TAG_LongArray
                let len = r.read_u32::<BigEndian>()?;
                let mut v = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let i = r.read_u64::<BigEndian>()?;
                    v.push(i)
                }
                Tag::TagLongArray(v)
            }

            x => return Err(Error::UnexpectedTag(x)),
        })
    }

    fn read_string<R>(r: &mut R) -> Result<String, Error>
        where R: Read
    {
        let len = r.read_u16::<BigEndian>()?;
        let mut buf = vec![0; len as usize];
        r.read_exact(&mut buf)?;
        Ok(String::from_utf8(buf)?)
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
            &Tag::TagLongArray(_) => "TAG_LongArray"
        }
    }

    pub fn pretty_print(&self, indent: usize, name: Option<&str>) {
        let name_s = name.map_or("".to_string(), |s| format!("(\"{}\")", s));

        match self {
            &Tag::TagCompound(ref v) => {
                println!("{1:0$}{2}{3} : {4} entries\n{1:0$}{{",
                         indent,
                         "",
                         self.get_name(),
                         name_s,
                         v.len());
                for (name, val) in v.iter() {
                    val.pretty_print(indent + 4, Some(name));
                }
                println!("{1:0$}}}", indent, "");
            }
            &Tag::TagList(ref data) => {
                let end = Tag::TagEnd;
                let ex = data.get(0).unwrap_or(&end);
                println!("{1:0$}{2}{3} : {4} entries of type {5}\n{1:0$}{{",
                         indent,
                         "",
                         self.get_name(),
                         name_s,
                         data.len(),
                         ex.get_name());
                for item in data.iter() {
                    item.pretty_print(indent + 4, None);
                }
                println!("{1:0$}}}", indent, "");
            }
            &Tag::TagString(ref s) => {
                println!("{1:0$}{2}{3} : {4}", indent, "", self.get_name(), name_s, s)
            }
            &Tag::TagByteArray(ref data) => {
                println!("{1:0$}{2}{3} : Length of {4}",
                         indent,
                         "",
                         self.get_name(),
                         name_s,
                         data.len());
            }
            &Tag::TagDouble(d) => {
                println!("{1:0$}{2}{3} : {4}", indent, "", self.get_name(), name_s, d);
            }
            &Tag::TagFloat(d) => {
                println!("{1:0$}{2}{3} : {4}", indent, "", self.get_name(), name_s, d);
            }
            &Tag::TagLong(d) => {
                println!("{1:0$}{2}{3} : {4}", indent, "", self.get_name(), name_s, d);
            }
            &Tag::TagInt(d) => {
                println!("{1:0$}{2}{3} : {4}", indent, "", self.get_name(), name_s, d);
            }
            &Tag::TagShort(d) => {
                println!("{1:0$}{2}{3} : {4}", indent, "", self.get_name(), name_s, d);
            }
            &Tag::TagByte(d) => {
                println!("{1:0$}{2}{3} : {4}", indent, "", self.get_name(), name_s, d);
            }
            &Tag::TagEnd => {
                println!("{1:0$}{2}{3}", indent, "", self.get_name(), name_s);
            }
            &Tag::TagIntArray(ref data) => {
                println!("{1:0$}{2}{3} : Length of {4}",
                         indent,
                         "",
                         self.get_name(),
                         name_s,
                         data.len());
            }
            &Tag::TagLongArray(ref data) => {
                println!("{1:0$}{2}{3} : Length of {4}",
                         indent,
                         "",
                         self.get_name(),
                         name_s,
                         data.len());
            }
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
        use flate2::read::GzDecoder;
        use std::fs;

        let level_dat = fs::File::open("tests/data/level.dat").unwrap();

        let mut decoder = GzDecoder::new(level_dat);
        let (_, tag) = Tag::parse(&mut decoder).unwrap();
        tag.pretty_print(0, None);
        let data = tag.key("Data").unwrap();
        assert_eq!(data.key("thundering").as_i8().unwrap(), 0);
        let game_rules = data.key("GameRules").as_map().unwrap();
        assert_eq!(game_rules.len(), 15);
        assert_eq!(data.key("LastPlayed").as_i64().unwrap(), 1424141505856);
        let motion = data.key("Player").key("Motion");
        // i don't know the exact values for these fields, so compare with a tolerance
        assert!((motion.index(0).as_f64().unwrap() - -0.035653).abs() < 0.00001f64);
        assert!((motion.index(1).as_f64().unwrap() - -1.062104).abs() < 0.00001f64);
        assert!((motion.index(2).as_f64().unwrap() - 0.0000000).abs() < 0.00001f64);
    }

    #[test]
    fn test_lifetimes() {
        use flate2::read::GzDecoder;
        use std::fs;

        let level_dat = fs::File::open("tests/data/level.dat").unwrap();

        let mut decoder = GzDecoder::new(level_dat);
        let (_, tag) = Tag::parse(&mut decoder).unwrap();
        let player_tag: &Tag = tag.key("Data").key("Player").unwrap();
        assert_eq!(player_tag.key("DeathTime").as_i16().unwrap(), 20);
        assert_eq!(player_tag.key("OnGround").as_i8().unwrap(), 0);
    }


    #[test]
    fn test_tag_byte() {
        let data = vec![1, 0, 5, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 69];
        test_tag(data, "hello", Tag::TagByte(69));
    }

    #[test]
    fn test_tag_byte_array() {
        let data = vec![7, 0, 5, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0, 0, 0,
                        3, 69, 250, 123];
        test_tag(data, "hello", Tag::TagByteArray(vec![69, 250, 123]));
    }

    #[test]
    fn test_tag_string() {
        let data = vec![8, 0, 5, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0, 3,
                        'c' as u8, 'a' as u8, 't' as u8];
        test_tag(data, "hello", Tag::TagString("cat".to_string()));
    }

    #[test]
    fn test_tag_list() {
        let data = vec![9, 0, 2, 'h' as u8, 'i' as u8, 1, 0, 0, 0, 3, 1, 2, 3];
        test_tag(data,
                 "hi",
                 Tag::TagList(vec![Tag::TagByte(1), Tag::TagByte(2), Tag::TagByte(3)]));
    }
}
