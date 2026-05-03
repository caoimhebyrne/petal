use crate::{
    ast::expression::{
        Expression,
        ExpressionKind,
    },
    core::span::Span,
};

#[derive(Debug, Clone, PartialEq)]
pub struct StructureInitialization {
    /// The fields being initialized.
    pub fields: Vec<StructureInitializationField>,
}

impl StructureInitialization {
    pub fn builder() -> StructureInitializationBuilder {
        StructureInitializationBuilder::default()
    }
}

impl From<StructureInitialization> for ExpressionKind {
    fn from(value: StructureInitialization) -> Self {
        Self::StructureInitialization(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructureInitializationField {
    /// The name of the field.
    pub name: String,

    /// The value being assigned to the field.
    pub value: Box<Expression>,

    /// The span within the source code that the field initialization is at.
    pub span: Span,
}

impl StructureInitializationField {
    /// Creates a new [`StructureInitializationField`].
    pub fn new(name: String, value: Expression, span: Span) -> Self {
        Self { name, value: value.into(), span }
    }
}

/// Builds a [`StructureInitialization`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StructureInitializationBuilder {
    /// The fields being initialized.
    fields: Vec<StructureInitializationField>,
}

impl StructureInitializationBuilder {
    /// Adds a new field to this [`StructureInitializationBuilder`].
    pub fn field(mut self, name: String, value: Expression, span: Span) -> Self {
        self.fields.push(StructureInitializationField::new(name, value, span));
        self
    }

    /// Builds this [`StructureInitializationBuilder`] into a [`StructureInitialization`].
    pub fn build(self) -> StructureInitialization {
        StructureInitialization { fields: self.fields }
    }
}
