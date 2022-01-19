use crate::pdf::{object::Name, Dictionary, Object, RawPdf};

const K_VERSION: &[u8] = b"Version";
const K_PAGES: &[u8] = b"Pages";
const K_PAGES_LABEL: &[u8] = b"PagesLabel";
const K_NAME: &[u8] = b"Name";

#[derive(Debug, Clone, PartialEq)]
pub enum CatalogError {
    MissingPages,
}

#[derive(Clone, PartialEq)]
pub struct Catalog<'a> {
    raw_pdf: &'a RawPdf,
    version: Option<&'a Name>,
    pages: &'a Dictionary,
    pages_label: Option<&'a Dictionary>,
    names: Option<&'a Dictionary>,
    // dests: Option<&'a Dictionary>,
    // viewer_preferences: Option<&'a Dictionary>,
    // page_layout: Option<&'a Name>,
    // page_mode: Option<&'a Name>,
    // outlines: Option<&'a Dictionary>,
    // threads: Option<&'a Array>,
    // /// Array or dictionary
    // open_action: Option<&'a Object>,
    // additional_actions: Option<&'a Dictionary>,
    // uri: Option<&'a Dictionary>,
    // acro_form: Option<&'a Dictionary>,
    // metadata: Option<&'a Stream>,
    // structure_tree: Option<&'a Dictionary>,
    // mark_info: Option<&'a Dictionary>,
    // lang: Option<&'a CbString>,
    // spider_info: Option<&'a Dictionary>,
    // output_intents: Option<&'a Array>,
    // piece_info: Option<&'a Dictionary>,
    // optional_content_properties: Option<&'a Dictionary>,
    // permissions: Option<&'a Dictionary>,
    // legal: Option<&'a Dictionary>,
    // requirements: Option<&'a Array>,
    // collection: Option<&'a Dictionary>,
    // needs_rendering: Option<bool>,
}

// Custom impl to skip `raw_pdf` field.
impl<'a> std::fmt::Debug for Catalog<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Catalog")
            .field("version", &self.version)
            .field("pages", &self.pages)
            .field("pages_label", &self.pages_label)
            .field("names", &self.names)
            .finish()
    }
}

impl<'a> Catalog<'a> {
    pub(crate) fn new_with(raw_pdf: &'a RawPdf, dict: &'a Dictionary) -> Result<Self, CatalogError> {
        Ok(Self {
            raw_pdf,
            version: dict.get(K_VERSION).and_then(Object::name),
            pages: dict
                .get(K_PAGES)
                .and_then(|o| match o {
                    Object::Reference(r) => raw_pdf.dereference(r),
                    other => Some(other),
                })
                .and_then(Object::dictionary)
                .ok_or(CatalogError::MissingPages)
                .map_err(|e| {
                    log::error!("Missing `{}` key. Got {:?}", String::from_utf8_lossy(K_PAGES), dict);
                    e
                })?,
            pages_label: dict.get(K_PAGES_LABEL).and_then(Object::dictionary),
            names: dict.get(K_NAME).and_then(Object::dictionary),
        })
    }

    // pub fn pages(&self) -> Pages {

    // }
}
