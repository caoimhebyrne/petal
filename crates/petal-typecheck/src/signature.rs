use crate::TypeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Signature {
    /// The type that the function returns.
    pub return_type: TypeId,
}
