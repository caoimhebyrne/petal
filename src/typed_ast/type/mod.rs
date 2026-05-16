use crate::typed_ast::r#type::db::{
    DefinedTypeId,
    TypeId,
};

pub(super) mod db;
pub(super) mod defined;

/// A type on a node within the typed AST.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Type {
    /// A reference to a defined type (e.g. struct, enum).
    Defined(DefinedTypeId),

    /// A generic type which needs to be substituted for its concrete type when available.
    Generic(usize),

    /// A signed integer of a certain size (i{x}).
    SignedInteger(u8),

    /// A reference of another type.
    Reference(TypeId),

    /// An unsigned integer of a certain size (u{x}).
    UnsignedInteger(u8),

    /// Unit, typically the result of a function call.
    Void,
}
