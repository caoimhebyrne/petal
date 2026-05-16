use std::collections::BTreeMap;

/// The ID of a [`Ty`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(usize);

/// A type on a node within the typed AST.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Type {
    /// A generic type which needs to be substituted for its concrete type when available.
    Generic(usize),

    /// A signed integer of a certain size (i{x}).
    SignedInteger(u8),

    /// An unsigned integer of a certain size (u{x}).
    UnsignedInteger(u8),

    /// Unit, typically the result of a function call.
    Void,
}

#[derive(Debug, Clone)]
pub struct TypeDb {
    /// The types allocated within this [`TypeBd`].
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

        Self { types, void_type_id }
    }
}

impl TypeDb {
    /// Returns the [`TypeId`] for the [`Type::Void`] type.
    pub fn void_type_id(&self) -> TypeId {
        self.void_type_id
    }
}

impl TypeDb {
    /// Retrieves a reference to a [`Type`] from the provided [`TypeId`].
    pub fn get_type(&self, type_id: TypeId) -> &Type {
        self.types.get(&type_id).expect("self.types.get should return `Some(_)`")
    }

    /// Retrieves a mutable reference to a [`Type`] from the provided [`TypeId`].
    pub fn get_type_mut(&mut self, type_id: TypeId) -> &mut Type {
        self.types.get_mut(&type_id).expect("self.types.get_mut should return `Some(_)`")
    }

    /// Allocates a new [`TypeId`] for the provided [`Type`].
    /// If a [`Type`] already exists, its existing ID will be returned and a new [`TypeId`] will not be allocated.
    pub fn get_or_insert_type(&mut self, ty: Type) -> TypeId {
        TypeDb::get_or_insert_type_into_map(&mut self.types, ty)
    }

    /// Allocates a new [`TypeId`] in the provided [`BTreeMap`] for the provided [`Type`].
    ///
    /// If a [`Type`] already exists, its existing ID will be returned and a new [`TypeId`] will not be allocated.
    ///
    /// The only exception to the rule above is [`Type::Generic`]s. A new type ID will _always_ be allocated for those.
    /// This is because a [`Type::Generic`] is only identified by the index of its generic parameter, and its variant
    /// does not have any unique information which ties it to the function.
    fn get_or_insert_type_into_map(types: &mut BTreeMap<TypeId, Type>, ty: Type) -> TypeId {
        if !matches!(ty, Type::Generic(_))
            && let Some((type_id, _)) = types.iter().find(|(_, it)| *it == &ty)
        {
            return *type_id;
        }

        let type_id = TypeId(types.len());
        types.insert(type_id, ty);
        type_id
    }
}
