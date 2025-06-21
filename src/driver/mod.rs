use crate::{driver::error::DriverError, ir::Function};
use std::path::PathBuf;

pub mod aarch64;
pub mod error;
pub mod x86_64;

pub type DriverResult<T> = Result<T, DriverError>;

// Responsible for generating assembly from the IR.
pub trait Driver {
    fn new(output_path: PathBuf) -> Self
    where
        Self: Sized;

    fn compile(&self, ir: Vec<Function>) -> DriverResult<()>;
}
