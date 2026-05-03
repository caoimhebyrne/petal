use crate::{
    ast::{
        expression::{
            Expression,
            ExpressionKind,
        },
        statement::StatementKind,
    },
    core::span::Span,
};

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    /// The name of the function being called.
    pub name: String,

    /// The arguments being passed to the function.
    pub arguments: Vec<FunctionCallArgument>,
}

impl FunctionCall {
    /// Creates a new [`FunctionCall`].
    pub fn new(name: impl Into<String>, arguments: Vec<FunctionCallArgument>) -> Self {
        Self { name: name.into(), arguments }
    }

    /// Creates a new [`FunctionCallBuilder`].
    pub fn builder(name: impl Into<String>) -> FunctionCallBuilder {
        FunctionCallBuilder::new(name)
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
    /// The name of the function being called.
    name: String,

    /// The arguments being passed to the function
    arguments: Vec<FunctionCallArgument>,
}

impl FunctionCallBuilder {
    /// Creates a new [`FunctionCall`].
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), arguments: vec![] }
    }

    /// Adds an argument to this function call.
    pub fn argument(mut self, name: Option<String>, value: Expression, span: Span) -> Self {
        self.arguments.push(FunctionCallArgument { name, value, span });
        self
    }

    /// Builds this [`FunctionCallBuilder`] into a [`FunctionCall`].
    pub fn build(self) -> FunctionCall {
        FunctionCall::new(self.name, self.arguments)
    }
}
