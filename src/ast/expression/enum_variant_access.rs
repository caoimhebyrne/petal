use crate::{
    ast::expression::ExpressionKind,
    typechecker::context::EnumId,
};

#[derive(Debug, Clone, PartialEq)]
pub struct EnumMemberAccess {
    /// The ID of the enum type that the member belongs to.
    pub enum_id: EnumId,

    /// The index of the variant within the enum.
    pub variant_index: usize,
}

impl From<EnumMemberAccess> for ExpressionKind {
    fn from(value: EnumMemberAccess) -> Self {
        Self::EnumMemberAccess(value)
    }
}
