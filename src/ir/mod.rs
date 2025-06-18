use std::fmt::Display;

pub mod generator;

mod context;
mod expression;
mod statement;

/// A value which can be an argument of an [Operation] in the IR.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Value {
    IntegerLiteral(u32),

    /// A reference to a variable in the current scope, the associated value being the variable's index.
    VariableReference(usize),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    /// Stores a value into a variable allocated on the stack.
    Store { variable_index: usize, value: Value },

    /// Returns a variable from the current function.
    Return { value: Option<Value> },
}

/// A function defined in the IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    /// The body of the function (a.k.a, the list of operations).
    pub body: Vec<Operation>,

    /// The name of the function
    pub name: String,

    // The size of the stack in bytes.
    pub stack_size: usize,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::IntegerLiteral(value) => write!(f, "{}", value),
            Value::VariableReference(variable_index) => write!(f, "@{}", variable_index),
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Store { variable_index, value } => write!(f, "store @{}, {}", variable_index, value),

            Operation::Return { value } => {
                if let Some(return_value) = value {
                    write!(f, "return {}", return_value)
                } else {
                    write!(f, "return")
                }
            }
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "function {} (stack size = {} bytes):\n", self.name, self.stack_size)?;

        for operation in &self.body {
            write!(f, "  {}\n", operation)?;
        }

        Ok(())
    }
}
