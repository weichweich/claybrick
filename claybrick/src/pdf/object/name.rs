use std::{borrow::Borrow, ops::Deref};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Name(Vec<u8>);

impl Name {
    pub const fn new(n: Vec<u8>) -> Self {
        Self(n)
    }

    pub fn from_str(s: &str) -> Self {
        Self(s.as_bytes().to_owned())
    }
}

impl Borrow<[u8]> for Name {
    fn borrow(&self) -> &[u8] {
        &self.0[..]
    }
}

impl From<Vec<u8>> for Name {
    fn from(v: Vec<u8>) -> Self {
        Name(v)
    }
}

impl From<&[u8]> for Name {
    fn from(v: &[u8]) -> Self {
        Name(v.to_vec())
    }
}

impl Deref for Name {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Name")
            .field(&String::from_utf8_lossy(&self.0[..]))
            .finish()
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &String::from_utf8_lossy(&self.0[..]))
    }
}
