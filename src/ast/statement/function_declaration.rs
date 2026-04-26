use crate::{
    ast::{
        statement::{
            Statement,
            StatementKind,
        },
        r#type::Type,
    },
    core::span::Span,
};

/// A function declaration within the AST.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    /// The name of the function being declared.
    pub name: String,

    /// The body of the function.
    pub body: Vec<Statement>,

    /// The parameters of the function.
    pub parameters: Vec<FunctionParameter>,

    /// The return type of the function.
    pub return_type: Option<Type>,
}

impl FunctionDeclaration {
    /// Creates a new [`FunctionDeclaration`].
    pub fn new(
        name: impl Into<String>,
        body: Vec<Statement>,
        parameters: Vec<FunctionParameter>,
        return_type: Option<Type>,
    ) -> Self {
        FunctionDeclaration { name: name.into(), body, parameters, return_type }
    }

    /// Creates a new [`FunctionDeclarationBuilder`].
    pub fn builder(name: impl Into<String>) -> FunctionDeclarationBuilder {
        FunctionDeclarationBuilder::new(name)
    }
}

/// Converts a [`FunctionDeclaration`] to a [`StatementKind`].
impl From<FunctionDeclaration> for StatementKind {
    fn from(value: FunctionDeclaration) -> Self {
        Self::FunctionDeclaration(value)
    }
}

/// A builder for a [`FunctionDeclaration`].
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclarationBuilder {
    /// The name of the function.
    name: String,

    /// The body of the function.
    body: Vec<Statement>,

    /// The parameters of the function.
    parameters: Vec<FunctionParameter>,

    /// The return type of the function.
    return_type: Option<Type>,
}

impl FunctionDeclarationBuilder {
    /// Creates a new [`FunctionDeclarationBuilder`].
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), body: vec![], parameters: vec![], return_type: None }
    }

    /// Adds a statement to the body of the function.
    pub fn statement(mut self, statement: Statement) -> Self {
        self.body.push(statement);
        self
    }

    /// Adds a parameter to the body of the function.
    pub fn parameter(mut self, name: impl Into<String>, r#type: Type, span: Span) -> Self {
        self.parameters.push(FunctionParameter::new(name, r#type, span));
        self
    }

    /// Sets the return type of the function.
    pub fn return_type(mut self, r#type: Type) -> Self {
        self.return_type = Some(r#type);
        self
    }

    /// Builds this [`FunctionDeclarationBuilder`] into a [`FunctionDeclaration`].
    pub fn build(self) -> FunctionDeclaration {
        FunctionDeclaration::new(self.name, self.body, self.parameters, self.return_type)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParameter {
    /// The name of the parameter.
    pub name: String,

    /// The type of the parameter.
    pub r#type: Type,

    /// The location within the source code that this parameter occurred at.
    pub span: Span,
}

impl FunctionParameter {
    /// Creates a new [`FunctionParameter`].
    pub fn new(name: impl Into<String>, r#type: Type, span: Span) -> Self {
        Self { name: name.into(), r#type, span }
    }
}
