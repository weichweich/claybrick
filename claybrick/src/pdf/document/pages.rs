use crate::pdf::{document::require_type, Array, Dictionary, IndirectObject, Object, RawPdf};

const T_PAGES: &[u8] = b"Pages";

const K_PARENT: &[u8] = b"Parent";
const K_KIDS: &[u8] = b"Kids";
const K_COUNT: &[u8] = b"Count";

pub enum PagesError {
    InvalidParent,
    MissingKids,
    InvalidKids,
    MissingCount,
    InvalidCount,
}

pub struct Pages<'a> {
    raw_pdf: &'a RawPdf,
    parent: Option<&'a IndirectObject>,
    /// PageTree or Page objects, indirect.
    kids: &'a Array,
    /// Number of leafs.
    count: usize,
}

impl<'a> Pages<'a> {
    pub(crate) fn new_with(raw_pdf: &'a RawPdf, dict: &'a Dictionary) -> Result<Self, PagesError> {
        let _ = require_type(dict, T_PAGES);

        let pages = Self {
            raw_pdf,
            parent: None,
            kids: match dict.get(K_KIDS).ok_or(PagesError::MissingKids)? {
                Object::Array(a) => Ok(a),
                Object::Reference(r) => raw_pdf
                    .dereference(r)
                    .and_then(Object::array)
                    .ok_or(PagesError::InvalidKids),
                _ => Err(PagesError::InvalidKids),
            }?,
            count: dict
                .get(K_COUNT)
                .ok_or(PagesError::MissingKids)?
                .integer()
                .ok_or(PagesError::InvalidCount)?
                .try_into()
                .map_err(|_| PagesError::InvalidCount)?,
        };

        if pages.count < pages.kids.len() {
            log::error!(
                "Invalid child count. Got {} children but count is {}",
                pages.kids.len(),
                pages.count
            );
            return Err(PagesError::InvalidCount);
        }

        Ok(pages)
    }
}
