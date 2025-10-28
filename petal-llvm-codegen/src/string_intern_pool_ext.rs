use petal_core::{
    error::Result,
    source_span::SourceSpan,
    string_intern::{StringInternPool, StringReference},
};

use crate::error::LLVMCodegenErrorKind;

/// An "extension trait" which adds result variants of [StringInternPool] functions.
pub(crate) trait StringInternPoolExt {
    /// Resolves a [StringReference], returning its string value if it exists.
    fn resolve_reference_or_err(&self, reference: &StringReference, span: SourceSpan) -> Result<&str>;
}

impl<'ctx> StringInternPoolExt for &'ctx dyn StringInternPool {
    fn resolve_reference_or_err(&self, reference: &StringReference, span: SourceSpan) -> Result<&str> {
        self.resolve_reference(&reference)
            .ok_or(LLVMCodegenErrorKind::unresolved_string_reference(&reference, span))
    }
}
