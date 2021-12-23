use std::ops::{Deref, DerefMut};

use super::Object;

#[derive(Debug, Clone, PartialEq)]
pub struct Array(Vec<Object>);

impl Array {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl Deref for Array {
    type Target = Vec<Object>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Array {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<Object>> for Array {
    fn from(objects: Vec<Object>) -> Self {
        Self(objects)
    }
}

impl std::fmt::Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Array [")?;
        for obj in self.iter() {
            write!(f, "\n  {}", obj)?;
        }
        write!(f, "]")?;
        Ok(())
    }
}
