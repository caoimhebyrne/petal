/// A reference to a piece of data in the data section.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DataSectionReference {
    /// The index of the item in the data section.
    pub index: usize,
}
