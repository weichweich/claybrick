pub use catalog::{Catalog, CatalogError};

use crate::pdf::{Dictionary, Object};

pub mod catalog;
pub mod pages;

const K_TYPE: &[u8] = b"Type";

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
