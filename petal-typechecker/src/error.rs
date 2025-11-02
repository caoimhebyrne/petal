use enum_display::EnumDisplay;
use petal_ast::{
    expression::{Expression, ExpressionKind},
    statement::{Statement, StatementKind},
    r#type::{Type, TypeKind},
};
use petal_core::{
    error::{Error, ErrorKind},
    source_span::SourceSpan,
};

#[derive(Debug, PartialEq, EnumDisplay)]
pub enum TypecheckerError {
    /// An attempt was made to declare a function, but one already exists with the same name.
    #[display("A function already exists with the name: '{0}")]
    DuplicateFunctionDeclaration(String),

    /// An attempt was made to declare a varaible, but one already exists with the provided name.
    #[display("A variable already exists with the name: '{0}")]
    DuplicateVariableDeclaration(String),

    /// A function context was expected, but one was not available.
    #[display("Expected a function context, but one was not available -- this may be a compiler bug")]
    ExpectedFunctionContext,

    /// A type was expected, but a different type was received.
    #[display("Expected type '{expected}' but received type '{received}'")]
    ExpectedType { expected: TypeKind, received: TypeKind },

    /// A type could not be resolved by the typechecker.
    #[display("Unable to resolve type: '{0:?}")]
    UnableToResolveType(TypeKind),

    /// A function was referenced, but it has not been defined yet.
    #[display("Unknown function: '{0}'")]
    UndeclaredFunction(String),

    /// A variable was referenced, but it has not been defined yet.
    #[display("Unknown variable: '{0}'")]
    UndeclaredVariable(String),

    /// An expression is not supported by the typechecker yet.
    #[display("Unable to type-check expression: {0:?}")]
    UnsupportedExpression(ExpressionKind),

    /// A statement is not supported by the typechecker yet.
    #[display("Unable to type-check statement: {0:?}")]
    UnsupportedStatement(StatementKind),
}

impl TypecheckerError {
    /// Creates a new [Error] with the kind as a [TypecheckerError::DuplicateFunctionDeclaration] kind.
    pub fn duplicate_function_declaration(name: &str, span: SourceSpan) -> Error {
        Error::new(TypecheckerError::DuplicateFunctionDeclaration(name.to_owned()), span)
    }

    /// Creates a new [Error] with the kind as a [TypecheckerError::DuplicateVariableDeclaration] kind.
    pub fn duplicate_variable_declaration(name: &str, span: SourceSpan) -> Error {
        Error::new(TypecheckerError::DuplicateVariableDeclaration(name.to_owned()), span)
    }

    /// Creates a new [Error] with the kind as a [TypecheckerError::ExpectedFunctionContext] kind.
    pub fn expected_function_context(span: SourceSpan) -> Error {
        Error::new(TypecheckerError::ExpectedFunctionContext, span)
    }

    /// Creates a new [Error] with the kind as a [TypecheckerError::ExpectedType] kind.
    pub fn expected_type(expected: TypeKind, received: TypeKind, span: SourceSpan) -> Error {
        Error::new(TypecheckerError::ExpectedType { expected, received }, span)
    }

    /// Creates a new [Error] with the kind as a [TypecheckerError::UnableToResolveType] kind.
    pub fn unable_to_resolve_type(r#type: &Type) -> Error {
        Error::new(TypecheckerError::UnableToResolveType(r#type.kind), r#type.span)
    }

    /// Creates a new [Error] with the kind as a [TypecheckerError::UndeclaredFunction] kind.
    pub fn undeclared_function(name: &str, span: SourceSpan) -> Error {
        Error::new(TypecheckerError::UndeclaredFunction(name.to_owned()), span)
    }

    /// Creates a new [Error] with the kind as a [TypecheckerError:UndeclaredVariable] kind.
    pub fn undeclared_variable(name: &str, span: SourceSpan) -> Error {
        Error::new(TypecheckerError::UndeclaredVariable(name.to_owned()), span)
    }

    /// Creates a new [Error] with the kind as a [TypecheckerError::UnsupportedExpression] kind.
    pub fn unsupported_expression(expression: Expression) -> Error {
        Error::new(
            TypecheckerError::UnsupportedExpression(expression.kind),
            expression.span,
        )
    }

    /// Creates a new [Error] with the kind as a [TypecheckerError::UnsupportedStatement] kind.
    pub fn unsupported_statement(statement: Statement) -> Error {
        Error::new(TypecheckerError::UnsupportedStatement(statement.kind), statement.span)
    }
}

impl ErrorKind for TypecheckerError {}
