use std::path::PathBuf;

use crate::{Driver, error::DriverResult};
use petal_ir::function::Function;

pub struct Aarch64MacOSDriver {
    /// The lines of assembly to output at the end of visiting the function's statements.
    pub assembly: Vec<String>,
}

impl Driver for Aarch64MacOSDriver {
    fn new() -> Self {
        Aarch64MacOSDriver { assembly: Vec::new() }
    }

    fn generate(&mut self, _functions: Vec<Function>, _output_path: &PathBuf) -> DriverResult<()> {
        todo!()
    }
}
