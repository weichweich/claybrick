#[derive(Debug, Clone)]
pub struct Pdf {
    pub(crate) version: (u8, u8),
    pub(crate) announced_binary: bool,
    pub(crate) objects: Vec<Object>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    String(String),
    Float(f32),
    Integer(i32),
    Bool(bool),
    Name(Name),
    Array,
    Dictionary,
    Stream,
    Null,
    IndirectObject(IndirectObject),
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

#[derive(Debug, Clone, PartialEq)]
pub struct IndirectObject {
    pub(crate) index: u32,
    pub(crate) generation: u32,
    pub(crate) object: Box<Object>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceObject {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

pub type Name = Vec<u8>;

