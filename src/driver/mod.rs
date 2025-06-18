use crate::ir::Function;

pub mod aarch64;
pub mod x86_64;

// Responsible for generating assembly from the IR.
pub trait Driver {
    fn compile(&self, ir: Vec<Function>) -> String;
}
