/// An integer literal defined in the source code.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct IntegerLiteral {
    /// A very wide data type is used intentionally here, this is to prevent needing enum variants for all
    /// supported widths. The type of the [crate::value::Value] should help with sizing during compilation
    /// or execution.
    pub literal: u64,
}
