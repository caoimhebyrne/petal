use crate::{
    ast::expression::binary_operation::BinaryOperator,
    typed_ast::{
        Expression,
        Function,
        FunctionKey,
        Program,
        r#type::{
            Type,
            TypeDb,
            TypeId,
        },
        visitor::{
            ProgramVisitor,
            walk_expression,
            walk_expression_binary_operation,
            walk_expression_function_call,
            walk_function,
            walk_program,
            walk_statement_return,
            walk_statement_variable_assignment,
            walk_statement_variable_declaration,
        },
    },
};

/// Prints the state of a [`Program`] to the standard output.
pub struct PrintingProgramVisitor<'db> {
    /// The current indentation level of this visitor.
    indentation_level: usize,

    /// A reference to the [`TypeDb`].
    type_db: &'db TypeDb,
}

impl<'db> PrintingProgramVisitor<'db> {
    /// Creates a new [`PrintingProgramVisitor`] with the provided [`TypeDb`] reference.
    pub fn new(type_db: &'db TypeDb) -> Self {
        Self { indentation_level: 0, type_db }
    }

    /// A convenience method for calling [`Self::new`] and [`Self::visit`].
    pub fn visit(program: &'db mut Program) {
        let mut visitor = Self::new(&program.type_db);
        walk_program(&mut visitor, &mut program.functions);
    }

    /// Returns a string containing the amount of padding required before this statement.
    fn indentation_string(&self) -> String {
        " ".repeat(self.indentation_level * 4)
    }

    /// Increases the indentation level by one.
    fn increase_indentation(&mut self) {
        self.indentation_level += 1;
    }

    /// Decreases the indentation level by one.
    fn decrease_indentation(&mut self) {
        self.indentation_level -= 1;
    }

    /// Returns a human-readable string for the type referenced by the provided [`TypeId`].
    fn visit_type_id(&self, type_id: TypeId) -> String {
        let ty = *self.type_db.get_type(type_id);

        match ty {
            Type::Defined(defined_type_id) => {
                let defined_type = self.type_db.get_defined_type(defined_type_id);
                defined_type.name.clone()
            }
            Type::SignedInteger(bits) => format!("i{bits}"),
            Type::Reference(inner_type_id) => format!("&{}", self.visit_type_id(inner_type_id)),
            Type::UnsignedInteger(bits) => format!("u{bits}"),
            Type::Generic(_) => "?".to_string(),
            Type::Void => "void".to_string(),
        }
    }
}

impl ProgramVisitor for PrintingProgramVisitor<'_> {
    fn visit_function(&mut self, key: &FunctionKey, function: &mut Function) {
        debug!(
            "Function '{}' -> returns {} ({:?}):",
            function.name,
            self.visit_type_id(function.return_type_id),
            function.return_type_id
        );

        self.increase_indentation();

        debug!("{}{key:?}", self.indentation_string());
        debug!("");

        if !function.parameters.is_empty() {
            debug!("{}Parameters:", self.indentation_string());

            self.increase_indentation();

            for parameter in &function.parameters {
                debug!(
                    "{}{} (type = {}, {:?})",
                    self.indentation_string(),
                    parameter.name,
                    self.visit_type_id(parameter.type_id),
                    parameter.type_id
                );
            }

            self.decrease_indentation();

            debug!("");
        }

        debug!("{}Body:", self.indentation_string());

        self.increase_indentation();

        walk_function(self, function);

        self.decrease_indentation();

        self.decrease_indentation();

        debug!("");
    }

    fn visit_statement_return(&mut self, value: Option<&mut Expression>) {
        debug!("{}Return", self.indentation_string());
        walk_statement_return(self, value);
    }

    fn visit_statement_variable_assignment(&mut self, name: &str, value: &mut Expression, type_id: &mut TypeId) {
        debug!(
            "{}Assign variable '{}' (type = {}, {:?})",
            self.indentation_string(),
            name,
            self.visit_type_id(*type_id),
            type_id
        );

        walk_statement_variable_assignment(self, name, value, type_id);
    }

    fn visit_statement_variable_declaration(&mut self, name: &str, value: &mut Expression, type_id: &mut TypeId) {
        debug!(
            "{}Declare variable '{}' (type = {}, {:?})",
            self.indentation_string(),
            name,
            self.visit_type_id(*type_id),
            type_id
        );

        walk_statement_variable_declaration(self, name, value, type_id);
    }

    fn visit_expression(&mut self, expression: &mut Expression) {
        self.increase_indentation();
        walk_expression(self, expression);
        self.decrease_indentation();
    }

    fn visit_expression_binary_operation(
        &mut self,
        left: &mut Expression,
        right: &mut Expression,
        operator: &mut BinaryOperator,
        type_id: &mut TypeId,
    ) {
        debug!("{}{} (type = {}, {:?})", self.indentation_string(), operator, self.visit_type_id(*type_id), type_id);
        walk_expression_binary_operation(self, left, right, operator, type_id);
    }

    fn visit_expression_function_call(
        &mut self,
        function_key: &FunctionKey,
        arguments: &mut [Expression],
        type_id: &mut TypeId,
    ) {
        debug!(
            "{}Function call (key = {:?} type = {}, {:?})",
            self.indentation_string(),
            function_key,
            self.visit_type_id(*type_id),
            type_id
        );

        walk_expression_function_call(self, function_key, arguments, type_id);
    }

    fn visit_expression_number_literal(&mut self, value: &mut f64, type_id: &mut TypeId) {
        debug!(
            "{}Number literal {} (type = {}, {:?})",
            self.indentation_string(),
            value,
            self.visit_type_id(*type_id),
            type_id
        );
    }

    fn visit_variable_reference(&mut self, variable_name: &mut str, type_id: &mut TypeId) {
        debug!(
            "{}Variable reference '{}' (type = {}, {:?})",
            self.indentation_string(),
            variable_name,
            self.visit_type_id(*type_id),
            type_id
        );
    }
}
