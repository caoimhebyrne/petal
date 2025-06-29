use crate::{
    X86_64LinuxDriver,
    error::{DriverError, DriverResult},
    visitor::ValueVisitor,
};
use petal_ir::{
    function::{Function, Local, LocalKind},
    value::{
        ValueType,
        binary_operation::{BinaryOperation, Operand},
        data_section_reference::DataSectionReference,
        function_call::FunctionCall,
        integer_literal::IntegerLiteral,
        local_reference::LocalReference,
    },
};

impl ValueVisitor for IntegerLiteral {
    type Driver = X86_64LinuxDriver;

    fn visit(&self, _function: &mut Function, _driver: &mut Self::Driver) -> DriverResult<String> {
        Ok(self.literal.to_string())
    }
}

impl ValueVisitor for LocalReference {
    type Driver = X86_64LinuxDriver;

    fn visit(&self, function: &mut Function, _driver: &mut Self::Driver) -> DriverResult<String> {
        let local = function
            .locals
            .get(self.index)
            .expect(&format!("Expected local at index {} but got None", self.index));

        if local.kind == LocalKind::Parameter {
            return Ok(X86_64LinuxDriver::local_parameter_register(
                self.index,
                local.value_type,
                true,
            ));
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

impl ValueVisitor for BinaryOperation {
    type Driver = X86_64LinuxDriver;

    fn visit(&self, function: &mut Function, driver: &mut Self::Driver) -> DriverResult<String> {
        let lhs = driver.visit_value(function, &self.lhs)?;
        let rhs = driver.visit_value(function, &self.rhs)?;

        // We can store the left value into `rax`, this is also going to be the result register
        // for this operation.
        driver.assembly.push(format!("mov rax, {}", lhs));

        // We can then perform the operation between `rax` and the rhs value.
        let instruction = match self.operand {
            Operand::Add => "add",
            Operand::Subtract => "sub",
            Operand::Multiply => "imul",
            Operand::Divide => return Err(DriverError::unsupported_operand(self.operand, None)),
        };

        driver.assembly.push(format!("{} rax, {}", instruction, rhs));
        driver.assembly.push(format!("mov r10, rax"));
        Ok("r10".to_string())
    }
}

impl ValueVisitor for FunctionCall {
    type Driver = X86_64LinuxDriver;

    fn visit(&self, function: &mut Function, driver: &mut Self::Driver) -> DriverResult<String> {
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
            if argument.r#type == ValueType::Reference {
                driver.assembly.push(format!("lea {}, {}", register, value));
            } else {
                driver.assembly.push(format!("mov {}, {}", register, value));
            }
        }

        driver.assembly.push(format!("call {}", self.name));
        Ok("rax".to_string())
    }
}
impl ValueVisitor for DataSectionReference {
    type Driver = X86_64LinuxDriver;

    fn visit(&self, function: &mut Function, _driver: &mut Self::Driver) -> DriverResult<String> {
        Ok(format!("[rip + {}_data_{}]", function.name, self.index))
    }
}
