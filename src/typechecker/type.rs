use std::fmt::Display;

use crate::typechecker::context::{
    SpecializedStructureId,
    StructureId,
};

/// The ID corresponding to a user-defined or specialized structure.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StructureReference {
    /// A referrence to a user-defined structure.
    Plain(StructureId),

    /// A reference to a specialized variant of a structure.
    Specialized(SpecializedStructureId),
}

impl Display for StructureReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plain(plain) => plain.fmt(f),
            Self::Specialized(specialized) => specialized.fmt(f),
        }
    }
}

/// A type as defined by the typechecker.
#[derive(Debug, Clone, PartialEq, Default, Eq, Hash)]
pub enum Type {
    /// An unsigned integer.
    UnsignedInteger(u8),

    /// A signed integer.
    SignedInteger(u8),

    /// A boolean (true or false).
    Boolean,

    /// The nothing type.
    Void,

    /// An optional wrapping another type.
    Optional(Box<Type>),

    /// A reference to another type.
    Reference(Box<Type>),

    /// A structure type.
    Structure(StructureReference),

    /// A generic type that needs to be resolved.
    GenericType(usize),

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
            Self::Optional(wrapped) => write!(f, "?{wrapped}"),
            Self::Structure(id) => write!(f, "<structure {id}>"),
            Self::GenericType(index) => write!(f, "<generic type @ {index}>"),
            Self::Unknown => write!(f, "<unknown>"),
        }
    }
}
