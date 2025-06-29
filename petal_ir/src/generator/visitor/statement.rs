use crate::{
    error::{IRError, IRResult},
    function::{Local, LocalKind},
    generator::IRGenerator,
    operation::Operation,
    value::Value,
};
use petal_core::ast::node::{
    self,
    statement::{VariableDeclaration, VariableReassignment},
};

/// A visitor for an AST statement.
/// This converts a [Statment] into an IR [Operation].
pub trait StatementVisitor {
    fn visit(&self, generator: &mut IRGenerator) -> IRResult<Operation>;
}

impl StatementVisitor for VariableDeclaration {
    fn visit(&self, generator: &mut IRGenerator) -> IRResult<Operation> {
        // If the value cannot be represented in the IR, there's no point in continuing with the declaration.
        let initialization_value = generator.visit_expression(&self.value)?;
        let function_scope = generator.function_scope(self.node.location)?;

        // If a variable has been declared, we can insert it into this function scope's local variables.
        let local_index = function_scope.locals.len();

        function_scope.locals.push(Local {
            name: self.name.clone(),
            value_type: self.declared_type.clone().into(),
            kind: LocalKind::Variable,
        });

        // Then, we just need to store the initialization value into the local.
        Ok(Operation::new_store_local(local_index, initialization_value))
    }
}

impl StatementVisitor for VariableReassignment {
    fn visit(&self, generator: &mut IRGenerator) -> IRResult<Operation> {
        // If the value cannot be represented in the IR, there's no point in continuing with the declaration.
        let initialization_value = generator.visit_expression(&self.value)?;
        let function_scope = generator.function_scope(self.node.location)?;

        // The variable should have already been declared.
        let local_index = function_scope
            .locals
            .iter()
            .position(|it| it.name == self.name)
            .ok_or(IRError::undefined_identifier(self.node.location))?;

        // Then, we just need to store the initialization value into the local.
        Ok(Operation::new_store_local(local_index, initialization_value))
    }
}

impl StatementVisitor for node::statement::Return {
    fn visit(&self, generator: &mut IRGenerator) -> IRResult<Operation> {
        let value = self
            .value
            .clone()
            .map(|it| generator.visit_expression(&it))
            .transpose()?;

        Ok(Operation::new_return(value))
    }
}

impl StatementVisitor for node::expression::FunctionCall {
    fn visit(&self, generator: &mut IRGenerator) -> IRResult<Operation> {
        let arguments = self
            .arguments
            .iter()
            .map(|it| generator.visit_expression(it))
            .collect::<IRResult<Vec<Value>>>()?;

        Ok(Operation::new_function_call(self.name.clone(), arguments))
    }
}
