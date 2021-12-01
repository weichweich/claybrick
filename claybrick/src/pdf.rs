#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pdf {
    pub(crate) version: (u8, u8),
    pub(crate) announced_binary: bool,
}
