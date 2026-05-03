use std::fmt::Display;

use crate::typechecker::context::StructureId;

/// A type as defined by the typechecker.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Type {
    /// An unsigned integer.
    UnsignedInteger(u8),

    /// A signed integer.
    SignedInteger(u8),

    /// A boolean (true or false).
    Boolean,

    /// The nothing type.
    Void,

    /// A reference to another type.
    Reference(Box<Type>),

    /// A structure type.
    Structure(StructureId),

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
            Self::Reference(referenced) => write!(f, "&{referenced}"),
            Self::Structure(id) => write!(f, "<structure {id}>"),
            Self::Unknown => write!(f, "<unknown>"),
        }
    }
}
