use crate::pdf::{document::require_type, Array, Dictionary, Object, RawPdf};

use super::{dict_types::PAGES, K_COUNT, K_KIDS};

pub enum PagesError {
    InvalidParent,
    MissingKids,
    InvalidKids,
    MissingCount,
    InvalidCount,
}

pub struct Pages<'a> {
    // storing a backref to the whole raw pdf document might be helpful to resolve objects etc.
    // raw_pdf: &'a RawPdf,
    // pages have a backref to their parent. We might want to store that here.
    // parent: Option<&'a IndirectObject>,
    /// PageTree or Page objects, indirect.
    kids: &'a Array,
    /// Number of leafs.
    count: usize,
}

impl<'a> Pages<'a> {
    pub(crate) fn new_with(raw_pdf: &'a RawPdf, dict: &'a Dictionary) -> Result<Self, PagesError> {
        let _ = require_type(dict, PAGES);

        let pages = Self {
            // raw_pdf,
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
