use std::{collections::HashMap, ops::Deref};

#[derive(Debug, Clone, PartialEq)]
pub struct Pdf {
    pub(crate) version: (u8, u8),
    pub(crate) announced_binary: bool,
    pub(crate) objects: Vec<Object>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    String(String),
    HexString(Vec<u8>),
    Float(f32),
    Integer(i32),
    Bool(bool),
    Name(Name),
    Array(Array),
    Dictionary(Dictionary),
    Stream(Dictionary, Vec<u8>),
    Null,
    Indirect(IndirectObject),
    Reference(Reference),
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

pub type Dictionary = HashMap<Name, Object>;

pub type Array = Vec<Object>;
