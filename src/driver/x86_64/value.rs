use crate::{
    driver::x86_64::X86_64Driver,
    ir::{BinaryOperation, Function, IntegerLiteral, Operand, VariableReference},
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
        return format!("dword ptr [rbp-{}]", variable.stack_index);
    }
}

impl ValueVisitor for BinaryOperation {
    fn visit(&self, driver: &X86_64Driver, function: &Function, code: &mut String) -> String {
        let left_value = driver.compile_value(&self.left, function, code);
        let right_value = driver.compile_value(&self.right, function, code);

        // We can store the left value into `eax`, that is the result register.
        code.push_str(&format!("    mov eax, {}\n", left_value));

        // We can then add the right value to eax.
        let instruction = match self.operand {
            Operand::Add => "add",
            Operand::Subtract => "sub",
            Operand::Multiply => "imul",
            Operand::Divide => todo!("i am NOT implementing this"),
        };

        code.push_str(&format!("    {} eax, {}\n", instruction, right_value));

        return "eax".to_string();
    }
}
