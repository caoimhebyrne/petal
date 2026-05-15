use crate::ast::expression::{
    Expression,
    ExpressionKind,
};

#[derive(Debug, Clone, PartialEq)]
pub struct OptionalWrap {
    /// The value being wrapped in an optional.
    pub inner_value: Box<Expression>,
}

impl OptionalWrap {
    /// Creates a new [`MemberAccess`].
    pub fn new(inner_value: Expression) -> Self {
        Self { inner_value: inner_value.into() }
    }
}

impl From<OptionalWrap> for ExpressionKind {
    fn from(value: OptionalWrap) -> Self {
        Self::OptionalWrap(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OptionalHasValue {
    /// The optional that a value is being checked for on.
    pub optional_value: Box<Expression>,
}

impl OptionalHasValue {
    /// Creates a new [`OptionalHasValue`].
    pub fn new(optional_value: Expression) -> Self {
        Self { optional_value: optional_value.into() }
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
}

impl OptionalForceUnwrap {
    /// Creates a new [`OptionalForceUnwrap`].
    pub fn new(optional_value: Expression) -> Self {
        Self { optional_value: optional_value.into() }
    }
}

impl From<OptionalForceUnwrap> for ExpressionKind {
    fn from(value: OptionalForceUnwrap) -> Self {
        Self::OptionalForceUnwrap(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OptionalUnwrap {
    /// The optional that is being unwrapped.
    pub optional_value: Box<Expression>,
}

impl OptionalUnwrap {
    /// Creates a new [`OptionalUnwrap`].
    pub fn new(optional_value: Expression) -> Self {
        Self { optional_value: optional_value.into() }
    }
}

impl From<OptionalUnwrap> for ExpressionKind {
    fn from(value: OptionalUnwrap) -> Self {
        Self::OptionalUnwrap(value)
    }
}
