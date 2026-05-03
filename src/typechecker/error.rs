use std::fmt::Display;

use crate::{
    core::{
        error::Error,
        span::Span,
    },
    typechecker::r#type::Type,
};

/// An AST error.
#[derive(Debug, PartialEq)]
pub struct TypecheckerError {
    /// The kind of typechecker error that this is.
    pub kind: TypecheckerErrorKind,

    /// The [`Span`] that the error occurred at.
    pub span: Span,
}

/// The different kinds of [`TypecheckerError`]s that exist.
#[derive(Debug, PartialEq)]
pub enum TypecheckerErrorKind {
    AmbiguousFunctionCall(String),
    BinaryOperationNotSupported(Type),
    IncompatibleBinaryOperationTypes { left: Type, right: Type },
    IncompatibleVariableDeclarationTypes { declared: Type, value: Type },
    IncompatibleReturnTypes { declared: Type, value: Type },
    FunctionCallArgumentSizeMismatch { name: String, expected: usize, got: usize },
    MissingFunctionCallArgument { function_name: String, parameter_name: String },
    ExpectedPositionalFunctionCallArgument { parameter_name: String },
    DuplicateFunctionCallArgument(String),
    IncompatibleFunctionCallArgument { parameter_name: String, parameter_type: Type, argument_type: Type },
    IncompatibleTypes { expected: Type, got: Type },
    InvalidDereference(Type),
    DuplicateFunctionDeclaration(String),
    DuplicateVariableDeclaration(String),
    UndeclaredFunction(String),
    UndeclaredVariable(String),
    UnknownType(String),
    StructureInitializationRequiresStructureType(Option<Type>),
    StructureInitializationMissingFields { expected: usize, got: usize },
    MemberAccessNotSupported,
    TypeDoesNotHaveMember { r#type: Type, name: String },
    UnsupportedFunctionCallee,
    VariableDeclarationMissingInitialValue,
}

impl TypecheckerErrorKind {
    /// Returns an [TypecheckerError] from this [TypecheckerErrorKind] at the provided [Span].
    pub fn at(self, span: Span) -> TypecheckerError {
        TypecheckerError { kind: self, span }
    }
}

impl TypecheckerError {
    /// Creates a new [`TypecheckerError`].
    pub fn new(kind: TypecheckerErrorKind, span: Span) -> Self {
        TypecheckerError { kind, span }
    }
}

impl Display for TypecheckerErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AmbiguousFunctionCall(name) => {
                write!(f, "More than one function exists with the name '{name}', consider renaming one of them")
            }

            Self::BinaryOperationNotSupported(r#type) => {
                write!(f, "Binary operations are not supported on the type '{}'", r#type)
            }

            Self::UnknownType(name) => write!(f, "Unknown type: '{name}'"),

            Self::IncompatibleBinaryOperationTypes { left, right } => {
                write!(f, "Binary operation has two values of incompatible types: '{}' and '{}'", left, right)
            }

            Self::IncompatibleVariableDeclarationTypes { declared, value } => {
                write!(f, "Unable to assign value of type '{}' to variable of type '{}'", value, declared)
            }

            Self::IncompatibleReturnTypes { declared, value } => {
                write!(f, "Unable to return value of type '{}' from function with return type '{}'", value, declared)
            }

            Self::UndeclaredFunction(name) => write!(f, "Function '{name}' has not been declared yet"),

            Self::UndeclaredVariable(name) => write!(f, "Variable '{name}' has not been declared yet"),

            Self::DuplicateFunctionCallArgument(name) => {
                write!(f, "Argument '{name}' has more than one value in this function call, this is not allowed")
            }

            Self::FunctionCallArgumentSizeMismatch { name, expected, got } => write!(
                f,
                "Function '{name}' has {expected} parameter(s), but {got} argument(s) passed in function call",
            ),

            Self::InvalidDereference(r#type) => write!(f, "Unable to dereference value of type '{type}'"),

            Self::MissingFunctionCallArgument { function_name, parameter_name } => {
                write!(
                    f,
                    "No value was provided for parameter named '{parameter_name}' in call to function '{function_name}'"
                )
            }

            Self::ExpectedPositionalFunctionCallArgument { parameter_name } => {
                write!(
                    f,
                    "Parameter '{parameter_name}' is a positional parameter, an explicit name must not be provided"
                )
            }

            Self::IncompatibleFunctionCallArgument { parameter_name, parameter_type, argument_type } => write!(
                f,
                "Parameter '{parameter_name}' has type '{}', but got argument of type '{}'",
                parameter_type, argument_type
            ),

            Self::DuplicateFunctionDeclaration(name) => write!(f, "A function named '{name}' already exists"),

            Self::DuplicateVariableDeclaration(name) => {
                write!(f, "A variable named '{name}' already exists in this scope")
            }

            Self::IncompatibleTypes { expected, got } => {
                write!(f, "Expected type '{}', but got '{}'", expected, got)
            }

            Self::StructureInitializationRequiresStructureType(inferred) => {
                write!(
                    f,
                    "A structure initialization expression requires a structure type, but got inferred type '{:?}'",
                    inferred
                )
            }

            Self::StructureInitializationMissingFields { expected, got } => {
                write!(
                    f,
                    "Structure initialization had {got} field(s), but structure declaration has {expected} field(s)"
                )
            }

            Self::MemberAccessNotSupported => {
                write!(f, "Member access expressions are only supported on structure value types at the moment")
            }

            Self::TypeDoesNotHaveMember { r#type, name } => {
                write!(f, "Type '{type}' does not have a member named '{name}'")
            }

            Self::UnsupportedFunctionCallee => write!(f, "Unable to resolve function callee"),

            Self::VariableDeclarationMissingInitialValue => {
                write!(f, "A variable declaration for a non-optional type must have an initial value")
            }
        }
    }
}

impl Error for TypecheckerError {
    fn span(&self) -> Option<Span> {
        Some(self.span)
    }
}

impl Display for TypecheckerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}
