use std::fmt::Display;

pub mod error;
pub mod generator;

mod context;
mod expression;
mod statement;

/// A value which can be an argument of an [Operation] in the IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    IntegerLiteral(IntegerLiteral),

    /// A reference to a variable in the current scope, the associated value being the variable's index.
    VariableReference(VariableReference),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegerLiteral {
    pub value: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableReference {
    pub variable_index: usize,
}

impl Value {
    pub fn size(&self) -> usize {
        match self {
            Value::IntegerLiteral(_) => 4,
            Value::VariableReference(_) => 4,
        }
    }
}

/// An operation in the IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    /// Stores a value into a variable allocated on the stack.
    Store(Store),

    /// Returns a variable from the current function.
    Return(Return),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Store {
    pub variable_index: usize,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Return {
    pub value: Option<Value>,
}

/// A function defined in the IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    /// The body of the function (a.k.a, the list of operations).
    pub body: Vec<Operation>,

    /// The name of the function
    pub name: String,

    /// The variables declared in this function.
    pub variables: Vec<Variable>,
}

/// Represents a variable defined within a function scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    /// The name of the variable.
    pub name: String,

    /// The size of this variable's value.
    pub expected_value_size: usize,

    /// The offset of this variable on the stack.
    pub stack_index: usize,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::IntegerLiteral(literal) => write!(f, "{}", literal.value),
            Value::VariableReference(reference) => write!(f, "@{}", reference.variable_index),
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Store(store) => write!(f, "@{} = {}", store.variable_index, store.value),

            Operation::Return(r#return) => {
                if let Some(value) = &r#return.value {
                    write!(f, "return {}", value)
                } else {
                    write!(f, "return")
                }
            }
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "function {}:\n", self.name)?;

        for operation in &self.body {
            write!(f, "  {}\n", operation)?;
        }

        Ok(())
    }
}
