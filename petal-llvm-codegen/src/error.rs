use std::fmt::Display;

use inkwell::builder::BuilderError;
use petal_ast::{
    expression::{Expression, ExpressionKind},
    statement::{Statement, StatementKind},
    r#type::ResolvedTypeKind,
};
use petal_core::{
    error::{Error, ErrorKind},
    source_span::SourceSpan,
    string_intern::StringReference,
};

#[derive(Debug, PartialEq)]
pub enum LLVMCodegenErrorKind {
    /// A type was encountered that was not resolved.
    UnresolvedType(String),

    /// A type was encountered that cannot be used for a value.
    BadValueType(ResolvedTypeKind),

    /// A string reference was encountered that could not be resolved.
    UnresolvedStringReference(StringReference),

    /// A statement could not be converted into code.
    UnableToCodegenStatement(StatementKind),

    /// An expression could not be converted into code.
    UnableToCodegenExpression(ExpressionKind),

    /// An error occurred while building the LLVM bytecode.
    BuilderError(BuilderError),

    /// A scope context was expected, but one was not bound.
    MissingScopeContext,

    /// A variable was referenced that was not declared.
    UndeclaredVariable(StringReference),
}

impl LLVMCodegenErrorKind {
    pub fn unresolved_type(name: &str, span: SourceSpan) -> Error {
        Error::new(LLVMCodegenErrorKind::UnresolvedType(name.to_owned()), span)
    }

    pub fn bad_value_type(kind: ResolvedTypeKind, span: SourceSpan) -> Error {
        Error::new(LLVMCodegenErrorKind::BadValueType(kind), span)
    }

    pub fn unresolved_string_reference(reference: &StringReference, span: SourceSpan) -> Error {
        Error::new(LLVMCodegenErrorKind::UnresolvedStringReference(*reference), span)
    }

    pub fn unable_to_codegen_statement(statement: &Statement) -> Error {
        Error::new(
            LLVMCodegenErrorKind::UnableToCodegenStatement(statement.kind.clone()),
            statement.span,
        )
    }

    pub fn unable_to_codegen_expression(expression: &Expression) -> Error {
        Error::new(
            LLVMCodegenErrorKind::UnableToCodegenExpression(expression.kind.clone()),
            expression.span,
        )
    }

    pub fn builder_error(error: BuilderError, span: SourceSpan) -> Error {
        Error::new(LLVMCodegenErrorKind::BuilderError(error), span)
    }

    pub fn missing_scope_context(span: SourceSpan) -> Error {
        Error::new(LLVMCodegenErrorKind::MissingScopeContext, span)
    }

    pub fn undeclared_variable(identifier: StringReference, span: SourceSpan) -> Error {
        Error::new(LLVMCodegenErrorKind::UndeclaredVariable(identifier), span)
    }
}

impl Display for LLVMCodegenErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLVMCodegenErrorKind::UnresolvedType(name) => {
                write!(f, "An unresolved type was encountered: '{}', this is a a bug", name)
            }

            LLVMCodegenErrorKind::BadValueType(kind) => {
                write!(f, "Unable to convert type '{}' to an LLVM value type", kind)
            }

            LLVMCodegenErrorKind::UnresolvedStringReference(reference) => {
                write!(
                    f,
                    "An unresolvable string reference was encountered: '{:?}', this is a bug",
                    reference
                )
            }

            LLVMCodegenErrorKind::UnableToCodegenStatement(kind) => {
                write!(f, "Unable to codegen statement '{:?}'", kind)
            }

            LLVMCodegenErrorKind::UnableToCodegenExpression(kind) => {
                write!(f, "Unable to codegen expression '{:?}'", kind)
            }

            LLVMCodegenErrorKind::BuilderError(error) => {
                write!(f, "LLVM builder error: '{}'", error)
            }

            LLVMCodegenErrorKind::MissingScopeContext => {
                write!(f, "A scope context was expected, but one was not found, this is a bug")
            }

            LLVMCodegenErrorKind::UndeclaredVariable(identifier) => {
                write!(
                    f,
                    "A variable ({:?}) was referenced, but it has not been declared yet, this is a bug",
                    identifier
                )
            }
        }
    }
}

impl ErrorKind for LLVMCodegenErrorKind {}
