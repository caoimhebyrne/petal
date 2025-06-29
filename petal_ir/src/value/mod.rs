use crate::value::{
    binary_operation::{BinaryOperation, Operand},
    function_call::FunctionCall,
    integer_literal::IntegerLiteral,
    local_reference::LocalReference,
};

pub mod binary_operation;
pub mod function_call;
pub mod integer_literal;
pub mod local_reference;

/// Represents a value in the intermediate representation.
///
/// This includes information about the value, like the expected type for it, where it occurred
/// at within the source code, and the actual value itself (see [kind]).
///
/// See [ValueKind], [ValueType].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value {
    /// The kind of value that this represents.
    pub kind: ValueKind,

    /// Additional type information about this value.
    pub r#type: ValueType,
}

/// The different kinds of values in the intermediate representation.
///
/// A "value" is anything from a constant which was defined at compile time, to a binary operation
/// between two values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueKind {
    /// An integer literal defined in the source code.
    IntegerLiteral(IntegerLiteral),

    /// A reference to a local variable or parameter.
    LocalReference(LocalReference),

    /// A binary operation between two values.
    BinaryOperation(BinaryOperation),

    /// A function call.
    FunctionCall(FunctionCall),
}

/// Represents the "type" of a value in the intermediate representation.
///
/// Most types will be similar to their [ValueKind], but they are more generic and may
/// provide additional information about the value. For example: [ValueKind::IntegerLiteral] does not
/// declare what width the integer is. That is located within [ValueType::Integer].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValueType {
    Integer { width: u8 },
    Reference,
}

impl Value {
    pub fn new(kind: ValueKind, r#type: ValueType) -> Value {
        Value { kind, r#type }
    }

    pub fn new_integer_literal(literal: u64, r#type: ValueType) -> Value {
        Value {
            kind: ValueKind::IntegerLiteral(IntegerLiteral { literal }),
            r#type,
        }
    }

    pub fn new_local_reference(index: usize, value_type: ValueType) -> Value {
        Value {
            kind: ValueKind::LocalReference(LocalReference { index }),
            r#type: value_type,
        }
    }

    pub fn new_binary_operation(lhs: Value, rhs: Value, operand: Operand, r#type: ValueType) -> Value {
        Value {
            kind: ValueKind::BinaryOperation(BinaryOperation {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                operand,
            }),
            r#type,
        }
    }

    pub fn new_function_call(name: String, arguments: Vec<Value>, r#type: ValueType) -> Value {
        Value {
            kind: ValueKind::FunctionCall(FunctionCall { name, arguments }),
            r#type,
        }
    }
}
