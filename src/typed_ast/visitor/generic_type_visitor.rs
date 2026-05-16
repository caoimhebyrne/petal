use crate::{
    ast::expression::binary_operation::BinaryOperator,
    typed_ast::{
        Expression,
        Function,
        FunctionKey,
        GenericInformation,
        Program,
        r#type::{
            Type,
            TypeDb,
            TypeId,
        },
        visitor::ProgramVisitor,
    },
};

/// Responsible for resolving any [`Ty::Generic`] references within specialized functions.
pub struct GenericTypeVisitor<'db> {
    /// The generic information which applies to the current "scope".
    generic_information: Option<GenericInformation>,

    /// The [`TypeDb`].
    type_db: &'db mut TypeDb,
}

impl<'db> GenericTypeVisitor<'db> {
    /// Creates a new [`GenericTypeVisitor`] which will resolve any generic types in specialized functions and types
    /// against the provided [`TypeDb`].
    pub fn new(type_db: &'db mut TypeDb) -> Self {
        Self { generic_information: None, type_db }
    }

    /// A convenience method for calling [`Self::new`] and [`Self::visit`].
    pub fn visit(program: &'db mut Program) {
        let mut visitor = Self::new(&mut program.type_db);

        for (function_key, function) in &mut program.functions {
            visitor.visit_function(function_key, function);
        }
    }

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

    /// Visits the type behind the provided [`TypeId`], resolving it to its concrete type if it is [`Type::Generic`].
    fn visit_type_id(&mut self, type_id: TypeId) {
        let Type::Generic(generic_type_index) = *self.type_db.get_type(type_id) else {
            return;
        };

        let actual_type = *self.type_db.get_type(self.get_generic_type_information().types[generic_type_index]);
        let mutable_ty = self.type_db.get_type_mut(type_id);
        *mutable_ty = actual_type;

        trace!("Type ID '{type_id:?}' was generic, but is now '{actual_type:?}'");
    }
}

impl ProgramVisitor for GenericTypeVisitor<'_> {
    fn visit_function(&mut self, key: &FunctionKey, function: &mut Function) {
        let Some(generic_information) = function.generic_information.clone() else {
            trace!("{key:?} has no generic information, ignoring");
            return;
        };

        self.set_generic_information(generic_information);

        self.visit_type_id(function.return_type_id);

        for parameter in &mut function.parameters {
            self.visit_type_id(parameter.type_id);
        }

        for statement in &mut function.body {
            self.visit_statement(statement);
        }

        self.unset_generic_information();
    }

    fn visit_statement_variable_declaration(&mut self, _name: &str, _value: &mut Expression, _type_id: &mut TypeId) {}

    fn visit_statement_return(&mut self, _value: Option<&mut Expression>) {}

    fn visit_expression_binary_operation(
        &mut self,
        _left: &mut Expression,
        _right: &mut Expression,
        _operator: &mut BinaryOperator,
        _type_id: &mut TypeId,
    ) {
    }

    fn visit_expression_function_call(
        &mut self,
        _function_key: &FunctionKey,
        _arguments: &mut [Expression],
        _type_id: &mut TypeId,
    ) {
    }

    fn visit_expression_number_literal(&mut self, _value: &mut f64, _type_id: &mut TypeId) {}

    fn visit_variable_reference(&mut self, _variable_name: &mut str, _type_id: &mut TypeId) {}
}
