use crate::{
    ast::{
        expression::{
            Expression,
            ExpressionKind,
        },
        statement::StatementKind,
        type_expr::GenericTypeArgument,
    },
    core::span::Span,
    typechecker::r#type::FunctionReference,
};

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    /// The resolved reference for the calee of this call.
    pub resolved_callee: Option<FunctionReference>,

    /// The expression providing the function to call.
    pub callee: Box<Expression>,

    /// The arguments being passed to the function.
    pub arguments: Vec<FunctionCallArgument>,

    /// The generic type arguments being passed to the function.
    pub generic_type_arguments: Vec<GenericTypeArgument>,
}

impl FunctionCall {
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

    /// The generic type arguments being passed to the function.
    generic_type_arguments: Vec<GenericTypeArgument>,
}

impl FunctionCallBuilder {
    /// Creates a new [`FunctionCall`].
    pub fn new(callee: Expression) -> Self {
        Self { callee, arguments: vec![], generic_type_arguments: vec![] }
    }

    /// Adds an argument to this function call.
    pub fn argument(mut self, name: Option<String>, value: Expression, span: Span) -> Self {
        self.arguments.push(FunctionCallArgument { name, value, span });
        self
    }

    /// Sets the generic type arguments of this function call.
    pub fn generic_type_arguments(mut self, generic_type_arguments: Vec<GenericTypeArgument>) -> Self {
        self.generic_type_arguments = generic_type_arguments;
        self
    }

    /// Builds this [`FunctionCallBuilder`] into a [`FunctionCall`].
    pub fn build(self) -> FunctionCall {
        FunctionCall {
            resolved_callee: None,
            callee: self.callee.into(),
            arguments: self.arguments,
            generic_type_arguments: self.generic_type_arguments,
        }
    }
}
