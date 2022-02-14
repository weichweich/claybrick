pub use catalog::{Catalog, CatalogError};

use crate::pdf::{Dictionary, Object};

pub mod catalog;
pub mod pages;

/// Dictionary type names
pub(crate) mod dict_types {
    pub const OBJECT_STREAM: &[u8] = b"ObjStm";
    pub const PAGES: &[u8] = b"Pages";
    pub const CATALOG: &[u8] = b"Catalog";
}

pub(crate) const K_TYPE: &[u8] = b"Type";
// parent key, for parent objects. not yet needed
// pub(crate) const K_PARENT: &[u8] = b"Parent";
pub(crate) const K_KIDS: &[u8] = b"Kids";
pub(crate) const K_COUNT: &[u8] = b"Count";
pub(crate) const K_VERSION: &[u8] = b"Version";
pub(crate) const K_PAGES: &[u8] = b"Pages";
pub(crate) const K_PAGES_LABEL: &[u8] = b"PagesLabel";
pub(crate) const K_NAME: &[u8] = b"Name";
pub(crate) const K_LENGTH: &[u8] = b"Length";
pub(crate) const K_STREAM_OBJECT_COUNT: &[u8] = b"N";
pub(crate) const K_FIRST: &[u8] = b"First";

fn require_type(dict: &Dictionary, t: &[u8]) -> Result<(), ()> {
    if let Some(k) = dict.get(K_TYPE).and_then(Object::name) {
        if &k[..] != t {
            log::warn!("Wrong dictionary type `{}`", k);
            Err(())
        } else {
            Ok(())
        }
    } else {
        log::warn!("Missing dictionary type");
        Err(())
    }
}
