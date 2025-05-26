#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Operation {
    Add,
    Subtract,
    Divide,
    Multiply,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Comparison {
    LessThan,
    GreaterThan,
}
