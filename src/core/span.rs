use crate::module_registry::ModuleId;

/// The location of some text within an original source file.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Span {
    /// The module that the span occurred in.
    pub module_id: ModuleId,

    /// The location that the span occurred at.
    pub location: SpanLocation,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SpanLocation {
    /// The index of the character in the source file that this span starts at.
    pub start: usize,

    /// The number of characters that this span consists of.
    pub length: usize,
}

/// The line information returned by [`Span::get_line_from_string`].
#[derive(Debug, PartialEq)]
pub struct SpanSourceInformation {
    /// The line itself.
    pub line: String,

    /// The index of the line in the original source.
    pub line_index: usize,

    /// The index of the column in the original source.
    pub column_index: usize,
}

impl Span {
    /// Creates a new [`Span`].
    pub fn new(module_id: ModuleId, start: usize, length: usize) -> Self {
        Span { module_id, location: SpanLocation { start, length } }
    }

    /// Creates a new [`Span`] from the start and end of the provided spans.
    pub fn between(start: Span, end: Span) -> Self {
        Span { module_id: start.module_id, location: SpanLocation::between(start.location, end.location) }
    }
}

impl SpanLocation {
    /// Creates a new [`SpanLocation`] from the start and end of the provided spans.
    pub fn between(start: SpanLocation, end: SpanLocation) -> Self {
        Self { start: start.start, length: (end.start + end.length) - start.start }
    }

    /// Returns information about this span's location in the provided [`source`] string.
    pub fn get_source_information(&self, source: &str) -> Option<SpanSourceInformation> {
        let mut current_offset: usize = 0;

        for (line_index, line) in source.lines().enumerate() {
            let line_start_offset = current_offset;

            // We need to add an extra character to include the new-line. The `Span` includes new
            // lines in its indices.
            current_offset += line.len() + 1;

            // The first line that contains the starting offset will be returned.
            if current_offset >= self.start {
                return Some(SpanSourceInformation {
                    line: line.into(),
                    column_index: self.start - line_start_offset,
                    line_index,
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::core::span::{
        SpanLocation,
        SpanSourceInformation,
    };

    #[test]
    fn get_source_information_without_new_lines() {
        let source = "ab c";
        let location = SpanLocation { start: 3, length: 1 };

        assert_eq!(
            location.get_source_information(source),
            Some(SpanSourceInformation { line_index: 0, column_index: 3, line: "ab c".into() })
        )
    }

    #[test]
    fn get_source_information_with_new_lines() {
        let source = "\n\nidentifier 123\n";
        let location = SpanLocation { start: 12, length: 3 };

        assert_eq!(
            location.get_source_information(source),
            Some(SpanSourceInformation { line_index: 2, column_index: 10, line: "identifier 123".into() })
        )
    }
}
