use super::Typechecker;
use crate::ast::node::kind::{FunctionDefinitionNode, ReturnNode, VariableDeclarationNode};

pub trait StatmentTypecheck {
    fn resolve<'a>(&mut self);
}

impl StatmentTypecheck for VariableDeclarationNode {
    fn resolve<'a>(&mut self) {
        // Before checking the value's type, we must first know what the declared type of the
        // variable is.
        self.declared_type = Typechecker::resolve_type(self.declared_type.clone());

        println!("todo: ensure type of expression matches variable type declaration");
    }
}

impl StatmentTypecheck for FunctionDefinitionNode {
    fn resolve<'a>(&mut self) {
        // We must resolve the return type of the function first.
        self.return_type = self.return_type.clone().map(Typechecker::resolve_type);

        // Then, we can check the types within the body.
        Typechecker::check_block(&mut self.body);
    }
}

impl StatmentTypecheck for ReturnNode {
    fn resolve<'a>(&mut self) {
        println!("todo: type-check return node expression");
        println!("todo: ensure return expression matches expected block return type");
    }
}
