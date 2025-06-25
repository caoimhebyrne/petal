use crate::Driver;

pub struct Aarch64MacOSDriver {
    /// The lines of assembly to output at the end of visiting the function's statements.
    pub assembly: Vec<String>,
}

impl Driver for Aarch64MacOSDriver {
    fn new() -> Self {
        Aarch64MacOSDriver { assembly: Vec::new() }
    }

    fn generate(&mut self, functions: Vec<petal_ir::function::Function>) -> petal_ir::error::IRResult<()> {
        todo!()
    }
}
