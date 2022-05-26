#[derive(Debug, Clone, PartialEq)]
pub enum XrefKind {
    Table,
    Stream { number: u32, generation: u32 },
}

/// References to objects inside a PDF section.
///
/// References in this table mark object indices either as used or unused.
/// Unused object indices may be reused for new objects. Used objects are
/// divided into two groups compressed and uncompressed objects. Uncompressed
/// objects can be imidiately accessed at the given byte offset while compressed
/// objects are contained inside a stream object.
///
/// The entries are sorted by the object index.
#[derive(Debug, Clone, PartialEq)]
pub struct Xref {
    /// The entries of the cross reference
    pub(crate) entries: Vec<XrefEntry>,

    /// An optional associate type
    pub(crate) kind: Option<XrefKind>,
}

impl Xref {
    pub(crate) fn new(mut entries: Vec<XrefEntry>) -> Self {
        entries.sort_by_key(|o| o.number());
        Xref { entries, kind: None }
    }

    pub(crate) fn new_table(entries: Vec<XrefEntry>) -> Self {
        let mut xref = Self::new(entries);
        xref.kind = Some(XrefKind::Table);
        xref
    }

    pub(crate) fn new_stream(entries: Vec<XrefEntry>, number: u32, generation: u32) -> Self {
        let mut xref = Self::new(entries);
        xref.kind = Some(XrefKind::Stream { number, generation });
        xref
    }

    pub fn used_objects(&self) -> impl Iterator<Item = &UsedObject> {
        self.entries
            .iter()
            .filter_map(|entry| if let XrefEntry::Used(u) = entry { Some(u) } else { None })
    }

    pub fn compressed_objects(&self) -> impl Iterator<Item = &UsedCompressedObject> {
        self.entries.iter().filter_map(|entry| {
            if let XrefEntry::UsedCompressed(u) = entry {
                Some(u)
            } else {
                None
            }
        })
    }

    pub fn free_objects(&self) -> impl Iterator<Item = &FreeObject> {
        self.entries
            .iter()
            .filter_map(|entry| if let XrefEntry::Free(u) = entry { Some(u) } else { None })
    }

    pub fn entries(&self) -> impl Iterator<Item = &XrefEntry> {
        self.entries.iter()
    }
}

impl std::ops::Deref for Xref {
    type Target = Vec<XrefEntry>;

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl From<Vec<XrefEntry>> for Xref {
    fn from(v: Vec<XrefEntry>) -> Self {
        Xref::new(v)
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
    /// Index of the object in the object streams
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
