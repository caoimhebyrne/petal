#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceSpan {
    /// The start offset of the span in the source code.
    pub start: usize,

    /// The end offset of the span in the source code.
    pub end: usize,
}

impl SourceSpan {
    /// Returns a [SourceSpan] containing the start and end offset between the two provided spans.
    pub fn between(start: &SourceSpan, end: &SourceSpan) -> Self {
        SourceSpan {
            start: start.start,
            end: end.end,
        }
    }

    /// Returns the length of this source span in characters.
    pub fn length(&self) -> usize {
        self.end - self.start
    }

    /// Returns the line and column within the provided source string that corresponds to the start of this source span.
    pub fn get_line_and_column(&self, string: &str) -> (usize, usize) {
        // If the start of this span is larger than the string's length, then we can just return 0,0.
        if self.start > string.len() {
            return (0, 0);
        }

        let mut line = 1;
        let mut column = 1;

        // We can then iterate over each character, incrementing line and column accordingly until we reach the
        // start index.
        for (index, character) in string.char_indices() {
            // If the index is equal to the start offset, then we have calculated what we need to calculate.
            if index == self.start {
                break;
            }

            // Otherwise, we can attempt to increment line and or column depending on what the character is.
            if character == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        (line, column)
    }
}
