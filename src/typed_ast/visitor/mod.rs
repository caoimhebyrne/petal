use crate::{
    ast::expression::binary_operation::BinaryOperator,
    typed_ast::{
        Expression,
        ExpressionKind,
        Function,
        FunctionKey,
        Statement,
        StatementKind,
        r#type::TypeId,
    },
};

pub(crate) mod generic_type_visitor;
pub(crate) mod print;

/// Visits a [`Program`] in the typed AST.
pub trait ProgramVisitor {
    /// Visits the provided [`Function`].
    fn visit_function(&mut self, key: &FunctionKey, function: &mut Function) {
        let _ = key;
        for statement in &mut function.body {
            self.visit_statement(statement);
        }
    }

    /// Visits the provided [`Statement`].
    fn visit_statement(&mut self, statement: &mut Statement) {
        match &mut statement.kind {
            StatementKind::Return(value) => {
                self.visit_statement_return(value.as_mut());
            }

            StatementKind::VariableDeclaration { name, value, type_id } => {
                self.visit_statement_variable_declaration(name, value, type_id);
            }
        }
    }

    /// Visits a variable declaration statement.
    fn visit_statement_variable_declaration(&mut self, name: &str, value: &mut Expression, type_id: &mut TypeId);

    /// Visits a return statement.
    fn visit_statement_return(&mut self, value: Option<&mut Expression>);

    /// Visits the provided [`Expression`].
    fn visit_expression(&mut self, expression: &mut Expression) {
        self.visit_expression_kind(&mut expression.kind, &mut expression.type_id);
    }

    /// Visits the provided [`ExpressionKind`].
    fn visit_expression_kind(&mut self, kind: &mut ExpressionKind, type_id: &mut TypeId) {
        match kind {
            ExpressionKind::BinaryOperation { left, right, operator } => {
                self.visit_expression_binary_operation(left, right, operator, type_id);
            }

            ExpressionKind::FunctionCall { function_key, arguments } => {
                self.visit_expression_function_call(function_key, arguments, type_id);
            }

            ExpressionKind::NumberLiteral(value) => {
                self.visit_expression_number_literal(value, type_id);
            }

            ExpressionKind::VariableReference(variable_name) => {
                self.visit_variable_reference(variable_name, type_id);
            }
        }
    }

    /// Visits a binary operation expression.
    fn visit_expression_binary_operation(
        &mut self,
        left: &mut Expression,
        right: &mut Expression,
        operator: &mut BinaryOperator,
        type_id: &mut TypeId,
    );

    /// Visits a function call expression.
    fn visit_expression_function_call(
        &mut self,
        function_key: &FunctionKey,
        arguments: &mut [Expression],
        type_id: &mut TypeId,
    );

    /// Visits a number literal expression.
    fn visit_expression_number_literal(&mut self, value: &mut f64, type_id: &mut TypeId);

    /// Visits a variable reference expression.
    fn visit_variable_reference(&mut self, variable_name: &mut str, type_id: &mut TypeId);
}
