use crate::driver::aarch64::Aarch64Driver;
use petal_core::ir::{Function, Return, Store};

pub trait OperationVisitor {
    fn visit(&self, driver: &Aarch64Driver, function: &Function, code: &mut String);
}

impl OperationVisitor for Store {
    fn visit(&self, driver: &Aarch64Driver, function: &Function, code: &mut String) {
        let variable = function.variables.get(self.variable_index).unwrap();
        let value = driver.compile_value(&self.value, function, code);

        code.push_str(&format!("    mov w8, {}\n", value));
        code.push_str(&format!("    str w8, [sp, {}]\n", variable.stack_index));
    }
}

impl OperationVisitor for Return {
    fn visit(&self, driver: &Aarch64Driver, function: &Function, code: &mut String) {
        if let Some(value) = &self.value {
            let value = driver.compile_value(value, function, code);
            code.push_str(&format!("    mov x0, {}\n", value));
        }
    }
}
