use crate::{
    ast::node::statement::{Return, VariableDeclaration, VariableReassignment},
    ir::{
        Operation,
        context::Context,
        generator::{IRResult, IntermediateRepresentation},
    },
};

/// Visits a statement in the AST, converting it to one or more IR operations.
pub(crate) trait StatementVisitor {
    fn visit(&self, context: &mut Context, operations: &mut Vec<Operation>) -> IRResult<()>;
}

impl StatementVisitor for VariableDeclaration {
    fn visit(&self, context: &mut Context, operations: &mut Vec<Operation>) -> IRResult<()> {
        // If the variable is being initialized with a value, we need to store that value
        // into the space on the stack.
        let value = IntermediateRepresentation::visit_expression(context, &self.value)?;

        // A variable declaration just needs us to allocate a space on the stack.
        let index = context
            .function_scope(Some(self.node.location))?
            .declare_variable(&self.name, value.size());

        operations.push(Operation::Store {
            variable_index: index,
            value,
        });

        Ok(())
    }
}

impl StatementVisitor for VariableReassignment {
    fn visit(&self, context: &mut Context, operations: &mut Vec<Operation>) -> IRResult<()> {
        let value = IntermediateRepresentation::visit_expression(context, &self.value)?;
        let index = context
            .function_scope(Some(self.node.location))?
            .find_variable_index(&self.name);

        operations.push(Operation::Store {
            variable_index: index,
            value,
        });

        Ok(())
    }
}

impl StatementVisitor for Return {
    fn visit(&self, context: &mut Context, operations: &mut Vec<Operation>) -> IRResult<()> {
        // If the return statement has a value, we can generate an IR value for it.
        let value = if let Some(value) = &self.value {
            Some(IntermediateRepresentation::visit_expression(context, &value)?)
        } else {
            None
        };

        operations.push(Operation::Return { value });
        Ok(())
    }
}
