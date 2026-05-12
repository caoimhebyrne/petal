use crate::{
    ast::expression::{
        Expression,
        ExpressionKind,
    },
    typechecker::r#type::Type,
};

#[derive(Debug, Clone, PartialEq)]
pub struct OptionalWrap {
    /// The value being wrapped in an optional.
    pub inner_value: Box<Expression>,

    /// The type of the inner value.
    pub inner_type: Type,
}

impl OptionalWrap {
    /// Creates a new [`MemberAccess`].
    pub fn new(inner_value: Expression) -> Self {
        Self { inner_value: inner_value.into(), inner_type: Type::Unknown }
    }
}

impl From<OptionalWrap> for ExpressionKind {
    fn from(value: OptionalWrap) -> Self {
        Self::OptionalWrap(value)
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct OptionalEmpty {
    /// The type of the inner value.
    pub inner_type: Type,
}

impl From<OptionalEmpty> for ExpressionKind {
    fn from(value: OptionalEmpty) -> Self {
        Self::OptionalEmpty(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OptionalHasValue {
    /// The optional that a value is being checked for on.
    pub optional_value: Box<Expression>,

    /// The type of the inner value.
    pub inner_type: Type,
}

impl OptionalHasValue {
    /// Creates a new [`OptionalHasValue`].
    pub fn new(optional_value: Expression) -> Self {
        Self { optional_value: optional_value.into(), inner_type: Type::Unknown }
    }
}

impl From<OptionalHasValue> for ExpressionKind {
    fn from(value: OptionalHasValue) -> Self {
        Self::OptionalHasValue(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OptionalForceUnwrap {
    /// The optional that is being unwrapped.
    pub optional_value: Box<Expression>,

    /// The type of the inner value.
    pub inner_type: Type,
}

impl OptionalForceUnwrap {
    /// Creates a new [`OptionalForceUnwrap`].
    pub fn new(optional_value: Expression) -> Self {
        Self { optional_value: optional_value.into(), inner_type: Type::Unknown }
    }
}

impl From<OptionalForceUnwrap> for ExpressionKind {
    fn from(value: OptionalForceUnwrap) -> Self {
        Self::OptionalForceUnwrap(value)
    }
}
