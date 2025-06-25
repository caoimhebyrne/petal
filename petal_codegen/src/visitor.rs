use petal_ir::{error::IRResult, function::Function};

pub trait OperationVisitor {
    type Driver: crate::Driver;

    fn visit(&self, function: &Function, driver: &mut Self::Driver) -> IRResult<()>;
}

pub trait ValueVisitor {
    type Driver: crate::Driver;

    fn visit(&self, function: &Function, driver: &mut Self::Driver) -> IRResult<String>;
}
