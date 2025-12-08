use petal_core::{error::Result, source_span::SourceSpan, string_intern::StringInternPool};

use crate::{context::scope::ScopeContext, error::LLVMCodegenError};

pub mod scope;

/// The context for an LLVM code generator.
pub struct CodegenContext<'ctx> {
    /// A reference to the string intern pool to read string values from.
    string_intern_pool: &'ctx dyn StringInternPool,

    /// The scope context that is currently bound. If one is not present, a scope has not been started yet.
    scope_context: Option<ScopeContext<'ctx>>,
}

impl<'ctx> CodegenContext<'ctx> {
    /// Creates a new [CodegenContext].
    pub fn new(string_intern_pool: &'ctx dyn StringInternPool) -> Self {
        CodegenContext {
            string_intern_pool,
            scope_context: None,
        }
    }

    /// Starts a new scope within this context.
    pub fn start_scope_context(&mut self) {
        self.scope_context = Some(ScopeContext::new(self.string_intern_pool));
    }

    /// Returns a reference to the current [ScopeContext].
    ///
    /// This function will return an error if a [ScopeContext] is not yet bound.
    pub fn scope_context(&mut self, span: SourceSpan) -> Result<&mut ScopeContext<'ctx>> {
        self.scope_context
            .as_mut()
            .ok_or(LLVMCodegenError::missing_scope_context(span))
    }

    /// Destroys the scope within this context.
    pub fn end_scope_context(&mut self) {
        self.scope_context = None;
    }
}
