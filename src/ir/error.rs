use std::fmt::Display;

use crate::{
    ast::node::{expression::Expression, statement::Statement},
    core::location::Location,
};

#[derive(Debug, Clone)]
pub enum IRErrorKind {
    // The IR only supports functions as top-level definitions.
    UnsupportedTopLevelStatement(Statement),

    // The IR generator attempted to start a new function scope without ending
    // the previous one.
    UnterminatedFunctionScope,

    // The IR generator expected to be within a function scope, but was not.
    ExpectedFunctionScope,

    // The provided statement is unsupported.
    UnsupportedStatement(Statement),

    // The provided expression is unsupported.
    UnsupportedExpression(Expression),
}

#[derive(Debug, Clone)]
pub struct IRError {
    pub kind: IRErrorKind,
    pub location: Option<Location>,
}

impl IRError {
    pub fn new(kind: IRErrorKind, location: Option<Location>) -> Self {
        Self { kind, location }
    }
}

impl Display for IRError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            IRErrorKind::UnsupportedTopLevelStatement(statement) => {
                write!(f, "Unsupported top-level statement: {:?}", statement)
            }

            IRErrorKind::UnterminatedFunctionScope => write!(
                f,
                "An attempt was made to start a new function scope, but the last one wasn't ended?"
            ),

            IRErrorKind::ExpectedFunctionScope => write!(f, "Expected to be in a function scope, but none was present"),

            IRErrorKind::UnsupportedStatement(statement) => write!(f, "Unsupported statement: {:?}", statement),
            IRErrorKind::UnsupportedExpression(expression) => write!(f, "Unsupported expression: {:?}", expression),
        }
    }
}
