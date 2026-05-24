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
    /// The `bool` type.
    boolean_type_id: TypeId,

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

        let boolean_type_id = Self::get_or_insert_type_into_map(&mut types, Type::Boolean);
        let void_type_id = Self::get_or_insert_type_into_map(&mut types, Type::Void);

        Self { boolean_type_id, defined_types: BTreeMap::default(), types, void_type_id }
    }
}

impl TypeDb {
    /// Returns the [`TypeId`] for the [`Type::Boolean`] type.
    pub fn boolean_type_id(&self) -> TypeId {
        self.boolean_type_id
    }

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
        if let Some(tuple) = types.iter().find(|it| it.1 == &ty) {
            return *tuple.0;
        }

        let type_id = TypeId(types.len());
        types.insert(type_id, ty);
        type_id
    }

    /// Returns a human-readable name for the [`Type`] from the provided [`TypeId`].
    pub fn get_type_description(&self, type_id: TypeId) -> String {
        let ty = self.get_type(type_id);
        match ty {
            Type::Boolean => "bool".to_string(),
            Type::Defined(defined_type_id) => {
                let defined_type = self.get_defined_type(*defined_type_id);

                if let Some(generic_information) = &defined_type.generic_information {
                    let generic_type_arguments = generic_information
                        .parameters
                        .iter()
                        .map(|it| self.get_type_description(it.type_id))
                        .collect::<Vec<String>>()
                        .join(",");

                    format!("{}<{}>", defined_type.name, generic_type_arguments)
                } else {
                    defined_type.name.clone()
                }
            }
            Type::Reference(inner_type_id) => {
                format!("&{}", self.get_type_description(*inner_type_id))
            }
            Type::SignedInteger(bits) => format!("i{bits}"),
            Type::UnsignedInteger(bits) => format!("u{bits}"),
            Type::Void => "void".to_string(),
        }
    }
}
