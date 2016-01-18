use std::io::Read;
use std::collections::HashMap;
use byteorder::{ReadBytesExt, BigEndian};

use super::error::Error;

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
}

impl Tag {
    pub fn parse_file<R>(r: &mut R) -> Result<(String, Tag), Error> where R: Read {
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
        }
    }

    pub fn pretty_print(&self, indent: usize, name: Option<&String>) {
        let name_s = name.map_or("".to_string(), |s| format!("9\"{}\")", s));

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
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_tag(data: Vec<u8>, name: &str, tag: Tag) {
        use std::io::Cursor;

        let mut cur = Cursor::new(data);
        let (n, t) = Tag::parse_file(&mut cur).unwrap();
        assert_eq!(n, name);
        assert_eq!(t, tag);
    }
    
    #[test]
    fn test_level_dat() {
        use flate2::read::{GzDecoder};
        use std::path::Path;
        use std::fs;
        use std::io::{Read, Bytes};

        let level_dat = fs::File::open("level.dat").unwrap();

        let mut decoder = GzDecoder::new(level_dat).unwrap();
        let (_, tag) = Tag::parse_file(&mut decoder).unwrap();
        tag.pretty_print(0, None);
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
