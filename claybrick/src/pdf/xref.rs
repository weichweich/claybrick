/// References to objects inside a PDF section.
///
/// References in this table mark object indices either as used or unused.
/// Unused object indices may be reused for new objects. Used objects are
/// divided into two groups compressed and uncompressed objects. Uncompressed
/// objects can be imidiately accessed at the given byte offset while compressed
/// objects are contained inside a stream object.
#[derive(Debug, Clone, PartialEq)]
pub struct Xref(Vec<XrefEntry>);

impl Xref {
    pub fn used_objects(&self) -> impl Iterator<Item = &UsedObject> {
        self.0
            .iter()
            .filter_map(|entry| if let XrefEntry::Used(u) = entry { Some(u) } else { None })
    }

    pub fn compressed_objects(&self) -> impl Iterator<Item = &UsedCompressedObject> {
        self.0.iter().filter_map(|entry| {
            if let XrefEntry::UsedCompressed(u) = entry {
                Some(u)
            } else {
                None
            }
        })
    }

    pub fn free_objects(&self) -> impl Iterator<Item = &FreeObject> {
        self.0
            .iter()
            .filter_map(|entry| if let XrefEntry::Free(u) = entry { Some(u) } else { None })
    }
}

impl From<Vec<XrefEntry>> for Xref {
    fn from(v: Vec<XrefEntry>) -> Self {
        Xref(v)
    }
}

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

/// Denotes a free object reference in a xref stream.
pub const XREF_FREE: usize = 0;
/// Denotes a used object reference in a xref stream.
pub const XREF_USED: usize = 1;
/// Denotes a used and compressed object reference in a xref stream.
pub const XREF_COMPRESSED: usize = 2;

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
        match self {
            XrefEntry::Free(_) => XREF_FREE,
            XrefEntry::Used(_) => XREF_USED,
            XrefEntry::UsedCompressed(_) => XREF_COMPRESSED,
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

impl From<Unsupported> for XrefEntry {
    fn from(v: Unsupported) -> Self {
        Self::Unsupported(v)
    }
}

impl From<UsedCompressedObject> for XrefEntry {
    fn from(v: UsedCompressedObject) -> Self {
        Self::UsedCompressed(v)
    }
}

impl From<UsedObject> for XrefEntry {
    fn from(v: UsedObject) -> Self {
        Self::Used(v)
    }
}

impl From<FreeObject> for XrefEntry {
    fn from(v: FreeObject) -> Self {
        Self::Free(v)
    }
}
