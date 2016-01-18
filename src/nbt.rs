use std::io::{Read};
use std::collections::HashMap;
use byteorder::{ReadBytesExt, BigEndian};

#[derive(Debug, PartialEq, Clone)]
pub enum Tag {
    TagEnd,
    TagByte(i8),
    TagShort(i16),
    TagInt(i32),
    TagLong(i64),
    TagFloat(f32),
    TagDouble(f64),
    TagByteArray(Vec<i8>),
    TagString(String),
    TagList(Vec<Tag>),
    TagCompound(HashMap<String, Tag>),
}

impl Tag {
    pub fn parse_file<R>(r: &mut R) -> (String, Tag) where R: Read {
        let ty = r.read_i8().unwrap();
        let name = read_string(r);
        let tag = Tag::parse_tag(r, Some(ty));
        (name, tag)
    }

    pub fn parse_tag<R>(r: &mut R, tag_type: Option<i8>) -> Tag where R: Read {
        match tag_type.unwrap_or_else(|| r.read_i8().unwrap()) {
            0 => Tag::TagEnd,
            1 => Tag::TagByte(r.read_i8().unwrap()),
            2 => Tag::TagShort(r.read_i16::<BigEndian>().unwrap()),
            3 => Tag::TagInt(r.read_i32::<BigEndian>().unwrap()),
            4 => Tag::TagLong(r.read_i64::<BigEndian>().unwrap()),
            5 => Tag::TagFloat(r.read_f32::<BigEndian>().unwrap()),
            6 => Tag::TagDouble(r.read_f64::<BigEndian>().unwrap()),
            7 => { // TAG_Byte_Array 
                let len: i32 = r.read_i32::<BigEndian>().unwrap();
                let mut v: Vec<i8> = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let b: i8 = r.read_i8().unwrap();
                    v.push(b)
                }
                Tag::TagByteArray(v)
            }
            8 => { // TAG_String
                let s: String = read_string(r);
                Tag::TagString(s)
            }
            9 => { // TAG_List
                let ty: i8 = r.read_i8().unwrap();
                let len: i32 = r.read_i32::<BigEndian>().unwrap();
                let mut v: Vec<Tag> = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let t: Tag = Tag::parse_tag(r, Some(ty));
                    v.push(t)
                }
                Tag::TagList(v)

            }
            10 => { // TAG_Compound
                let mut v = HashMap::new();
                loop {
                    let ty = r.read_i8().unwrap();
                    if ty == 0 {
                        break;
                    }
                    let name = read_string(r);
                    let value = Tag::parse_tag(r, Some(ty));
                    v.insert(name, value);
                }
                Tag::TagCompound(v)
            }
            x => panic!(">>> Unexpected tag type {:?}", x)
        }
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
        let name_s = match name {
            Some(ref s) => format!("(\"{}\")", s),
            None => "".to_owned()
        };
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
                for item in data.iter() {item.pretty_print(indent+4, None); }
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
            _ => println!("?")
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct NamedTag {
    pub tag: Tag,
    pub name: String
}

fn read_string<R>(r: &mut R) -> String
where R: Read {
    let len: i16 = r.read_i16::<BigEndian>().unwrap();
    let mut s2: Vec<u8> = Vec::new();
    for b in r.take(len as u64).bytes() {
        let b: u8 = b.unwrap();
        if b > 0 { s2.push(b) }
    }
    String::from_utf8(s2).unwrap()
}

#[test]
fn test_level_dat() {
    use flate2::read::{GzDecoder};
    use std::path::Path;
    use std::fs;
    use std::io::{Read, Bytes};

    let level_dat_path = Path::new("level.dat");
    let level_dat = fs::File::open(&level_dat_path).unwrap();

    let mut decoder: GzDecoder<fs::File> = GzDecoder::new(level_dat).unwrap();
    let (_, tag) = Tag::parse_file(&mut decoder);
    tag.pretty_print(0, None);
}

#[test]
fn test_tag_byte() {
    use std::io::Cursor;

    let data: Vec<u8> = vec!(1, 0, 5, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 69);
    let mut cur = Cursor::new(data);
    let (name, tag) = Tag::parse_file(&mut cur);
    assert_eq!(name, "hello");
    if let Tag::TagByte(b) = tag {
        assert_eq!(b, 69);
    } else {
        panic!("Unexpected tag type!");
    }
}

#[test]
fn test_tag_byte_array() {
    use std::io::Cursor;

    let data: Vec<u8> = vec!(7, 0, 5, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0, 0, 0, 3, 69, 250, 123);
    let mut cur = Cursor::new(data);
    let (name, tag) = Tag::parse_file(&mut cur);
    assert_eq!(name, "hello");
    if let Tag::TagByteArray(bytes) = tag {
        assert_eq!(bytes, vec!(69, -6, 123));
    } else {
        panic!("Unexpected tag type!");
    }
}

#[test]
fn test_tag_string() {
    use std::io::Cursor;

    let data: Vec<u8> = vec!(8, 0, 5, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0, 3, 'c' as u8, 'a' as u8, 't' as u8);
    let mut cur = Cursor::new(data);
    let (name , tag) = Tag::parse_file(&mut cur);
    assert_eq!(name, "hello");
    if let Tag::TagString(s) = tag {
        assert_eq!(s, "cat");
    } else {
        panic!("Unexpected tag type!");
    }
}

#[test]
fn test_tag_list() {
    use std::io::Cursor;
    let data: Vec<u8> = vec!(9, 0, 2, 'h' as u8, 'i' as u8, 1, 0, 0, 0, 3, 1, 2, 3);
    let mut cur = Cursor::new(data);
    let (name, tag) = Tag::parse_file(&mut cur);
    assert_eq!(name, "hi");
    if let Tag::TagList(data) = tag {
        assert_eq!(data.len(), 3);
        assert_eq!(data[0], Tag::TagByte(1));
        assert_eq!(data[1], Tag::TagByte(2));
        assert_eq!(data[2], Tag::TagByte(3));
    } else {
        panic!("Unexpected tag type");
    }

}
