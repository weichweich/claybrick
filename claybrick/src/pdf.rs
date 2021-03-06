use fnv::FnvHashMap;
use std::{collections::HashMap, ops::Deref};

pub use self::{
    document::{Catalog, CatalogError},
    object::{Array, CbString, IndirectObject, Name, Object, Reference, Stream},
    trailer::Trailer,
    xref::Xref,
};

pub mod document;
pub mod object;
pub mod trailer;
pub mod xref;

#[derive(Debug, Clone, PartialEq)]
pub struct RawPdf {
    pub(crate) version: (u8, u8),
    pub(crate) announced_binary: bool,
    pub(crate) sections: Vec<PdfSection>,
}

impl RawPdf {
    pub fn object(&self, num: usize) -> Option<&Object> {
        self.sections.iter().find_map(|s| s.objects.get(&num))
    }

    pub fn catalog(&self) -> Result<Catalog, CatalogError> {
        // TODO: enforce at-least-one-section assertion.
        // TODO: enforce required-trailer assertion.
        let root = &self
            .sections
            .first()
            .expect("FIXME: We always assert at least one section.")
            .trailer
            .root;
        let catalog = self
            .object(
                root.index
                    .try_into()
                    .expect("FIXME: replace u32 in data model with usize"),
            )
            .unwrap()
            .indirect()
            .unwrap()
            .object
            .dictionary()
            .unwrap();

        Catalog::new_with(self, catalog)
    }

    pub fn dereference(&self, reference: &Reference) -> Option<&Object> {
        self.sections.iter().find_map(|s| {
            s.objects
                .get(&reference.index.try_into().unwrap())
                .and_then(Object::indirect)
                .filter(|io| io.generation == reference.generation)
                .map(|io| &*io.object)
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfSection {
    /// Mapping from object number to object
    pub(crate) objects: FnvHashMap<usize, Object>,
    pub(crate) trailer: Trailer,
    pub(crate) xref: Xref,
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
