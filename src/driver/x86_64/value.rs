use crate::{
    driver::x86_64::X86_64Driver,
    ir::{Function, IntegerLiteral, VariableReference},
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
