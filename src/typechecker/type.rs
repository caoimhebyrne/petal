use std::fmt::Display;

/// A type as defined by the typechecker.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum Type {
    /// An unsigned integer.
    UnsignedInteger(u8),

    /// A signed integer.
    SignedInteger(u8),

    /// A boolean (true or false).
    Boolean,

    /// The nothing type.
    Void,

    /// The type has not been resolved for this element yet.
    #[default]
    Unknown,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsignedInteger(bits) => write!(f, "u{bits}"),
            Self::SignedInteger(bits) => write!(f, "i{bits}"),
            Self::Boolean => write!(f, "bool"),
            Self::Void => write!(f, "void"),
            Self::Unknown => write!(f, "<unknown>"),
        }
    }
}
