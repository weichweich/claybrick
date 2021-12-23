use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Pdf {
    pub(crate) version: (u8, u8),
    pub(crate) announced_binary: bool,
    pub(crate) objects: Vec<Object>,
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
    String(String),
    HexString(Bytes),
    Float(f32),
    Integer(i32),
    Bool(bool),
    Name(Name),
    Array(Array),
    Dictionary(Dictionary),
    Stream(Dictionary, Bytes),
    Null,
    Indirect(IndirectObject),
    Reference(Reference),
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
            Object::Stream(_dict, _data) => write!(f, "Stream {{}}"),
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

impl From<String> for Object {
    fn from(v: String) -> Self {
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

#[derive(Debug, Clone, PartialEq)]
pub struct IndirectObject {
    pub(crate) index: u32,
    pub(crate) generation: u32,
    pub(crate) object: Box<Object>,
}

impl Display for IndirectObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Indirect {} {} {{ {} }}", self.index, self.generation, self.object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reference {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Name(Vec<u8>);

impl From<Vec<u8>> for Name {
    fn from(v: Vec<u8>) -> Self {
        Name(v)
    }
}

impl Deref for Name {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Name")
            .field(&String::from_utf8_lossy(&self.0[..]))
            .finish()
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &String::from_utf8_lossy(&self.0[..]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bytes(Vec<u8>);

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

pub type Dictionary = HashMap<Name, Object>;

#[derive(Debug, Clone, PartialEq)]
pub struct Array(Vec<Object>);

impl Array {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl Deref for Array {
    type Target = Vec<Object>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Array {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<Object>> for Array {
    fn from(objects: Vec<Object>) -> Self {
        Self(objects)
    }
}

impl std::fmt::Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Array [")?;
        for obj in self.iter() {
            write!(f, "\n  {}", obj)?;
        }
        write!(f, "]")?;
        Ok(())
    }
}
