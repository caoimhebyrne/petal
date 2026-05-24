#[derive(Default)]
pub struct Writer {
    /// The indentation level that the writer is currently at.
    indentation: usize,

    /// The string being written to.
    pub inner: String,
}

impl Writer {
    /// Appends a string literal to this [`Writer`].
    pub fn append(&mut self, str: &str) {
        self.inner += str;
    }

    /// Appends white-space representing the current indentation level to this [`Writer`].
    pub fn append_indentation_string(&mut self) {
        self.append(&self.get_indentation_string());
    }

    /// Appends a string literal representing a single line to this [`Writer`].
    pub fn append_line(&mut self, str: &str) {
        self.append_indentation_string();
        self.append(str);
        self.append("\n");
    }

    /// Increases this [`Writer`]'s indentation by one.
    pub fn increase_indentation(&mut self) {
        self.indentation = self.indentation.saturating_add(1);
    }

    /// Decreases this [`Writer`]'s indentation by one.
    pub fn decrease_indentation(&mut self) {
        self.indentation = self.indentation.saturating_sub(1);
    }

    /// Returns a [`String`] representing the indentation level of this builder.
    fn get_indentation_string(&self) -> String {
        " ".repeat(self.indentation * 4)
    }
}
