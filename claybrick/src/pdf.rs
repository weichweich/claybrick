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

pub const K_SIZE: &[u8] = b"Size";
pub const K_PREVIOUS: &[u8] = b"Prev";
pub const K_ENCRYPT: &[u8] = b"Encrypt";
pub const K_ROOT: &[u8] = b"Root";
pub const K_ID: &[u8] = b"ID";
pub const K_X_REF_STM: &[u8] = b"XRefStm";

#[derive(Debug, Clone, PartialEq)]
pub struct Pdf {
    pub(crate) version: (u8, u8),
    pub(crate) announced_binary: bool,
    pub(crate) sections: Vec<PdfSection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfSection {
    pub(crate) objects: Vec<Object>,
    pub(crate) startxref: usize,
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

impl TryFrom<Dictionary> for Trailer {
    type Error = Dictionary;

    fn try_from(value: Dictionary) -> Result<Self, Self::Error> {
        // TODO: don't clone
        Ok(Self {
            size: value.get(K_SIZE).and_then(Object::integer).ok_or_else(|| {
                log::error!("invalid size");
                value.clone()
            })?,

            previous: value.get(K_PREVIOUS).and_then(Object::integer),

            root: value
                .get(K_ROOT)
                .and_then(Object::reference)
                .map(Clone::clone)
                .ok_or_else(|| {
                    log::error!("invalid root");
                    value.clone()
                })?,

            encrypt: value.get(K_ENCRYPT).map(Clone::clone),

            info: value.get(K_ROOT).and_then(Object::dictionary).map(Clone::clone),

            id: value.get(K_ID).and_then(Object::array).and_then(|a| {
                if a.len() == 2 {
                    Some([
                        a.get(0).and_then(Object::hex_string)?.clone(),
                        a.get(1).and_then(Object::hex_string)?.clone(),
                    ])
                } else {
                    None
                }
            }),

            x_ref_stm: value.get(K_X_REF_STM).and_then(Object::integer),
        })
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

// TODO move xref related things into a separate module
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XrefTableEntry {
    pub object: usize,
    pub byte_offset: usize,
    pub generation: u32,
    /// Marks objects that are not in use/deleted as free.
    pub free: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreeObject {
    /// Number of this object
    pub number: usize,
    /// Next generation number that should be used
    pub gen: usize,
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
    pub gen: usize,
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
pub enum XrefStreamEntry {
    Free(FreeObject),
    Used(UsedObject),
    /// Object is stored in compressed stream
    UsedCompressed(UsedCompressedObject),

    /// Unsupported xref entry. Point to null object.
    Unsupported(Unsupported),
}

impl XrefStreamEntry {
    pub fn type_num(&self) -> usize {
        // TODO: use constantants
        match self {
            XrefStreamEntry::Free(_) => 0,
            XrefStreamEntry::Used(_) => 1,
            XrefStreamEntry::UsedCompressed(_) => 2,
            XrefStreamEntry::Unsupported(Unsupported { type_num, .. }) => *type_num,
        }
    }

    pub fn number(&self) -> usize {
        match self {
            XrefStreamEntry::Free(FreeObject { number, .. }) => *number,
            XrefStreamEntry::Used(UsedObject { number, .. }) => *number,
            XrefStreamEntry::UsedCompressed(UsedCompressedObject { number, .. }) => *number,
            XrefStreamEntry::Unsupported(Unsupported { number, .. }) => *number,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Xref {
    Table(Vec<XrefTableEntry>),
    Stream(Vec<XrefStreamEntry>),
}

impl From<Vec<XrefTableEntry>> for Xref {
    fn from(v: Vec<XrefTableEntry>) -> Self {
        Xref::Table(v)
    }
}

impl From<Vec<XrefStreamEntry>> for Xref {
    fn from(v: Vec<XrefStreamEntry>) -> Self {
        Xref::Stream(v)
    }
}
