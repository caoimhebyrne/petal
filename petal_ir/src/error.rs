use petal_core::core::location::Location;
use std::fmt::Display;

/// An error that occurred during the generation of the intermediate representation.
///
/// See [crate::generator::IRGenerator].
#[derive(Debug, Clone)]
pub struct IRError {
    /// The kind of error being returned.
    pub kind: IRErrorKind,

    /// The location that the error occurred at within the source file.
    pub location: Location,
}

/// The result of an IR-related function.
pub type IRResult<T> = Result<T, IRError>;

/// The different kinds of errors that can occur during intermediate representation generation.
#[derive(Debug, Clone)]
pub enum IRErrorKind {
    /// An unsupported top-level statement was encountered.
    UnsupportedTopLevelStatement,

    /// An attempt was made to start a new function scope while the previous one was still ongoing.
    UnterminatedFunctionScope,

    /// An attempt was made to use a function scope, but one wasn't started.
    ExpectedFunctionScope,

    /// Type information was missing.
    MissingTypeInformation,
}

impl IRError {
    pub fn new(kind: IRErrorKind, location: Location) -> IRError {
        IRError { kind, location }
    }

    pub fn unsupported_top_level_statement(location: Location) -> IRError {
        IRError::new(IRErrorKind::UnsupportedTopLevelStatement, location)
    }

    pub fn unterminated_function_scope(location: Location) -> IRError {
        IRError::new(IRErrorKind::UnterminatedFunctionScope, location)
    }

    pub fn expected_function_scope(location: Location) -> IRError {
        IRError::new(IRErrorKind::ExpectedFunctionScope, location)
    }

    pub fn missing_type_information(location: Location) -> IRError {
        IRError::new(IRErrorKind::MissingTypeInformation, location)
    }
}

impl Display for IRError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            IRErrorKind::UnsupportedTopLevelStatement => write!(
                f,
                "Encountered an unsupported top-level statement during IR generation. The intermediate representation only supports function definitions as top level statements at the moment."
            ),

            IRErrorKind::UnterminatedFunctionScope => write!(
                f,
                "An attempt was made to start a function scope, but the previous one was not ended. If you're trying to use nested functions - those are not supported at the moment."
            ),

            IRErrorKind::ExpectedFunctionScope => write!(
                f,
                "An attempt was made to use the current function scope, but one wasn't started. This is probably a bug..."
            ),

            IRErrorKind::MissingTypeInformation => write!(f, "Type information was missing for an expression"),
        }
    }
}
