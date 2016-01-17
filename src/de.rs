use byteorder::{ReadBytesExt, BigEndian};
use serde::de;
use serde::de::value::ValueDeserializer;
use std::cmp::{PartialEq};
use std::fmt::{Debug};
use std::io::{Read};

#[derive(Debug)]
pub enum Error {
    BadListLength,
    BadMapLength,
    NotStringKeys,
    Syntax(String),
    EndOfStream,
    UnknownField(String),
    MissingField(String),
}

impl de::Error for Error {
    fn syntax(msg: &str) -> Error {
        Error::Syntax(msg.to_owned())
    }

    fn end_of_stream() -> Error {
        Error::EndOfStream
    }

    fn unknown_field(field: &str) -> Error {
        Error::UnknownField(field.to_owned())
    }

    fn missing_field(field: &'static str) -> Error {
        Error::MissingField(field.to_owned())
    }
}

impl From<de::value::Error> for Error {
    fn from(error: de::value::Error) -> Error {
        match error {
            de::value::Error::SyntaxError => Error::Syntax("expected some value".to_owned()),
            de::value::Error::EndOfStreamError => Error::EndOfStream,
            de::value::Error::UnknownFieldError(f) => Error::UnknownField(f),
            de::value::Error::MissingFieldError(f) => Error::MissingField(f.to_owned()),
        }
    }
}

struct Deserializer<R>{r: R}

pub fn read<R, T>(r: R) -> Result<T, Error>
    where R: Read, T: de::Deserialize
{
    let mut d = Deserializer{r: r};
    de::Deserialize::deserialize(&mut d)
}

pub fn read_named<R, T>(r: R) -> Result<(String, T), Error>
    where R: Read, T: de::Deserialize
{
    let mut d = Deserializer{r: r};
    let ty = d.read_type();
    let name = d.read_string();
    let mut dty = TypedDeserializer{d: &mut d, ty: ty};
    de::Deserialize::deserialize(&mut dty).map(|t| (name, t))
}

