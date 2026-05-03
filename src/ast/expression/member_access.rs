use crate::ast::expression::{
    Expression,
    ExpressionKind,
};

#[derive(Debug, Clone, PartialEq)]
pub struct MemberAccess {
    /// The target of the member access.
    pub target: Box<Expression>,

    /// The name of the member being accessed on the target.
    pub name: String,
}

impl MemberAccess {
    /// Creates a new [`MemberAccess`].
    pub fn new(target: Expression, name: String) -> Self {
        Self { target: target.into(), name }
    }
}

impl From<MemberAccess> for ExpressionKind {
    fn from(value: MemberAccess) -> Self {
        Self::MemberAccess(value)
    }
}
