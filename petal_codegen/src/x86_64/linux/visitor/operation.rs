use crate::{X86_64LinuxDriver, error::DriverResult, visitor::OperationVisitor};
use petal_ir::{
    function::{Function, Local, LocalKind},
    operation::{r#return::Return, store_local::StoreLocal},
    value::{ValueType, function_call::FunctionCall},
};

impl OperationVisitor for StoreLocal {
    type Driver = X86_64LinuxDriver;

    fn visit(&self, function: &mut Function, driver: &mut Self::Driver) -> DriverResult<()> {
        // The position of the variable on the stack depends on the size of the items before it.
        let stack_position = function
            .locals
            .iter()
            .take(self.index)
            .map(|it| X86_64LinuxDriver::size_of(it.value_type))
            .sum::<usize>();

        // Now that we know the position, we can store the value.
        let value = driver.visit_value(function, &self.value)?;

        // TODO: actual sizing
        let instruction = match self.value.r#type {
            ValueType::Integer { .. } => format!("mov qword ptr [rsp+{}], {}", stack_position, value),
        };

        driver.assembly.push(instruction);
        Ok(())
    }
}

impl OperationVisitor for Return {
    type Driver = X86_64LinuxDriver;

    fn visit(&self, function: &mut Function, driver: &mut Self::Driver) -> DriverResult<()> {
        if let Some(value) = &self.value {
            let the_value = driver.visit_value(function, &value)?;

            driver.assembly.push(match value.r#type {
                ValueType::Integer { .. } => format!("mov rax, {}", the_value),
            });
        }

        Ok(())
    }
}

impl OperationVisitor for FunctionCall {
    type Driver = X86_64LinuxDriver;

    fn visit(&self, function: &mut Function, driver: &mut Self::Driver) -> DriverResult<()> {
        for (idx, argument) in self.arguments.iter().enumerate() {
            let register = X86_64LinuxDriver::local_parameter_register(idx, argument.r#type, false);

            if idx >= 6 {
                function.locals.push(Local {
                    kind: LocalKind::Variable,
                    name: "__petal_internal_tmp".into(),
                    value_type: argument.r#type,
                });
            }

            let value = driver.visit_value(function, &argument)?;
            driver.assembly.push(format!("mov {}, {}", register, value));
        }

        driver.assembly.push(format!("call {}", self.name));
        Ok(())
    }
}
