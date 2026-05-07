use crate::{
    ast::{
        statement::{
            Statement,
            StatementKind,
        },
        type_expr::TypeExpr,
    },
    core::span::Span,
    typechecker::r#type::Type,
};

/// Modifiers for a function declaration.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DeclarationModifier {
    /// This declaration is public and can be accessed by other modules.
    Public,

    /// This declaration's name should not be mangled, as it is provided by other code. It also will have an empty
    /// body.
    Extern,
}

/// A function declaration within the AST.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    /// The name of the type that owns the function.
    pub owner_type_name: Option<String>,

    /// The name of the function being declared.
    pub name: String,

    /// The body of the function.
    pub body: Vec<Statement>,

    /// The parameters of the function.
    pub parameters: Vec<FunctionParameter>,

    /// The declared return type of the function.
    pub return_type_expr: Option<TypeExpr>,

    /// The resolved return type.
    pub return_type: Type,

    /// The modifiers applied to this [`FunctionDeclaration`].
    pub modifiers: Vec<DeclarationModifier>,
}

impl FunctionDeclaration {
    /// Creates a new [`FunctionDeclaration`].
    pub fn new(
        owner_type_name: Option<String>,
        name: String,
        body: Vec<Statement>,
        parameters: Vec<FunctionParameter>,
        return_type_expr: Option<TypeExpr>,
        return_type: Type,
        modifiers: Vec<DeclarationModifier>,
    ) -> Self {
        FunctionDeclaration { owner_type_name, name, body, parameters, return_type_expr, return_type, modifiers }
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
    /// The name of the type that owns the function.
    owner_type_name: Option<String>,

    /// The name of the function.
    name: String,

    /// The body of the function.
    body: Vec<Statement>,

    /// The parameters of the function.
    parameters: Vec<FunctionParameter>,

    /// The declared return type of the function.
    return_type_expr: Option<TypeExpr>,

    /// The resolved return type of the function.
    return_type: Type,

    /// The modifiers of this function
    modifiers: Vec<DeclarationModifier>,
}

impl FunctionDeclarationBuilder {
    /// Creates a new [`FunctionDeclarationBuilder`].
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            owner_type_name: None,
            name: name.into(),
            body: vec![],
            parameters: vec![],
            return_type_expr: None,
            return_type: Type::Unknown,
            modifiers: Vec::new(),
        }
    }

    /// Sets the name of the owner type of the function.
    pub fn owner_type_name(mut self, value: String) -> Self {
        self.owner_type_name = Some(value);
        self
    }

    /// Adds a statement to the body of the function.
    pub fn statement(mut self, statement: Statement) -> Self {
        self.body.push(statement);
        self
    }

    /// Adds a parameter to the body of the function.
    pub fn parameter(
        mut self,
        name: impl Into<String>,
        type_expr: TypeExpr,
        r#type: Type,
        is_named: bool,
        span: Span,
    ) -> Self {
        self.parameters.push(FunctionParameter::new(name, type_expr, r#type, is_named, span));
        self
    }

    /// Sets the return type of the function.
    pub fn return_type(mut self, type_expr: TypeExpr, r#type: Type) -> Self {
        self.return_type_expr = Some(type_expr);
        self.return_type = r#type;
        self
    }

    /// Adds a modifier to this function.
    pub fn modifier(mut self, modifier: DeclarationModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    /// Builds this [`FunctionDeclarationBuilder`] into a [`FunctionDeclaration`].
    pub fn build(self) -> FunctionDeclaration {
        FunctionDeclaration::new(
            self.owner_type_name,
            self.name,
            self.body,
            self.parameters,
            self.return_type_expr,
            self.return_type,
            self.modifiers,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParameter {
    /// The name of the parameter.
    pub name: String,

    /// The declared type of the parameter.
    pub type_expr: TypeExpr,

    /// The resolved type of the parameter.
    pub r#type: Type,

    /// Whether the parameter is a named parameter.
    pub is_named: bool,

    /// The location within the source code that this parameter occurred at.
    pub span: Span,
}

impl FunctionParameter {
    /// Creates a new [`FunctionParameter`].
    pub fn new(name: impl Into<String>, type_expr: TypeExpr, r#type: Type, is_named: bool, span: Span) -> Self {
        Self { name: name.into(), type_expr, r#type, is_named, span }
    }
}
