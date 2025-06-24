use crate::{
    driver::x86_64::X86_64Driver,
    ir::{Function, Return, Store},
};

pub trait OperationVisitor {
    fn visit(&self, driver: &X86_64Driver, function: &Function, code: &mut String);
}

impl OperationVisitor for Store {
    fn visit(&self, driver: &X86_64Driver, function: &Function, code: &mut String) {
        let variable = function.variables.get(self.variable_index).unwrap();
        let value = driver.compile_value(&self.value, function, code);

        code.push_str(&format!(
            "    mov dword ptr [rbp-{}], {}\n",
            variable.stack_index, value
        ));
    }
}

impl OperationVisitor for Return {
    fn visit(&self, driver: &X86_64Driver, function: &Function, code: &mut String) {
        if let Some(value) = &self.value {
            let value = driver.compile_value(value, function, code);
            if value != "rax" {
                code.push_str(&format!("    mov eax, {}\n", value));
            }
        }
    }
}
