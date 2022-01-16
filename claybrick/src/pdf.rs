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
    pub(crate) sections: Vec<PdfSection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfSection {
    pub(crate) objects: Vec<Object>,
    pub(crate) trailer: Option<Trailer>,
    pub(crate) xref: Xref,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trailer {
    pub size: i32,
    pub previous: Option<i32>,
    pub root: Reference,
    pub encrypt: Option<Object>,
    pub info: Option<Dictionary>,
    pub id: Option<[Bytes; 2]>,
    pub x_ref_stm: Option<i32>,
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

    pub fn array(&self) -> Option<&Array> {
        if let Object::Array(a) = self {
            Some(a)
        } else {
            None
        }
    }

    pub fn integer(&self) -> Option<i32> {
        if let Object::Integer(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    pub fn reference(&self) -> Option<&Reference> {
        if let Object::Reference(r) = self {
            Some(r)
        } else {
            None
        }
    }

    pub fn hex_string(&self) -> Option<&Bytes> {
        if let Object::HexString(b) = self {
            Some(b)
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

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Bytes(pub Vec<u8>);

impl std::fmt::Debug for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Bytes").field(&hex::encode(&self.0[..])).finish()
    }
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct Xref(Vec<XrefEntry>);

#[derive(Debug, Clone, PartialEq)]
pub struct FreeObject {
    /// Number of this object
    pub number: usize,
    /// Next generation number that should be used
    pub generation: usize,
    /// Next free object number
    pub next_free: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsedObject {
    /// Number of this object
    pub number: usize,
    /// The position of this object in the pdf file in bytes, starting from the
    /// beginning of the PDF.
    pub byte_offset: usize,
    /// Next generation number that should be used
    pub generation: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsedCompressedObject {
    /// Number of this object
    pub number: usize,
    /// The number of the stream object that contains this object
    pub containing_object: usize,
    /// Next generation number that should be used
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unsupported {
    /// Number of this object
    pub number: usize,
    /// The number of the stream object that contains this object
    pub type_num: usize,
    /// The number of the stream object that contains this object
    pub w1: usize,
    /// Next generation number that should be used
    pub w2: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum XrefEntry {
    Free(FreeObject),
    Used(UsedObject),
    /// Object is stored in compressed stream
    UsedCompressed(UsedCompressedObject),

    /// Unsupported xref entry. Point to null object.
    Unsupported(Unsupported),
}

impl XrefEntry {
    pub fn type_num(&self) -> usize {
        // TODO: use constantants
        match self {
            XrefEntry::Free(_) => 0,
            XrefEntry::Used(_) => 1,
            XrefEntry::UsedCompressed(_) => 2,
            XrefEntry::Unsupported(Unsupported { type_num, .. }) => *type_num,
        }
    }

    pub fn number(&self) -> usize {
        match self {
            XrefEntry::Free(FreeObject { number, .. }) => *number,
            XrefEntry::Used(UsedObject { number, .. }) => *number,
            XrefEntry::UsedCompressed(UsedCompressedObject { number, .. }) => *number,
            XrefEntry::Unsupported(Unsupported { number, .. }) => *number,
        }
    }
}

impl From<Vec<XrefEntry>> for Xref {
    fn from(v: Vec<XrefEntry>) -> Self {
        Xref(v)
    }
}
