use crate::error::DriverResult;
use petal_ir::function::Function;

pub trait OperationVisitor {
    type Driver: crate::Driver;

    fn visit(&self, function: &Function, driver: &mut Self::Driver) -> DriverResult<()>;
}

pub trait ValueVisitor {
    type Driver: crate::Driver;

    fn visit(&self, function: &Function, driver: &mut Self::Driver) -> DriverResult<String>;
}
