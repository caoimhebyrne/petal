const TAB_WIDTH: usize = 4;

/// Generates a string of C code by keeping track of indentation, and allowing users to append to it.
#[derive(Default)]
pub(crate) struct Writer {
    /// The string being built.
    pub code: String,

    /// The current level of indentation.
    indentation_level: usize,
}

impl Writer {
    /// Writes a new line of code to the [`Writer`].
    pub fn append(&mut self, string: impl AsRef<str>) {
        for _ in 0..self.indentation_level * TAB_WIDTH {
            self.code.push(' ');
        }

        self.code.push_str(string.as_ref());
        self.code.push('\n');
    }

    /// Increases the level of indentation by 1.
    pub fn increase_indent(&mut self) {
        self.indentation_level += 1;
    }

    /// Decreases the level of indentation by 1.
    pub fn decrease_indent(&mut self) {
        self.indentation_level -= 1;
    }
}
