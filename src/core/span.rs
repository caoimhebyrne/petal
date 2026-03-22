/// The location of some text within an original source file.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Span {
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
    pub fn new(start: usize, length: usize) -> Self {
        Span { start, length }
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

    use crate::core::span::{Span, SpanSourceInformation};

    #[test]
    fn get_source_information_without_new_lines() {
        let source = "ab c";
        let span = Span { start: 3, length: 1 };

        assert_eq!(
            span.get_source_information(source),
            Some(SpanSourceInformation { line_index: 0, column_index: 3, line: "ab c".into() })
        )
    }

    #[test]
    fn get_source_information_with_new_lines() {
        let source = "\n\nidentifier 123\n";
        let span = Span { start: 12, length: 3 };

        assert_eq!(
            span.get_source_information(source),
            Some(SpanSourceInformation { line_index: 2, column_index: 10, line: "identifier 123".into() })
        )
    }
}
