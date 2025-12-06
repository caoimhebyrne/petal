///! An expression node is similar to a statement node. The only difference between them being the fact that an
///! expression yields a certain value.
use petal_core::{source_span::SourceSpan, r#type::TypeReference};

use crate::{
    expression::{
        binary_operation::BinaryOperation, identifier_reference::IdentifierReference, integer_literal::IntegerLiteral,
        reference::Reference, string_literal::StringLiteral,
    },
    node::FunctionCall,
};

pub mod binary_operation;
pub mod identifier_reference;
pub mod integer_literal;
pub mod reference;
pub mod string_literal;

#[derive(Debug, PartialEq, Clone)]
pub struct ExpressionNode {
    /// The kind of [ExpressionNode] that this is.
    pub kind: ExpressionNodeKind,

    /// The expected type for this [ExpressionNode].
    pub r#type: Option<TypeReference>,

    /// The span within the source file that this node occurred at.
    pub span: SourceSpan,
}

impl ExpressionNode {
    /// Instantiates a new [ExpressionNode].
    pub fn new(kind: ExpressionNodeKind, span: SourceSpan) -> Self {
        ExpressionNode {
            kind,
            r#type: None,
            span,
        }
    }

    /// Instantiates a new [ExpressionNode].
    pub fn from(kind: impl Into<ExpressionNodeKind>, span: SourceSpan) -> Self {
        ExpressionNode::new(kind.into(), span)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionNodeKind {
    /// An integer literal.
    IntegerLiteral(IntegerLiteral),

    /// A string literal.
    StringLiteral(StringLiteral),

    /// An identifier reference.
    IdentifierReference(IdentifierReference),

    /// A binary operation between two expression nodes.
    BinaryOperation(BinaryOperation),

    /// A function call.
    FunctionCall(FunctionCall),

    /// A reference to another value.
    Reference(Reference),
}
