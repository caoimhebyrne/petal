use crate::{
    driver::aarch64::Aarch64Driver,
    ir::{BinaryOperation, Function, IntegerLiteral, Operand, VariableReference},
};

pub trait ValueVisitor {
    fn visit(&self, driver: &Aarch64Driver, function: &Function, code: &mut String) -> String;
}

impl ValueVisitor for IntegerLiteral {
    fn visit(&self, _driver: &Aarch64Driver, _function: &Function, _code: &mut String) -> String {
        return self.value.to_string();
    }
}

impl ValueVisitor for VariableReference {
    fn visit(&self, _driver: &Aarch64Driver, function: &Function, _code: &mut String) -> String {
        let variable = function.variables.get(self.variable_index).unwrap();
        return format!("[sp, {}]", variable.stack_index);
    }
}

impl ValueVisitor for BinaryOperation {
    fn visit(&self, driver: &Aarch64Driver, function: &Function, code: &mut String) -> String {
        let left = driver.compile_value(&self.left, function, code);
        let right = driver.compile_value(&self.right, function, code);

        let operation = match self.operand {
            Operand::Add => "add",
            Operand::Divide => "udiv",
            Operand::Multiply => "mul",
            Operand::Subtract => "sub",
        };

        // These operations typically do not work with immediate values.
        // r9 will be used for the left, and r10 will be used for the right.
        code.push_str(&format!("    mov x9, {}\n", left));
        code.push_str(&format!("    mov x10, {}\n", right));
        code.push_str(&format!("    {} x0, {}, {}\n", operation, "x9", "x10"));
        return format!("x0");
    }
}
