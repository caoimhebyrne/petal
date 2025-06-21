use crate::{
    driver::aarch64::Aarch64Driver,
    ir::{Function, IntegerLiteral, VariableReference},
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
