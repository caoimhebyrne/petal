use crate::typed_ast::{
    Function,
    FunctionKey,
    GenericInformation,
    Program,
    r#type::{
        Type,
        db::{
            TypeDb,
            TypeId,
        },
    },
    visitor::{
        ProgramVisitor,
        walk_program,
    },
};

/// Responsible for resolving any [`Ty::Generic`] references within specialized functions.
pub struct GenericTypeVisitor<'db> {
    /// The [`TypeDb`].
    type_db: &'db mut TypeDb,
}

impl<'db> GenericTypeVisitor<'db> {
    /// Creates a new [`GenericTypeVisitor`] which will resolve any generic types in specialized functions and types
    /// against the provided [`TypeDb`].
    pub fn new(type_db: &'db mut TypeDb) -> Self {
        Self { type_db }
    }

    /// A convenience method for calling [`Self::new`] and [`Self::visit`].
    pub fn visit(program: &'db mut Program) {
        let mut visitor = Self::new(&mut program.type_db);
        walk_program(&mut visitor, &mut program.functions);
    }

    /// Visits the type behind the provided [`TypeId`], resolving it to its concrete type if it is [`Type::Generic`].
    fn visit_type_id(&mut self, generic_information: &GenericInformation, type_id: TypeId) {
        let Type::Generic(generic_type_index) = *self.type_db.get_type(type_id) else {
            return;
        };

        let actual_type = *self.type_db.get_type(generic_information.types[generic_type_index]);
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

        self.visit_type_id(&generic_information, function.return_type_id);

        for parameter in &mut function.parameters {
            self.visit_type_id(&generic_information, parameter.type_id);
        }

        // Note: we don't need to walk the function body
    }
}
