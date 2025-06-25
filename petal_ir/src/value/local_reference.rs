/// A reference to a local variable or parameter.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LocalReference {
    /// The index of the local.
    pub index: usize,

    /// Whether this is referencing a parameter.
    pub is_parameter: bool,
}
