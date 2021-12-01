#[derive(Debug, Clone)]
pub struct Pdf {
    pub(crate) version: (u8, u8),
    pub(crate) announced_binary: bool,
    pub(crate) objects: Vec<Object>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    String(String),
    Float(f64),
    Integer(isize),
    Bool(bool),
    Name,
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

impl From<isize> for Object {
    fn from(v: isize) -> Self {
        Self::Integer(v)
    }
}

impl From<f64> for Object {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<String> for Object {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndirectObject {
    pub(crate) index: u32,
    pub(crate) generation: u32,
    pub(crate) object: Box<Object>,
}
