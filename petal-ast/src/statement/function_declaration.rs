use petal_core::{source_span::SourceSpan, string_intern::StringReference, r#type::TypeReference};

use crate::statement::{StatementNode, TopLevelStatementNodeKind};

/// A modifier of a function declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionModifier {
    /// This function is defined externally.
    External,

    /// This function is public.
    Public,
}

/// A parameter within a function declaration.
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionParameter {
    /// The name of the parameter.
    pub name: StringReference,

    /// The expected type of the parameter.
    pub r#type: TypeReference,

    /// The span within the source code that the parameter was defined at.
    pub span: SourceSpan,
}

impl FunctionParameter {
    /// Instantiates a [FunctionParameter].
    pub fn new(name: StringReference, r#type: TypeReference, span: SourceSpan) -> Self {
        FunctionParameter { name, r#type, span }
    }
}

/// A function declaration.
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDeclaration {
    /// The name of this function declaration.
    pub name: StringReference,

    /// The modifiers applied to this function declaration.
    pub modifiers: Vec<FunctionModifier>,

    /// The parameters within this function declaration.
    pub parameters: Vec<FunctionParameter>,

    /// The return type of this function.
    pub return_type: TypeReference,

    /// The body of this function.
    pub body: Vec<StatementNode>,
}

impl FunctionDeclaration {
    /// Instantiates a [FunctionDeclaration].
    pub fn new(
        name: StringReference,
        modifiers: Vec<FunctionModifier>,
        parameters: Vec<FunctionParameter>,
        return_type: TypeReference,
        body: Vec<StatementNode>,
    ) -> Self {
        FunctionDeclaration {
            name,
            modifiers,
            parameters,
            return_type,
            body,
        }
    }

    /// Returns whether this function declaration is an external one.
    pub fn is_external(&self) -> bool {
        self.modifiers.contains(&FunctionModifier::External)
    }

    /// REturns whether this function declaration is a public one.
    pub fn is_public(&self) -> bool {
        self.modifiers.contains(&FunctionModifier::Public)
    }
}

impl From<FunctionDeclaration> for TopLevelStatementNodeKind {
    fn from(val: FunctionDeclaration) -> Self {
        TopLevelStatementNodeKind::FunctionDeclaration(val)
    }
}
