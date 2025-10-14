#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceSpan {
    /// The start offset of the span in the source code.
    pub start: usize,

    /// The end offset of the span in the source code.
    pub end: usize,
}
