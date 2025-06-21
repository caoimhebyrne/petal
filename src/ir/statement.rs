use crate::{
    ast::{
        self,
        node::statement::{VariableDeclaration, VariableReassignment},
    },
    ir::{
        Operation, Return, Store,
        context::Context,
        generator::{IRResult, IntermediateRepresentation},
    },
};

/// Visits a statement in the AST, converting it to one or more IR operations.
pub(crate) trait StatementVisitor {
    fn visit(&self, context: &mut Context) -> IRResult<Operation>;
}

impl StatementVisitor for VariableDeclaration {
    fn visit(&self, context: &mut Context) -> IRResult<Operation> {
        // If the variable is being initialized with a value, we need to store that value
        // into the space on the stack.
        let value = IntermediateRepresentation::visit_expression(context, &self.value)?;

        // A variable declaration just needs us to allocate a space on the stack.
        let index = context
            .function_scope(self.node)?
            .declare_variable(&self.name, value.size(), self.node)?;

        Ok(Operation::Store(Store {
            variable_index: index,
            value,
        }))
    }
}

impl StatementVisitor for VariableReassignment {
    fn visit(&self, context: &mut Context) -> IRResult<Operation> {
        let value = IntermediateRepresentation::visit_expression(context, &self.value)?;

        let index = context.function_scope(self.node)?.find_variable_index(&self.name);

        Ok(Operation::Store(Store {
            variable_index: index,
            value,
        }))
    }
}

impl StatementVisitor for ast::node::statement::Return {
    fn visit(&self, context: &mut Context) -> IRResult<Operation> {
        // If the return statement has a value, we can generate an IR value for it.
        let value = if let Some(value) = &self.value {
            Some(IntermediateRepresentation::visit_expression(context, &value)?)
        } else {
            None
        };

        Ok(Operation::Return(Return { value }))
    }
}
