/// A type as defined by the typechecker.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum Type {
    /// An unsigned integer.
    UnsignedInteger(u8),

    /// A signed integer.
    SignedInteger(u8),

    /// The nothing type.
    Void,

    /// The type has not been resolved for this element yet.
    #[default]
    Unknown,
}
