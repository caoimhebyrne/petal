use crate::ast::expression::{
    Expression,
    ExpressionKind,
};

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    /// The name of the function being called.
    pub name: String,

    /// The arguments being passed to the function.
    pub arguments: Vec<Expression>,
}

impl FunctionCall {
    /// Creates a new [`FunctionCall`].
    pub fn new(name: impl Into<String>, arguments: Vec<Expression>) -> Self {
        Self { name: name.into(), arguments }
    }

    /// Creates a new [`FunctionCallBuilder`].
    pub fn builder(name: impl Into<String>) -> FunctionCallBuilder {
        FunctionCallBuilder::new(name)
    }
}

impl From<FunctionCall> for ExpressionKind {
    fn from(value: FunctionCall) -> Self {
        Self::FunctionCall(value)
    }
}

/// A builder for a [`FunctionCall`].
pub struct FunctionCallBuilder {
    /// The name of the function being called.
    name: String,

    /// The arguments being passed to the function
    arguments: Vec<Expression>,
}

impl FunctionCallBuilder {
    /// Creates a new [`FunctionCall`].
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), arguments: vec![] }
    }

    /// Adds an argument to this function call.
    pub fn argument(mut self, argument: Expression) -> Self {
        self.arguments.push(argument);
        self
    }

    /// Builds this [`FunctionCallBuilder`] into a [`FunctionCall`].
    pub fn build(self) -> FunctionCall {
        FunctionCall::new(self.name, self.arguments)
    }
}
