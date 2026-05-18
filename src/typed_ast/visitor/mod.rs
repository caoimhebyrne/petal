use std::collections::BTreeMap;

use crate::{
    ast::expression::binary_operation::BinaryOperator,
    typed_ast::{
        Expression,
        ExpressionKind,
        Function,
        FunctionKey,
        Statement,
        StatementKind,
        r#type::db::TypeId,
    },
};

// todo: replace all generic types in function calls and structures with a new type id by visiting each node

pub(crate) mod print;

/// Visits a [`Program`] in the typed AST.
///
/// Implementers of this trait are encouraged to only override the methods that they need. If you are overriding a
/// method, you are encouraged to call the `walk_x` free functions provided by this module to continue visiting.
///
/// Implementers of this trait must also implement [`Sized`]. This is to allow the trait to be used as a generic type
/// parameter.
pub trait ProgramVisitor: Sized {
    /// Visits the provided [`Function`].
    #[allow(unused_variables)] // not used by this implementation, but may be by others
    fn visit_function(&mut self, key: &FunctionKey, function: &mut Function) {
        walk_function(self, function);
    }

    /// Visits the provided [`Statement`].
    fn visit_statement(&mut self, statement: &mut Statement) {
        walk_statement(self, statement);
    }

    /// Visits a reference value assignment statement.
    fn visit_statement_reference_value_assignment(&mut self, target: &mut Expression, value: &mut Expression) {
        walk_statement_reference_value_assignment(self, target, value);
    }

    /// Visits a return statement.
    fn visit_statement_return(&mut self, value: Option<&mut Expression>) {
        walk_statement_return(self, value);
    }

    /// Visits a variable assignment statement.
    fn visit_statement_variable_assignment(
        &mut self,
        name: &str,
        value: &mut Expression,
        variable_type_id: &mut TypeId,
    ) {
        walk_statement_variable_assignment(self, name, value, variable_type_id);
    }

    /// Visits a variable declaration statement.
    fn visit_statement_variable_declaration(&mut self, name: &str, value: &mut Expression, type_id: &mut TypeId) {
        walk_statement_variable_declaration(self, name, value, type_id);
    }

    /// Visits the provided [`Expression`].
    fn visit_expression(&mut self, expression: &mut Expression) {
        walk_expression(self, expression);
    }

    /// Visits a binary operation expression.
    fn visit_expression_binary_operation(
        &mut self,
        left: &mut Expression,
        right: &mut Expression,
        operator: &mut BinaryOperator,
        type_id: &mut TypeId,
    ) {
        walk_expression_binary_operation(self, left, right, operator, type_id);
    }

    /// Visits a dereference expression.
    fn visit_expression_dereference(&mut self, reference: &mut Expression) {
        walk_expression_dereference(self, reference);
    }

    /// Visits a function call expression.
    fn visit_expression_function_call(
        &mut self,
        function_key: &FunctionKey,
        arguments: &mut [Expression],
        type_id: &mut TypeId,
    ) {
        walk_expression_function_call(self, function_key, arguments, type_id);
    }

    /// Visits a number literal expression.
    #[allow(unused_variables)] // not used by this implementation, but may be by others
    fn visit_expression_number_literal(&mut self, value: &mut f64, type_id: &mut TypeId) {}

    /// Visits a reference expression.
    fn visit_expression_reference(&mut self, value: &mut Expression) {
        walk_expression_reference(self, value);
    }

    /// Visits a structure field reference.
    fn visit_expression_structure_field_reference(&mut self, target: &mut Expression, field_index: &mut usize) {
        walk_expression_structure_field_reference(self, target, field_index);
    }

    /// Visits a structure initialization expression.
    fn visit_expression_structure_initialization(&mut self, field_values: &mut Vec<Expression>) {
        walk_expression_structure_initialization(self, field_values);
    }

    /// Visits a variable reference expression.
    #[allow(unused_variables)] // not used by this implementation, but may be by others
    fn visit_expression_variable_reference(&mut self, variable_name: &mut str, type_id: &mut TypeId) {}

    /// Visits a [`TypeId`].
    #[allow(unused_variables)] // not used by this implementation, but may be by others
    fn visit_type_id(&mut self, type_id: &mut TypeId) {}
}

/// Invokes the `visitor` on any child nodes within a [`Program`].
///
/// The individual required fields of the [`Program`] are passed individually. This allows the caller to take
/// ownership of a reference to any fields that this method is not concerned with (e.g. [`TypeDb`]).
pub fn walk_program<V: ProgramVisitor>(visitor: &mut V, functions: &mut BTreeMap<FunctionKey, Function>) {
    for (key, function) in functions {
        visitor.visit_function(key, function);
    }
}

/// Invokes the `visitor` on any child nodes within a [`Function`].
pub fn walk_function<V: ProgramVisitor>(visitor: &mut V, function: &mut Function) {
    visitor.visit_type_id(&mut function.return_type_id);

    for parameter in &mut function.parameters {
        visitor.visit_type_id(&mut parameter.type_id);
    }

    for statement in &mut function.body {
        visitor.visit_statement(statement);
    }
}

