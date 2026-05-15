use crate::{
    ast::expression::binary_operation::BinaryOperator,
    typed_ast::{
        Expression,
        Function,
        FunctionKey,
        GenericInformation,
        r#type::Ty,
        visitor::ProgramVisitor,
    },
};

/// Responsible for resolving any [`Ty::Generic`] references within specialized functions.
#[derive(Default)]
pub struct GenericTypeVisitor {
    /// The generic information which applies to the current "scope".
    generic_information: Option<GenericInformation>,
}

impl GenericTypeVisitor {
    /// Attempts to get the [`GenericInformation`] that this visitor is visiting, returning an error if it could not
    /// be found.
    fn get_generic_type_information(&self) -> &GenericInformation {
        self.generic_information.as_ref().expect("self.generic_information should be `Some(_)`")
    }

    /// Sets the [`GenericInformation`] that this visitor is currently visiting.
    fn set_generic_information(&mut self, generic_information: GenericInformation) {
        self.generic_information = Some(generic_information);
    }

    /// Removes the current [`GenericInformation`] from this visitor.
    fn unset_generic_information(&mut self) {
        self.generic_information = None;
    }

    /// Visits [`Ty`], resolving it to its concrete type if it is [`Ty::Generic`].
    fn visit_ty(&self, ty: &mut Ty) {
        let Ty::Generic(generic_type_index) = ty else {
            return;
        };

        let generic_information = self.get_generic_type_information();

        let (_, actual_type) = generic_information
            .types
            .iter()
            .nth(*generic_type_index)
            .expect("generic_type_index should be in bounds of generic_information.types");

        *ty = *actual_type;
    }
}

impl ProgramVisitor for GenericTypeVisitor {
    fn visit_function(&mut self, key: &FunctionKey, function: &mut Function) {
        let Some(generic_information) = function.generic_information.clone() else {
            trace!("Function named '{}' ({:?}) has no generic information, ignoring", function.name, key);
            return;
        };

        self.set_generic_information(generic_information);

        self.visit_ty(&mut function.return_ty);

        for parameter in &mut function.parameters {
            self.visit_ty(&mut parameter.ty);
        }

        for statement in &mut function.body {
            self.visit_statement(statement);
        }

        self.unset_generic_information();
    }

    fn visit_statement_variable_declaration(&mut self, _name: &str, value: &mut Expression, ty: &mut Ty) {
        self.visit_expression(value);
        self.visit_ty(ty);
    }

    fn visit_statement_return(&mut self, value: Option<&mut Expression>) {
        if let Some(value) = value {
            self.visit_expression(value);
        }
    }

    fn visit_expression_binary_operation(
        &mut self,
        left: &mut Expression,
        right: &mut Expression,
        _operator: &mut BinaryOperator,
        ty: &mut Ty,
    ) {
        self.visit_expression(left);
        self.visit_expression(right);
        self.visit_ty(ty);
    }

    fn visit_expression_function_call(
        &mut self,
        _function_key: &FunctionKey,
        arguments: &mut [Expression],
        ty: &mut Ty,
    ) {
        for argument in arguments {
            self.visit_expression(argument);
        }

        self.visit_ty(ty);
    }

    fn visit_expression_number_literal(&mut self, _value: &mut f64, ty: &mut Ty) {
        self.visit_ty(ty);
    }

    fn visit_variable_reference(&mut self, _variable_name: &mut str, ty: &mut Ty) {
        self.visit_ty(ty);
    }
}
