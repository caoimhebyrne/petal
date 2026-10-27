use std::collections::HashMap;

/// A unique ID assigned to a [`TypeKind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeKind {
    /// The nothing type.
    Void,

    /// A signed integer of a certain amount of bits.
    Integer(u16),

    /// An invalid type.
    Invalid,
}

pub struct Types {
    /// For lookup from TypeId -> TypeKind.
    vec: Vec<TypeKind>,

    /// For lookup from TypeKind -> TypeId.
    map: HashMap<TypeKind, TypeId>,

    /// The ID of the built-in [`TypeKind::Void`] type.
    void: TypeId,

    /// The ID of the built-in [`TypeKind::Invalid`] type.
    invalid: TypeId,
}

impl Default for Types {
    /// Create a new [`Types`] with the built-in types already interned.
    fn default() -> Self {
        let mut types = Self {
            vec: Vec::new(),
            map: HashMap::new(),
            void: TypeId(0),
            invalid: TypeId(0),
        };

        types.void = types.intern(TypeKind::Void);
        types.invalid = types.intern(TypeKind::Invalid);
        types
    }
}

impl Types {
    /// Return the ID of the built-in [`TypeKind::Void`] type.
    pub const fn void(&self) -> TypeId {
        self.void
    }

    /// Return the ID of the built-in [`TypeKind::Invalid`] type.
    pub const fn invalid(&self) -> TypeId {
        self.invalid
    }

    /// Retrieves the [`TypeKind`] associated with the provided [`TypeId`].
    ///
    /// # Panics
    ///
    /// This function will panic if the [`TypeId`] was not produced by this [`Types`].
    pub fn get(&self, id: TypeId) -> &TypeKind {
        self.vec
            .get(id.0 as usize)
            .expect("type id does not have an associated kind")
    }

    /// Return a [`TypeId`] for the provided [`TypeKind`].
    ///
    /// If the [`TypeKind`] has already been passed to this function, then its existing type ID will be returned.
    ///
    /// # Panics
    ///
    /// This function will panic if more than [`u32::MAX`] types are interned.
    pub fn intern(&mut self, kind: TypeKind) -> TypeId {
        // If the type has already been allocated, then we can just return it.
        if let Some(id) = self.map.get(&kind) {
            return *id;
        }

        // Otherwise, we can assign a new ID to it, and add it to both the vec and the map.
        let id = TypeId(u32::try_from(self.vec.len()).expect("too many types were interned"));
        self.vec.push(kind);
        self.map.insert(kind, id);
        id
    }
}
