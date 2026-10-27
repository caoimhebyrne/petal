use petal_diagnostic::{Diagnostic, DiagnosticSeverity};

pub fn print_diagnostics(source_path: &str, diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        let prefix = if diagnostic.severity == DiagnosticSeverity::Error {
            "error"
        } else {
            "warn"
        };

        println!(
            "{:>5}({}:{}..{}): {}",
            prefix,
            source_path,
            diagnostic.span.start(),
            diagnostic.span.end(),
            diagnostic.message
        );
    }
}
