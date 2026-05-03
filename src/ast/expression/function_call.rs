use crate::{
    ast::{
        expression::{
            Expression,
            ExpressionKind,
        },
        statement::StatementKind,
    },
    core::span::Span,
    typechecker::context::FunctionId,
};

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    /// The resolved ID for the calee of this call.
    pub resolved_callee: Option<FunctionId>,

    /// The expression providing the function to call.
    pub callee: Box<Expression>,

    /// The arguments being passed to the function.
    pub arguments: Vec<FunctionCallArgument>,
}

impl FunctionCall {
    /// Creates a new [`FunctionCall`].
    pub fn new(callee: Expression, arguments: Vec<FunctionCallArgument>) -> Self {
        Self { resolved_callee: None, callee: callee.into(), arguments }
    }

    /// Creates a new [`FunctionCallBuilder`].
    pub fn builder(function_path: Expression) -> FunctionCallBuilder {
        FunctionCallBuilder::new(function_path)
    }
}

/// An argument being passed during a function call.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallArgument {
    /// The name provided for the argument.
    pub name: Option<String>,

    /// The value provided for the argument.
    pub value: Expression,

    /// The span that this argument occurred at.
    pub span: Span,
}

impl From<FunctionCall> for ExpressionKind {
    fn from(value: FunctionCall) -> Self {
        Self::FunctionCall(value)
    }
}

impl From<FunctionCall> for StatementKind {
    fn from(value: FunctionCall) -> Self {
        Self::FunctionCall(value)
    }
}

/// A builder for a [`FunctionCall`].
pub struct FunctionCallBuilder {
    /// The expression providing the function to call.
    callee: Expression,

    /// The arguments being passed to the function
    arguments: Vec<FunctionCallArgument>,
}

impl FunctionCallBuilder {
    /// Creates a new [`FunctionCall`].
    pub fn new(callee: Expression) -> Self {
        Self { callee, arguments: vec![] }
    }

    /// Adds an argument to this function call.
    pub fn argument(mut self, name: Option<String>, value: Expression, span: Span) -> Self {
        self.arguments.push(FunctionCallArgument { name, value, span });
        self
    }

    /// Builds this [`FunctionCallBuilder`] into a [`FunctionCall`].
    pub fn build(self) -> FunctionCall {
        FunctionCall::new(self.callee, self.arguments)
    }
}
