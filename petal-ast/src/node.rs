//! This module contains definitions of nodes that are compatible as both statements and expressions.

use petal_core::string_intern::StringReference;

use crate::{
    expression::{ExpressionNode, ExpressionNodeKind},
    statement::StatementNodeKind,
};

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    /// The name of the function that is being called.
    pub name: StringReference,

    /// The values being passed as arguments to the function.
    pub arguments: Vec<ExpressionNode>,
}

impl FunctionCall {
    /// Instantiates a new [FunctionCall].
    pub fn new(name: StringReference, arguments: Vec<ExpressionNode>) -> Self {
        FunctionCall { name, arguments }
    }
}

impl From<FunctionCall> for StatementNodeKind {
    fn from(val: FunctionCall) -> Self {
        StatementNodeKind::FunctionCall(val)
    }
}

impl From<FunctionCall> for ExpressionNodeKind {
    fn from(val: FunctionCall) -> Self {
        ExpressionNodeKind::FunctionCall(val)
    }
}
