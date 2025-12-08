use enum_display::EnumDisplay;
use inkwell::builder::BuilderError;
use petal_ast::{
    expression::{ExpressionNode, ExpressionNodeKind},
    statement::{StatementNode, StatementNodeKind, TopLevelStatementNode, TopLevelStatementNodeKind},
};
use petal_core::{
    error::{Error, ErrorKind, Result},
    source_span::SourceSpan,
    r#type::{ResolvedType, TypeId, TypeReference},
};

#[derive(Debug, PartialEq, EnumDisplay)]
pub enum LLVMCodegenError {
    #[display(
        "A variable was referenced that has not yet been declared: '{0}'. This should have been caught by the typechecker!"
    )]
    UndeclaredVariable(String),

    #[display(
        "A function was referenced that has not yet been declared: '{0}'. This should have been caught by the typechecker!"
    )]
    UndeclaredFunction(String),

    #[display("Cannot assign a value to a parameter")]
    UnableToAssignToParameter,

    #[display("Encountered an unresolved type ({0:?}), this should have been handled by the typechecker!")]
    UnresolvedType(TypeId),

    #[display("Unable to generate code for expression: {0:?}")]
    UnprocessableExpression(ExpressionNodeKind),

    #[display("Unable to generate code for statement: {0:?}")]
    UnprocessableStatement(StatementNodeKind),

    #[display("Unable to generate code for top-level statement: {0:?}")]
    UnprocessableTopLevelStatement(TopLevelStatementNodeKind),

    #[display("The type {0:?} cannot be used in this context during code-generation")]
    UnprocessableType(ResolvedType),

    #[display("A scope context was created, but none was available. This is most likely a compiler bug.")]
    MissingScopeContext,

    #[display("LLVM returned an error while generating code: '{0}'")]
    CodegenError(String),
}

impl LLVMCodegenError {
    /// Initializes an [Error] with the [LLVMCodegenError::UndeclaredVariable] kind.
    pub fn undeclared_variable(name: &str, span: SourceSpan) -> Error {
        Error::new(LLVMCodegenError::UndeclaredVariable(name.into()), span)
    }

    /// Initializes an [Error] with the [LLVMCodegenError::UndeclaredFunction] kind.
    pub fn undeclared_function(name: &str, span: SourceSpan) -> Error {
        Error::new(LLVMCodegenError::UndeclaredFunction(name.into()), span)
    }

    /// Initializes an [Error] with the [LLVMCodegenError::UnableToAssignToParameter] kind.
    pub fn unable_to_assign_to_parameter(span: SourceSpan) -> Error {
        Error::new(LLVMCodegenError::UnableToAssignToParameter, span)
    }

    /// Initializes an [Error] with the [LLVMCodegenError::UnresolvedType] kind.
    pub fn unresolved_type(reference: &TypeReference) -> Error {
        Error::new(LLVMCodegenError::UnresolvedType(reference.id), reference.span)
    }

    /// Initializes an [Error] with the [LLVMCodegenError::UnprocessableExpression] kind.
    pub fn unprocessable_expression(expression: &ExpressionNode) -> Error {
        Error::new(
            LLVMCodegenError::UnprocessableExpression(expression.kind.clone()),
            expression.span,
        )
    }

    /// Initializes an [Error] with the [LLVMCodegenError::UnprocessableStatement] kind.
    pub fn unprocessable_statement(statement: &StatementNode) -> Error {
        Error::new(
            LLVMCodegenError::UnprocessableStatement(statement.kind.clone()),
            statement.span,
        )
    }

    /// Initializes an [Error] with the [LLVMCodegenError::UnprocessableTopLevelStatement] kind.
    pub fn unprocessable_top_level_statement(statement: &TopLevelStatementNode) -> Error {
        Error::new(
            LLVMCodegenError::UnprocessableTopLevelStatement(statement.kind.clone()),
            statement.span,
        )
    }

    /// Initializes an [Error] with the [LLVMCodegenError::UnprocessableType] kind.
    pub fn unprocessable_type(r#type: ResolvedType, span: SourceSpan) -> Error {
        Error::new(LLVMCodegenError::UnprocessableType(r#type), span)
    }

    /// Initializes an [Error] with the [LLVMCodegenError::MissingScopeContext] kind.
    pub fn missing_scope_context(span: SourceSpan) -> Error {
        Error::new(LLVMCodegenError::MissingScopeContext, span)
    }

    /// Initializes an [Error] with the [LLVMCodegenError::CodegenError] kind.
    pub fn codegen_error(message: String, span: SourceSpan) -> Error {
        Error::new(LLVMCodegenError::CodegenError(message), span)
    }
}

pub trait IntoCodegenResult<T> {
    fn into_codegen_result(self, span: SourceSpan) -> Result<T>;
}

impl<T> IntoCodegenResult<T> for std::result::Result<T, BuilderError> {
    fn into_codegen_result(self, span: SourceSpan) -> Result<T> {
        self.map_err(|it| LLVMCodegenError::codegen_error(it.to_string(), span))
    }
}

impl ErrorKind for LLVMCodegenError {}
