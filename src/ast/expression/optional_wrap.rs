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
