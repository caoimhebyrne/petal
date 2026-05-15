/// A type on a node within the typed AST.
#[derive(Debug, Copy, Clone)]
pub enum Ty {
    /// A generic type which needs to be substituted for its concrete type when available.
    Generic(usize),

    /// A signed integer of a certain size (i{x}).
    SignedInteger(u8),

    /// An unsigned integer of a certain size (u{x}).
    UnsignedInteger(u8),

    /// Unit, typically the result of a function call.
    Void,
}
