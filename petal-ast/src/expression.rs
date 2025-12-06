///! An expression node is similar to a statement node. The only difference between them being the fact that an
///! expression yields a certain value.
use petal_core::{source_span::SourceSpan, string_intern::StringReference, r#type::TypeReference};

use crate::{
    expression::{binary_operation::BinaryOperation, reference::Reference},
    node::FunctionCall,
};

pub mod binary_operation;
pub mod reference;

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
    IntegerLiteral { value: u64 },

    /// A string literal.
    StringLiteral { value: StringReference },

    /// An identifier reference.
    IdentifierReference { identifier: StringReference },

    /// A binary operation between two expression nodes.
    BinaryOperation(BinaryOperation),

    /// A function call.
    FunctionCall(FunctionCall),

    /// A reference to another value.
    Reference(Reference),
}
