use crate::{X86_64LinuxDriver, visitor::ValueVisitor};
use petal_ir::{
    error::IRResult,
    function::Function,
    value::{integer_literal::IntegerLiteral, local_reference::LocalReference},
};

impl ValueVisitor for IntegerLiteral {
    type Driver = X86_64LinuxDriver;

    fn visit(&self, _function: &Function, _driver: &mut Self::Driver) -> IRResult<String> {
        Ok(self.literal.to_string())
    }
}

impl ValueVisitor for LocalReference {
    type Driver = X86_64LinuxDriver;

    fn visit(&self, function: &Function, _driver: &mut Self::Driver) -> IRResult<String> {
        if self.is_parameter {
            todo!()
        }

        // The position of the variable on the stack depends on the size of the items before it.
        let stack_position = function
            .locals
            .iter()
            .take(self.index)
            .map(|it| X86_64LinuxDriver::size_of(it.value_type))
            .sum::<usize>();

        Ok(format!("[rsp+{}]", stack_position))
    }
}