impl<R: Read> Deserializer<R> {
    #[inline]
    fn read_string(&mut self) -> String {
        let len = self.r.read_i16::<BigEndian>().unwrap();
        let mut buf = vec![0; len as usize];
        self.r.read_exact(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[inline]
    fn read_type(&mut self) -> u8 {
        self.r.read_u8().unwrap()
    }

    #[inline]
    fn read_tag<V>(&mut self, tag_type: Option<u8>, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor
    {
        match tag_type.unwrap_or_else(|| self.read_type()) {
            0 => visitor.visit_none(),
            1 => visitor.visit_i8(self.r.read_i8().unwrap()),
            2 => visitor.visit_i16(self.r.read_i16::<BigEndian>().unwrap()),
            3 => visitor.visit_i32(self.r.read_i32::<BigEndian>().unwrap()),
            4 => visitor.visit_i64(self.r.read_i64::<BigEndian>().unwrap()),
            5 => visitor.visit_f32(self.r.read_f32::<BigEndian>().unwrap()),
            6 => visitor.visit_f64(self.r.read_f64::<BigEndian>().unwrap()),
            7 => { // byte array
                let len = self.r.read_i32::<BigEndian>().unwrap();
                let mut buf = vec![0; len as usize];
                self.r.read_exact(&mut buf).unwrap();
                visitor.visit_byte_buf(buf)
            }
            8 => { // string
                let s = self.read_string();
                visitor.visit_string(s)
            }
            9 => { // list
                let ty = self.read_type();
                let len = self.r.read_u32::<BigEndian>().unwrap();
                visitor.visit_seq(SeqVisitor{d: self, ty: ty, len: len})
            }
            10 => { // compound
                visitor.visit_map(MapVisitor{d: self, ty: 1})
            }
            x => panic!("unexpected tag type {:?}", x)
        }
    }
}

impl<R: Read> de::Deserializer for Deserializer<R> {
    type Error = Error;
    
    #[inline]
    fn visit<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor
    {
        self.read_tag(None, visitor)
    }
}

struct TypedDeserializer<'a, R: 'a>{d: &'a mut Deserializer<R>, ty: u8}

impl<'a, R: Read> de::Deserializer for TypedDeserializer<'a, R> {
    type Error = Error;
    
    #[inline]
    fn visit<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor
    {
        self.d.read_tag(Some(self.ty), visitor)
    }
}

struct SeqVisitor<'a, R: 'a>{d: &'a mut Deserializer<R>, ty: u8, len: u32}

impl<'a, R: Read> de::SeqVisitor for SeqVisitor<'a, R> {
    type Error = Error;
    
    #[inline]
    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize
    {
        if self.len == 0 {
            Ok(None)
        } else {
            self.len -= 1;
            let mut d = TypedDeserializer{d: self.d, ty: self.ty};
            let value = try!(de::Deserialize::deserialize(&mut d));
            Ok(Some(value))
        }
    }

    #[inline]
    fn end(&mut self) -> Result<(), Error> {
        if self.len == 0 {
            Ok(())
        } else {
            Err(Error::BadListLength)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len as usize, Some(self.len as usize))
    }
}

struct MapVisitor<'a, R: 'a>{d: &'a mut Deserializer<R>, ty: u8}

impl<'a, R: Read> de::MapVisitor for MapVisitor<'a, R> {
    type Error = Error;

    #[inline]
    fn visit_key<K>(&mut self) -> Result<Option<K>, Error>
        where K: de::Deserialize
    {
        self.ty = self.d.read_type();
        if self.ty == 0 {
            Ok(None)
        } else {
            let k = self.d.read_string();
            let mut d = k.clone().into_deserializer();
            let key = try!(de::Deserialize::deserialize(&mut d));
            Ok(Some(key))
        }
    }

    #[inline]
    fn visit_value<V>(&mut self) -> Result<V, Error>
        where V: de::Deserialize
    {
        let mut d = TypedDeserializer{d: self.d, ty: self.ty};
        de::Deserialize::deserialize(&mut d)
    }

    #[inline]
    fn end(&mut self) -> Result<(), Error> {
        if self.ty == 0 {
            Ok(())
        } else {
            Err(Error::BadMapLength)
        }
    }
}

fn test_named_read<R, T>(r: R) -> (String, T)
    where T: de::Deserialize, R: Read
{
    let res: Result<(String, T), Error> = read_named(r);
    match res {
        Err(e) => panic!("deserialize error: {:?}", e),
        Ok(v) => v
    }
}

fn test_named<T>(data: Vec<u8>, name: &'static str, val: T)
    where T: Debug + de::Deserialize + PartialEq
{
    use std::io::Cursor;
    let (n, v): (String, T) = test_named_read(Cursor::new(data));
    assert_eq!(n, name);
    assert_eq!(v, val);
}

#[test]
fn test_tag_byte() {
    let data = vec!(1, 0, 5, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 69);
    test_named(data, "hello", 69 as u8);
}

#[test]
fn test_tag_byte_array() {
    use serde::bytes::{ByteBuf};
    let data: Vec<u8> = vec!(7, 0, 5, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0, 0, 0, 3, 69, 250, 123);
    test_named(data, "hello", ByteBuf::from(vec!(69 as u8, 250, 123)));
}

#[test]
fn test_tag_string() {
    let data: Vec<u8> = vec!(8, 0, 5, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, 0, 3, 'c' as u8, 'a' as u8, 't' as u8);
    test_named(data, "hello", "cat".to_owned());
}

#[test]
fn test_tag_list() {
    let data: Vec<u8> = vec!(9, 0, 2, 'h' as u8, 'i' as u8, 1, 0, 0, 0, 3, 1, 2, 3);
    test_named(data, "hi", vec![1 as i8, 2, 3]);
}
