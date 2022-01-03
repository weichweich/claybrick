use std::{collections::HashMap, fmt::Display, ops::Deref};

pub use self::{
    array::Array,
    indirect::{IndirectObject, Reference},
    name::Name,
    stream::Stream,
    string::CbString,
};

pub mod array;
pub mod indirect;
pub mod name;
pub mod stream;
pub mod string;

#[derive(Debug, Clone, PartialEq)]
pub struct Pdf {
    pub(crate) version: (u8, u8),
    pub(crate) announced_binary: bool,
    pub(crate) objects: Vec<Object>,
    pub(crate) startxref: usize,
    pub(crate) xref: Vec<XrefTableEntry>,
}

impl Display for Pdf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pdf {{")?;
        write!(f, " version: {}.{}", self.version.0, self.version.1)?;
        write!(f, ", binary: {}", self.announced_binary)?;
        for obj in &self.objects {
            write!(f, "\n  {}", obj)?;
        }
        write!(f, "}}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    String(CbString),
    HexString(Bytes),
    Float(f32),
    Integer(i32),
    Bool(bool),
    Name(Name),
    Array(Array),
    Dictionary(Dictionary),
    Stream(Stream),
    Null,
    Indirect(IndirectObject),
    Reference(Reference),
}

impl Object {
    pub fn name(&self) -> Option<&Name> {
        if let Object::Name(n) = self {
            Some(n)
        } else {
            None
        }
    }

    pub fn indirect(&self) -> Option<&IndirectObject> {
        if let Object::Indirect(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn stream(&self) -> Option<&Stream> {
        if let Object::Stream(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn dictionary(&self) -> Option<&Dictionary> {
        if let Object::Dictionary(d) = self {
            Some(d)
        } else {
            None
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::String(obj) => obj.fmt(f),
            Object::HexString(obj) => obj.fmt(f),
            Object::Float(obj) => obj.fmt(f),
            Object::Integer(obj) => obj.fmt(f),
            Object::Bool(obj) => obj.fmt(f),
            Object::Name(obj) => obj.fmt(f),
            Object::Array(obj) => obj.fmt(f),
            //TODO: implement display
            Object::Dictionary(_obj) => write!(f, "dict"),
            Object::Stream(Stream {
                dictionary: _dict,
                data: _data,
            }) => write!(f, "Stream {{}}"),
            Object::Null => write!(f, "NULL"),
            Object::Indirect(obj) => obj.fmt(f),
            Object::Reference(obj) => write!(f, "{:?}", obj),
        }
    }
}

impl From<bool> for Object {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<i32> for Object {
    fn from(v: i32) -> Self {
        Self::Integer(v)
    }
}

impl From<f32> for Object {
    fn from(v: f32) -> Self {
        Self::Float(v)
    }
}

impl From<CbString> for Object {
    fn from(v: CbString) -> Self {
        Self::String(v)
    }
}

impl From<Name> for Object {
    fn from(n: Name) -> Self {
        Self::Name(n)
    }
}

impl From<Vec<Object>> for Object {
    fn from(a: Vec<Object>) -> Self {
        Self::Array(a.into())
    }
}

impl From<Array> for Object {
    fn from(a: Array) -> Self {
        Self::Array(a)
    }
}

impl From<Dictionary> for Object {
    fn from(d: Dictionary) -> Self {
        Self::Dictionary(d)
    }
}

impl From<Stream> for Object {
    fn from(s: Stream) -> Self {
        Self::Stream(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bytes(pub Vec<u8>);

impl From<Vec<u8>> for Bytes {
    fn from(v: Vec<u8>) -> Self {
        Bytes(v)
    }
}

impl Deref for Bytes {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let limited_length = self.len().min(15);
        write!(f, "{}", &String::from_utf8_lossy(&self.0[..limited_length]))
    }
}

impl std::borrow::Borrow<[u8]> for Bytes {
    fn borrow(&self) -> &[u8] {
        &self.0[..]
    }
}

pub type Dictionary = HashMap<Name, Object>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XrefTableEntry {
    pub object: usize,
    pub byte_offset: usize,
    pub generation: u32,
    /// Marks objects that are not in use/deleted as free.
    pub free: bool,
}
