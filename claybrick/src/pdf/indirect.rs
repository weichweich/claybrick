use std::fmt::Display;

use super::Object;

#[derive(Debug, Clone, PartialEq)]
pub struct IndirectObject {
    pub(crate) index: u32,
    pub(crate) generation: u32,
    pub(crate) object: Box<Object>,
}

impl Display for IndirectObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Indirect {} {} {{ {} }}", self.index, self.generation, self.object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reference {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}