/// Invokes the `visitor`'s specialized methods on the provided [`Statement`].
fn walk_statement<V: ProgramVisitor>(visitor: &mut V, statement: &mut Statement) {
    match &mut statement.kind {
        StatementKind::FunctionCall { function_key, arguments, return_type_id } => {
            visitor.visit_expression_function_call(function_key, arguments, return_type_id);
        }

        StatementKind::ReferenceValueAssignment { target, value } => {
            visitor.visit_statement_reference_value_assignment(target, value);
        }

        StatementKind::Return(value) => {
            visitor.visit_statement_return(value.as_mut());
        }

        StatementKind::VariableAssignment { name, value, variable_type_id } => {
            visitor.visit_statement_variable_assignment(name, value, variable_type_id);
        }

        StatementKind::VariableDeclaration { name, value, type_id } => {
            visitor.visit_statement_variable_declaration(name, value, type_id);
        }
    }
}

pub fn walk_statement_reference_value_assignment<V: ProgramVisitor>(
    visitor: &mut V,
    target: &mut Expression,
    value: &mut Expression,
) {
    visitor.visit_expression(target);
    visitor.visit_expression(value);
}

/// Invokes the `visitor` on any child nodes within a return statement.
pub fn walk_statement_return<V: ProgramVisitor>(visitor: &mut V, value: Option<&mut Expression>) {
    if let Some(expression) = value {
        visitor.visit_expression(expression);
    }
}

/// Invokes the `visitor` on any child nodes within a variable assignment statement.
pub fn walk_statement_variable_assignment<V: ProgramVisitor>(
    visitor: &mut V,
    _name: &str,
    value: &mut Expression,
    variable_type_id: &mut TypeId,
) {
    visitor.visit_type_id(variable_type_id);
    visitor.visit_expression(value);
}

/// Invokes the `visitor` on any child nodes within a variable declaration statement.
pub fn walk_statement_variable_declaration<V: ProgramVisitor>(
    visitor: &mut V,
    _name: &str,
    value: &mut Expression,
    type_id: &mut TypeId,
) {
    visitor.visit_type_id(type_id);
    visitor.visit_expression(value);
}

/// Invokes the `visitor`'s specialized methods on the proivded [`Expression`].
fn walk_expression<V: ProgramVisitor>(visitor: &mut V, expression: &mut Expression) {
    visitor.visit_type_id(&mut expression.type_id);

    match &mut expression.kind {
        ExpressionKind::BinaryOperation { left, right, operator } => {
            visitor.visit_expression_binary_operation(left, right, operator, &mut expression.type_id);
        }

        ExpressionKind::Dereference(reference) => {
            visitor.visit_expression_dereference(reference);
        }

        ExpressionKind::FunctionCall { function_key, arguments } => {
            visitor.visit_expression_function_call(function_key, arguments, &mut expression.type_id);
        }

        ExpressionKind::NumberLiteral(value) => {
            visitor.visit_expression_number_literal(value, &mut expression.type_id);
        }

        ExpressionKind::Reference(value) => {
            visitor.visit_expression_reference(value);
        }

        ExpressionKind::StructureFieldReference { target, field_index } => {
            visitor.visit_expression_structure_field_reference(target, field_index);
        }

        ExpressionKind::StructureInitialization { field_values } => {
            visitor.visit_expression_structure_initialization(field_values);
        }

        ExpressionKind::VariableReference(variable_name) => {
            visitor.visit_expression_variable_reference(variable_name, &mut expression.type_id);
        }
    }
}

/// Invokes the `visitor` on any child nodes within a binary operation expression.
pub fn walk_expression_binary_operation<V: ProgramVisitor>(
    visitor: &mut V,
    left: &mut Expression,
    right: &mut Expression,
    _operator: &mut BinaryOperator,
    type_id: &mut TypeId,
) {
    visitor.visit_type_id(type_id);
    visitor.visit_expression(left);
    visitor.visit_expression(right);
}

/// Invokes the `visitor` on any child nodes within a dereference expression.
pub fn walk_expression_dereference<V: ProgramVisitor>(visitor: &mut V, reference: &mut Expression) {
    visitor.visit_expression(reference);
}

/// Invokes the `visitor` on any child nodes within a function call expression.
pub fn walk_expression_function_call<V: ProgramVisitor>(
    visitor: &mut V,
    _function_key: &FunctionKey,
    arguments: &mut [Expression],
    type_id: &mut TypeId,
) {
    visitor.visit_type_id(type_id);

    for argument in arguments {
        visitor.visit_expression(argument);
    }
}

/// Invokes the `visitor` on any child nodes within a reference expression.
pub fn walk_expression_reference<V: ProgramVisitor>(visitor: &mut V, value: &mut Expression) {
    visitor.visit_expression(value);
}

/// Invokes the `visitor` on any child nodes within a structure field reference expression.
pub fn walk_expression_structure_field_reference<V: ProgramVisitor>(
    visitor: &mut V,
    expression: &mut Expression,
    _field_index: &mut usize,
) {
    visitor.visit_expression(expression);
}

/// Invokes the `visitor` on any child nodes within a structure initialization expression.
pub fn walk_expression_structure_initialization<V: ProgramVisitor>(
    visitor: &mut V,
    field_values: &mut Vec<Expression>,
) {
    for field_value in field_values {
        visitor.visit_expression(field_value);
    }
}
