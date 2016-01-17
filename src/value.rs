use serde::de;

#[derive(Debug, PartialEq)]
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
    TagCompound(Vec<(String, Tag)>),
}

impl Tag {
    pub fn get_name(&self) -> String {
        let s = match self {
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
            &Tag::TagCompound(_) => "TAG_Compound"
        } ;
        s.to_owned()
    }

    pub fn pretty_print(&self, indent: usize, name: Option<&String>) {
        let name_s = match name {
            Some(ref s) => format!("(\"{}\")", s),
            None => "".to_owned()
        };
        match self {
            &Tag::TagCompound(ref v) => { println!("{1:0$}{2}{3} : {4} entries\n{1:0$}{{", indent,"", self.get_name(), name_s, v.len()); 
                for &(ref n, ref item) in v.iter() { item.pretty_print(indent+4, Some(n)); }
                println!("{1:0$}}}", indent, "");
            }
            &Tag::TagList(ref data) => {
                let def_ex = Tag::TagEnd;
                let ex = data.get(0).unwrap_or_else(|| &def_ex);
                println!("{1:0$}{2}{3} : {4} entries of type {5}\n{1:0$}{{",
                         indent,"", self.get_name(), name_s, data.len(), ex.get_name());
                for item in data.iter() {item.pretty_print(indent+4, None); }
                println!("{1:0$}}}", indent, "");
            }
            &Tag::TagString(ref s) => { println!("{1:0$}{2}{3} : {4}", indent, "", self.get_name(), name_s, s) }
            &Tag::TagByteArray(ref v) => { println!("{1:0$}{2}{3} : Length of {4}", indent, "", self.get_name(), name_s, v.len()); }
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

impl de::Deserialize for Tag {
    fn deserialize<D>(deserializer: &mut D) -> Result<Tag, D::Error>
        where D: de::Deserializer
    {
        deserializer.visit(Visitor)
    }
}

struct Visitor;

impl de::Visitor for Visitor {
    type Value = Tag;
    
    fn visit_i8<E>(&mut self, v: i8) -> Result<Tag, E> {
        Ok(Tag::TagByte(v))
    }
    fn visit_i16<E>(&mut self, v: i16) -> Result<Tag, E> {
        Ok(Tag::TagShort(v))
    }
    fn visit_i32<E>(&mut self, v: i32) -> Result<Tag, E> {
        Ok(Tag::TagInt(v))
    }
    fn visit_i64<E>(&mut self, v: i64) -> Result<Tag, E> {
        Ok(Tag::TagLong(v))
    }
    fn visit_f32<E>(&mut self, v: f32) -> Result<Tag, E> {
        Ok(Tag::TagFloat(v))
    }
    fn visit_f64<E>(&mut self, v: f64) -> Result<Tag, E> {
        Ok(Tag::TagDouble(v))
    }
    fn visit_byte_buf<E>(&mut self, v: Vec<u8>) -> Result<Tag, E> {
        Ok(Tag::TagByteArray(v))
    }
    fn visit_string<E>(&mut self, v: String) -> Result<Tag, E> {
        Ok(Tag::TagString(v))
    }
    
    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<Tag, V::Error>
        where V: de::SeqVisitor
    {
        let (s, _) = visitor.size_hint();
        let mut vec = Vec::with_capacity(s);
        loop {
            match visitor.visit() {
                Ok(None) => break,
                Ok(Some(v)) => vec.push(v),
                Err(e) => return Err(e),
            }
        }
        visitor.end().map(|_| Tag::TagList(vec))
    }

    fn visit_map<V>(&mut self, mut visitor: V) -> Result<Tag, V::Error>
        where V: de::MapVisitor
    {
        let (s, _) = visitor.size_hint();
        let mut vec = Vec::with_capacity(s);
        loop {
            match visitor.visit() {
                Ok(None) => break,
                Ok(Some((k, v))) => vec.push((k, v)),
                Err(e) => return Err(e),
            }
        }
        visitor.end().map(|_| Tag::TagCompound(vec))
    }
}

#[test]
fn test_level_dat() {
    use flate2::read::{GzDecoder};
    use std::path::Path;
    use std::fs;
    use std::io::{Read, Bytes};
    use super::de::{read_named, Error};

    let level_dat_path = Path::new("level.dat");
    let level_dat = fs::File::open(&level_dat_path).unwrap();

    let decoder = GzDecoder::new(level_dat).unwrap();
    let res: Result<(String, Tag), Error> = read_named(decoder);
    match res {
        Err(e) => panic!("deserialize error: {:?}", e),
        Ok((_, v)) => v.pretty_print(0, None),
    }
}
