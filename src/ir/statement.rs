use crate::{
    ast::node::statement::{Return, VariableDeclaration},
    ir::{Operation, context::Context, generator::IntermediateRepresentation},
};

/// Visits a statement in the AST, converting it to one or more IR operations.
pub(crate) trait StatementVisitor {
    fn visit(&self, context: &mut Context, operations: &mut Vec<Operation>);
}

impl StatementVisitor for VariableDeclaration {
    fn visit(&self, context: &mut Context, operations: &mut Vec<Operation>) {
        // If the variable is being initialized with a value, we need to store that value
        // into the space on the stack.
        let value = IntermediateRepresentation::visit_expression(context, &self.value);

        // A variable declaration just needs us to allocate a space on the stack.
        let index = context.function_scope().declare_variable(&self.name, value.size());

        operations.push(Operation::Store {
            variable_index: index,
            value,
        });
    }
}

impl StatementVisitor for Return {
    fn visit(&self, context: &mut Context, operations: &mut Vec<Operation>) {
        // If the return statement has a value, we can generate an IR value for it.
        let value = self
            .value
            .as_ref()
            .map(|it| IntermediateRepresentation::visit_expression(context, &it));

        operations.push(Operation::Return { value });
    }
}
