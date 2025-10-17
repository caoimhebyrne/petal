#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceSpan {
    /// The start offset of the span in the source code.
    pub start: usize,

    /// The end offset of the span in the source code.
    pub end: usize,
}

impl SourceSpan {
    /// Returns a [SourceSpan] containing the start and end offset between the two provided spans.
    pub fn between(start: &SourceSpan, end: &SourceSpan) -> SourceSpan {
        SourceSpan {
            start: start.start,
            end: end.end,
        }
    }
}
