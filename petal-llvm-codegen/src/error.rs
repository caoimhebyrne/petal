use enum_display::EnumDisplay;
use petal_core::{
    error::{Error, ErrorKind},
    source_span::SourceSpan,
};

#[derive(Debug, PartialEq, EnumDisplay)]
pub enum LLVMCodegenErrorKind {
    #[display(
        "A variable was referenced that has not yet been declared: '{0}'. This should have been caught by the typechecker!"
    )]
    UndeclaredVariable(String),

    #[display("A scope context was created, but none was available. This is most likely a compiler bug.")]
    MissingScopeContext,
}

impl LLVMCodegenErrorKind {
    /// Initializes an [Error] with the [LLVMCodegenErrorKind::UndeclaredVariable] kind.
    pub fn undeclared_variable(name: &str, span: SourceSpan) -> Error {
        Error::new(LLVMCodegenErrorKind::UndeclaredVariable(name.into()), span)
    }

    /// Initializes an [Error] with the [LLVMCodegenErrorKind::MissingScopeContext] kind.
    pub fn missing_scope_context(span: SourceSpan) -> Error {
        Error::new(LLVMCodegenErrorKind::MissingScopeContext, span)
    }
}

impl ErrorKind for LLVMCodegenErrorKind {}
