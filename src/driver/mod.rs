use std::path::PathBuf;

use crate::ir::Function;

pub mod aarch64;
pub mod x86_64;

// Responsible for generating assembly from the IR.
pub trait Driver {
    fn new(output_path: PathBuf) -> Self
    where
        Self: Sized;

    fn compile(&self, ir: Vec<Function>);
}
