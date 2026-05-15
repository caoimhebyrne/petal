use crate::{
    ast::expression::binary_operation::BinaryOperator,
    typed_ast::{
        Expression,
        Function,
        FunctionKey,
        r#type::Ty,
        visitor::ProgramVisitor,
    },
};

/// Prints the state of a [`Program`] to the standard output.
#[derive(Default)]
pub struct PrintingProgramVisitor {
    /// The current indentation level of this visitor.
    indentation_level: usize,
}

impl PrintingProgramVisitor {
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

    /// Returns a human-readable string for the provided [`Ty`].
    fn visit_ty(ty: &Ty) -> String {
        match ty {
            Ty::SignedInteger(bits) => format!("i{bits}"),
            Ty::UnsignedInteger(bits) => format!("u{bits}"),
            Ty::Generic(_) => "?".to_string(),
            Ty::Void => "void".to_string(),
        }
    }
}

impl ProgramVisitor for PrintingProgramVisitor {
    fn visit_function(&mut self, key: &FunctionKey, function: &mut Function) {
        debug!("Function '{}' (key = {:?}):", function.name, key);

        self.increase_indentation();

        if !function.parameters.is_empty() {
            debug!("{}Parameters:", self.indentation_string());

            self.increase_indentation();

            for parameter in &function.parameters {
                debug!(
                    "{}{} (ty = {})",
                    self.indentation_string(),
                    parameter.name,
                    PrintingProgramVisitor::visit_ty(&parameter.ty)
                );
            }

            self.decrease_indentation();

            debug!("");
        }

        debug!("{}Body:", self.indentation_string());

        self.increase_indentation();

        for statement in &mut function.body {
            self.visit_statement(statement);
        }

        self.decrease_indentation();

        self.decrease_indentation();

        debug!("");
    }

    fn visit_statement_variable_declaration(&mut self, name: &str, value: &mut Expression, ty: &mut Ty) {
        debug!("{}Variable '{}' (ty = {})", self.indentation_string(), name, PrintingProgramVisitor::visit_ty(ty));
        self.visit_expression(value);
    }

    fn visit_statement_return(&mut self, value: Option<&mut Expression>) {
        debug!("{}Return", self.indentation_string());

        if let Some(value) = value {
            self.visit_expression(value);
        }
    }

    fn visit_expression(&mut self, expression: &mut Expression) {
        self.increase_indentation();
        self.visit_expression_kind(&mut expression.kind, &mut expression.ty);
        self.decrease_indentation();
    }

    fn visit_expression_binary_operation(
        &mut self,
        left: &mut Expression,
        right: &mut Expression,
        operator: &mut BinaryOperator,
        ty: &mut Ty,
    ) {
        debug!("{}{} (ty = {})", self.indentation_string(), operator, PrintingProgramVisitor::visit_ty(ty));

        self.visit_expression(left);
        self.visit_expression(right);
    }

    fn visit_expression_function_call(
        &mut self,
        function_key: &FunctionKey,
        arguments: &mut [Expression],
        ty: &mut Ty,
    ) {
        debug!(
            "{}Function call (key = {:?}, ty = {})",
            self.indentation_string(),
            function_key,
            PrintingProgramVisitor::visit_ty(ty)
        );

        for argument in arguments.iter_mut() {
            self.visit_expression(argument);
        }
    }

    fn visit_expression_number_literal(&mut self, value: &mut f64, ty: &mut Ty) {
        debug!("{}Number literal {} (ty = {})", self.indentation_string(), value, PrintingProgramVisitor::visit_ty(ty));
    }

    fn visit_variable_reference(&mut self, variable_name: &mut str, ty: &mut Ty) {
        debug!(
            "{}Variable reference '{}' (ty = {})",
            self.indentation_string(),
            variable_name,
            PrintingProgramVisitor::visit_ty(ty)
        );
    }
}
