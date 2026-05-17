use std::collections::{
    BTreeMap,
    btree_map::Keys,
};

use crate::typed_ast::r#type::{
    Type,
    defined::DefinedType,
};

/// The ID of a [`Type`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(usize);

/// The ID of a [`DefinedType`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DefinedTypeId(usize);

#[derive(Debug, Clone)]
pub struct TypeDb {
    /// The "custom" types that were defined by the program.
    defined_types: BTreeMap<DefinedTypeId, DefinedType>,

    /// The types allocated within this [`TypeDb`].
    types: BTreeMap<TypeId, Type>,

    /// The `void` type.
    void_type_id: TypeId,
}

impl Default for TypeDb {
    /// Creates a new [`TypeDb`], pre-populating some default `TypeId`s that can be accessed via their dedicated
    /// methods.
    fn default() -> Self {
        let mut types = BTreeMap::default();

        let void_type_id = Self::get_or_insert_type_into_map(&mut types, Type::Void);

        Self { defined_types: BTreeMap::default(), types, void_type_id }
    }
}

impl TypeDb {
    /// Returns the [`TypeId`] for the [`Type::Void`] type.
    pub fn void_type_id(&self) -> TypeId {
        self.void_type_id
    }
}

impl TypeDb {
    /// Retrieves a reference to a [`DefinedType`] from the provided [`DefinedType`].
    pub fn get_defined_type(&self, defined_type_id: DefinedTypeId) -> &DefinedType {
        self.defined_types.get(&defined_type_id).expect("self.defined_types.get should return `Some(_)`")
    }

    /// Finds a [`DefinedTypeId`] for the [`DefinedType`] which has a matching `name`.
    pub fn find_defined_type(&self, name: &str, generic_type_arguments: &[TypeId]) -> Option<DefinedTypeId> {
        self.defined_types
            .iter()
            .find(|(_, it)| {
                if it.name != name {
                    return false;
                }

                if let Some(generic_information) = &it.generic_information {
                    if generic_type_arguments.is_empty() {
                        return false;
                    }

                    if generic_type_arguments.len() != generic_information.parameters.len() {
                        return false;
                    }

                    for (parameter, argument_type_id) in
                        generic_information.parameters.iter().zip(generic_type_arguments)
                    {
                        if parameter.type_id != *argument_type_id {
                            return false;
                        }
                    }
                }

                true
            })
            .map(|it| *it.0)
    }

    /// Allocates a new [`DefinedTypeId`] for the provided [`DefinedType`].
    pub fn insert_defined_type(&mut self, defined_type: DefinedType) -> DefinedTypeId {
        let id = DefinedTypeId(self.defined_types.len());
        self.defined_types.insert(id, defined_type);
        id
    }

    /// Returns an [`Iter`] of [`DefinedTypeId`]s present in this [`TypeDb`].
    pub fn iter_defined_types(&self) -> Keys<'_, DefinedTypeId, DefinedType> {
        self.defined_types.keys()
    }
}

impl TypeDb {
    /// Retrieves a reference to a [`Type`] from the provided [`TypeId`].
    pub fn get_type(&self, type_id: TypeId) -> &Type {
        self.types.get(&type_id).expect("self.types.get should return `Some(_)`")
    }

    /// Allocates a new [`TypeId`] for the provided [`Type`].
    /// If a [`Type`] already exists, its existing ID will be returned and a new [`TypeId`] will not be allocated.
    pub fn get_or_insert_type(&mut self, ty: Type) -> TypeId {
        TypeDb::get_or_insert_type_into_map(&mut self.types, ty)
    }

    /// Allocates a new [`TypeId`] in the provided [`BTreeMap`] for the provided [`Type`].
    /// If a [`Type`] already exists, its existing ID will be returned and a new [`TypeId`] will not be allocated.
    fn get_or_insert_type_into_map(types: &mut BTreeMap<TypeId, Type>, ty: Type) -> TypeId {
        let type_id = TypeId(types.len());
        types.insert(type_id, ty);
        type_id
    }
}
