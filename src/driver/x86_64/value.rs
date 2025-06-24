use crate::{
    driver::x86_64::X86_64Driver,
    ir::{BinaryOperation, Function, FunctionCall, IntegerLiteral, Operand, VariableReference},
};

pub trait ValueVisitor {
    fn visit(&self, driver: &X86_64Driver, function: &Function, code: &mut String) -> String;
}

impl ValueVisitor for IntegerLiteral {
    fn visit(&self, _driver: &X86_64Driver, _function: &Function, _code: &mut String) -> String {
        return self.value.to_string();
    }
}

impl ValueVisitor for VariableReference {
    fn visit(&self, _driver: &X86_64Driver, function: &Function, _code: &mut String) -> String {
        let variable = function.variables.get(self.variable_index).unwrap();
        if variable.is_parameter {
            match self.variable_index {
                0 => return "rdi".to_string(),
                1 => return "rsi".to_string(),
                2 => return "rdx".to_string(),
                3 => return "rcx".to_string(),
                4 => return "r8".to_string(),
                5 => return "r9".to_string(),
                _ => {}
            };
        }

        return format!("dword ptr [rbp-{}]", variable.stack_index);
    }
}

impl ValueVisitor for BinaryOperation {
    fn visit(&self, driver: &X86_64Driver, function: &Function, code: &mut String) -> String {
        let left_value = driver.compile_value(&self.left, function, code);
        let right_value = driver.compile_value(&self.right, function, code);

        // We can store the left value into `rax`, that is the result register.
        code.push_str(&format!("    mov rax, {}\n", left_value));

        // We can then add the right value to rax.
        let instruction = match self.operand {
            Operand::Add => "add",
            Operand::Subtract => "sub",
            Operand::Multiply => "imul",
            Operand::Divide => todo!("i am NOT implementing this"),
        };

        code.push_str(&format!("    {} rax, {}\n", instruction, right_value));

        return "rax".to_string();
    }
}

impl ValueVisitor for FunctionCall {
    fn visit(&self, driver: &X86_64Driver, function: &Function, code: &mut String) -> String {
        for (idx, argument) in self.arguments.iter().enumerate() {
            let value = driver.compile_value(argument, function, code);

            let register = match idx {
                0 => "rdi",
                1 => "rsi",
                2 => "rdx",
                3 => "rcx",
                4 => "r8",
                5 => "r9",
                _ => todo!("arguments on the stack"),
            };

            code.push_str(&format!("    mov {}, {}\n", register, value));
        }

        code.push_str(&format!("    call {}\n", self.name));
        "rax".to_string()
    }
}
