use petal_core::{
    string_intern::{StringInternPool, StringInternPoolImpl},
    r#type::pool::TypePool,
};

/// The state of the petal compiler.
pub struct CompilerState {
    /// The [StringInternPool] to use.
    pub string_intern_pool: Box<dyn StringInternPool>,

    /// The [TypePool] to use.
    pub type_pool: TypePool,
}

impl CompilerState {
    /// Creates a new [CompilerState].
    pub fn new() -> Self {
        CompilerState {
            string_intern_pool: Box::new(StringInternPoolImpl::new()),
            type_pool: TypePool::new(),
        }
    }
}
