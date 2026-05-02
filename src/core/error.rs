use std::fmt::Display;

use crate::{
    core::span::Span,
    module_registry::ModuleRegistry,
};

/// An error trait for all error implementations to derive.
///
/// It is responsible for ensuring that an error implementation derives [`Display`], and provides
/// an (optional) accessor for the [`Span`] that the error occurred at.
pub trait Error: Display {
    /// The [`Span`] that this error occurred at. If span information is not available, then
    /// [`None`] should be returned.
    fn span(&self) -> Option<Span>;

    /// Prints this error to the standard output, including information about the source file and
    /// line that caused it.
    fn print_to_stderr(&self, module_registry: &ModuleRegistry) {
        let span = match self.span() {
            Some(span) => span,
            _ => {
                error!("{}", self);
                return;
            }
        };

        let module = module_registry.get_module(span.module_id);

        // We need to find the line within the source file that caused this error. If we cannot
        // find the line, then we can just print a generic error message without any line
        // information.
        if let Some(source_information) = span.location.get_source_information(&module.file_contents) {
            let line_number_str = format!("{}", source_information.line_index + 1);
            let left_padding = " ".repeat(line_number_str.len());

            error!("{}", self);

            eprintln!(
                "{} ---> {}:{}:{}",
                left_padding,
                module.file_path.to_string_lossy(),
                source_information.line_index + 1,
                source_information.column_index + 1,
            );

            eprintln!("{} | ", left_padding);
            eprintln!("{} | {}", line_number_str, source_information.line);
            eprintln!(
                "{} | {}{}",
                left_padding,
                " ".repeat(source_information.column_index),
                "^".repeat(span.location.length)
            );
        } else {
            error!("{}", self);
            eprintln!("---> {}", module.file_path.to_string_lossy(),);
        }
    }
}

impl<E: Error + 'static> From<E> for Box<dyn Error> {
    fn from(value: E) -> Self {
        Box::new(value)
    }
}
