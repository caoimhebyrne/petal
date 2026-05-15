use std::{
    collections::{
        HashMap,
        HashSet,
    },
    fmt::Display,
    mem,
};

use crate::{
    ast::{
        statement::{
            function_declaration::{
                DeclarationModifier,
                FunctionDeclaration,
                FunctionParameter,
            },
            variable_declaration::VariableDeclaration,
        },
        type_expr::{
            EnumVariant,
            GenericTypeArgument,
            GenericTypeParameter,
            StructureField,
        },
    },
    core::span::Span,
    module_registry::ModuleId,
    typechecker::{
        error::{
            TypecheckerError,
            TypecheckerErrorKind,
        },
        r#type::Type,
    },
};

/// A unique identifier for a function.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FunctionId(usize);

impl Display for FunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A unique identifier for a declared type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DeclaredTypeId(usize);

impl Display for DeclaredTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The identifier for a structure type.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct StructureId(usize);

impl Display for StructureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The identifier for an enum type.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct EnumId(usize);

impl Display for EnumId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The built-in types that should be discovered from the standard library during compilation.
#[derive(Default, Clone)]
pub struct IncompleteBuiltinTypes {
    /// str (string::CompileTimeStr)
    pub compile_time_str: Option<StructureId>,
}

/// The different kinds of types that can be produced during compilation. They will not directly map to a type in the
/// source code.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SyntheticType {
    /// An optional type.
    Optional {
        // The inner value type.
        inner_type: Type,
    },
}

/// The context of a [`Typechecker`].
#[derive(Default)]
pub(crate) struct TypecheckerContext {
    /// The functions that have been validated by this [`Typechecker`] instance.
    pub(crate) functions: HashMap<FunctionId, CheckedFunction>,

    /// The enums that have been declared during compilation.
    pub(crate) enums: HashMap<EnumId, DeclaredEnum>,

    /// The current scope. By default, this is the global scope.
    pub(crate) scope: Scope,

    /// The types that have been declared by the user in the current scope.
    pub(crate) types: HashMap<DeclaredTypeId, DeclaredType>,

    /// The structures that have been declared by the user.
    pub(crate) structures: HashMap<StructureId, DeclaredStructure>,

    /// The specialized structures that have been generated during compilation.
    pub(crate) specialized_structures: HashMap<SpecializedStructureId, SpecializedStructure>,

    /// The specialized functions that have been generated during compilation.
    pub(crate) specialized_functions: HashMap<SpecializedFunctionId, SpecializedFunction>,

    /// The types that have been synthesised during compilation.
    ///
    /// This could include: optional type implementations and generic type implementations.
    pub(crate) synthetic_types: HashSet<SyntheticType>,

    /// The built-in types discovered during compilation.
    pub(crate) builtin_types: IncompleteBuiltinTypes,
}

/// A scope holds the variables that have been declared, and is typically created for each block.
#[derive(Default, Debug)]
pub(crate) struct Scope {
    /// The variables that have been declared in this scope.
    pub(crate) variables: HashMap<String, Type>,

    /// The smart-casts that have occurred within this scope.
    pub(crate) smart_casted_variables: HashMap<String, Type>,

    /// The parent scope, if this is a child.
    pub(crate) parent: Option<Box<Scope>>,

    /// The generic type parameters that are available in this scope.
    pub(crate) generic_type_parameters: Vec<GenericTypeParameter>,

    /// The type that "result value" of this scope intends to be.
    pub(crate) result_type: Type,
}

impl Scope {
    /// Creates a new scope which is a child of this scope.
    pub(crate) fn create_child(self, generic_type_parameters: Vec<GenericTypeParameter>, result_type: Type) -> Self {
        Self {
            variables: HashMap::default(),
            smart_casted_variables: HashMap::default(),
            parent: Some(Box::new(self)),
            generic_type_parameters,
            result_type,
        }
    }
}

impl Scope {
    /// Attempts to lookup the type of a variable in this scope.
    /// If the variable could not be found in this scope, the parent scope(s) are searched until one is found.
    pub(crate) fn lookup_variable(&self, name: &str) -> Option<&Type> {
        self.variables.get(name).or_else(|| self.parent.as_ref().and_then(|p| p.lookup_variable(name)))
    }
}

#[derive(Debug)]
pub struct FunctionLookupRequest {
    /// The name of the type which owns the function.
    pub owner_type_name: Option<String>,

    /// The name of the function.
    pub name: String,

    // The namespace that the function is defined in.
    pub namespace: Option<String>,
}

impl TypecheckerContext {
    /// Creates a child of the current scope, and makes it the current scope.
    pub fn push_child_scope(&mut self, generic_type_parameters: Vec<GenericTypeParameter>, result_type: Type) {
        self.scope = mem::take(&mut self.scope).create_child(generic_type_parameters, result_type);
    }

    /// Takes the parent of the current scope, and makes it the current scope.
    pub fn pop_child_scope(&mut self, span: Span) -> Result<(), TypecheckerError> {
        // TODO: Remove the unwrap.
        self.scope = *self.scope.parent.take().ok_or(TypecheckerErrorKind::ExpectedParentScope.at(span))?;
        Ok(())
    }

    pub(crate) fn insert_synthetic_type(&mut self, synthetic_type: SyntheticType) {
        self.synthetic_types.insert(synthetic_type);
    }

    /// Attempts to get a [`CheckedFunction`] from this [`Typechecker`] by its name.
    pub(crate) fn get_checked_function(
        &self,
        request: &FunctionLookupRequest,
        span: Span,
    ) -> Result<&CheckedFunction, TypecheckerError> {
        trace!("Attempting to find checked function from request: {:?}", request);

        // TODO: We should support function overloads.
        let function_candidates = self
            .functions
            .values()
            .filter(|it| {
                it.name == request.name
                    && it.owner_type_name == request.owner_type_name
                    && it.namespace == request.namespace
            })
            .filter(|it| it.is_visible_to_module(span.module_id))
            .collect::<Vec<_>>();

        if function_candidates.len() > 1 {
            debug!(
                "Lookup for function name '{}' (owned by '{:?}' in namespace '{:?}') in module {} has the following candidates: {}",
                request.name,
                request.owner_type_name,
                request.namespace,
                span.module_id,
                function_candidates.iter().map(|it| it.name.clone()).collect::<Vec<_>>().join(", ")
            );
            return Err(TypecheckerErrorKind::AmbiguousFunctionCall(request.name.clone()).at(span));
        }

        function_candidates
            .first()
            .copied()
            .ok_or(TypecheckerErrorKind::UndeclaredFunction(request.name.clone()).at(span))
    }

    /// Attempts to get a variable from this [`Typechecker`] by its name.
    pub(crate) fn get_variable(&self, name: &str, span: Span) -> Result<&Type, TypecheckerError> {
        self.scope.lookup_variable(name).ok_or(TypecheckerErrorKind::UndeclaredVariable(name.into()).at(span))
    }

    /// Inserts a [`CheckedFunction`] into this [`Typechecker`].
    pub(crate) fn insert_checked_function(
        &mut self,
        namespace: Option<String>,
        function_declaration: &FunctionDeclaration,
        span: Span,
    ) -> Result<FunctionId, TypecheckerError> {
        let function_id = FunctionId(self.functions.len());

        self.functions.insert(
            function_id,
            CheckedFunction::new(
                span.module_id,
                function_id,
                namespace,
                function_declaration.owner_type_name.clone(),
                function_declaration.name.clone(),
                function_declaration.parameters.clone(),
                function_declaration.generic_type_parameters.clone(),
                function_declaration.return_type.clone(),
                function_declaration.modifiers.clone(),
            ),
        );

        Ok(function_id)
    }

    /// Inserts a variable into this [`Typechecker`].
    pub(crate) fn insert_variable_from_declaration(
        &mut self,
        variable_declaration: &VariableDeclaration,
        span: Span,
    ) -> Result<(), TypecheckerError> {
        self.insert_variable(variable_declaration.name.clone(), variable_declaration.r#type.clone(), span)
    }

    /// Inserts a variable into this [`Typechecker`].
    pub(crate) fn insert_variable(&mut self, name: String, r#type: Type, span: Span) -> Result<(), TypecheckerError> {
        if self.get_variable(&name, span).is_ok() {
            return Err(TypecheckerErrorKind::DuplicateVariableDeclaration(name).at(span));
        }

        self.scope.variables.insert(name, r#type);
        Ok(())
    }

    /// Retrieves a [`DeclaredType`] from this [`TypecheckerContext`] by its name.
    pub(crate) fn get_declared_type_by_name(&self, name: &str, span: Span) -> Option<&DeclaredType> {
        self.types.values().find(|it| it.name == name && it.is_visible_to_module(span.module_id))
    }

    /// Inserts a [`DeclaredType`] into this [`TypecheckerContext`].
    pub(crate) fn insert_declared_type(
        &mut self,
        namespace: Option<String>,
        name: String,
        r#type: Type,
        modifiers: Vec<DeclarationModifier>,
        generic_type_parameters: Vec<GenericTypeParameter>,
        span: Span,
    ) -> Result<DeclaredTypeId, TypecheckerError> {
        self.insert_computed_declared_type(namespace, name, modifiers, generic_type_parameters, span, |_, _| r#type)
    }

    /// Inserts a [`DeclaredType`] into this [`TypecheckerContext`], executing [`type_fn`] with the [`DeclaredTypeId`] to
    /// get a [`Type`].
    pub(crate) fn insert_computed_declared_type<TypeFn>(
        &mut self,
        namespace: Option<String>,
        name: String,
        modifiers: Vec<DeclarationModifier>,
        generic_type_parameters: Vec<GenericTypeParameter>,
        span: Span,
        type_fn: TypeFn,
    ) -> Result<DeclaredTypeId, TypecheckerError>
    where
        TypeFn: FnOnce(&mut Self, DeclaredTypeId) -> Type,
    {
        // First, we must evaluate the latest type id.
        let id = DeclaredTypeId(self.types.len());

        // Then, we can evaluate the type.
        let r#type = type_fn(self, id);

        // And finally, we can insert the type.
        self.types.insert(
            id,
            DeclaredType { id, module_id: span.module_id, namespace, name, r#type, generic_type_parameters, modifiers },
        );

        Ok(id)
    }

    /// Inserts a [`DeclaredStructure`] into this [`TypecheckerContext`].
    pub(crate) fn insert_declared_structure(
        &mut self,
        declared_type_id: DeclaredTypeId,
        fields: Vec<StructureField>,
        _span: Span,
    ) -> StructureId {
        let structure_id = StructureId(self.structures.len());

        self.structures.insert(structure_id, DeclaredStructure::new(declared_type_id, fields));

        structure_id
    }

    /// Creates a [`SpecializedStructure`] from a [`DeclaredtypeId`] which maps to a generic structure.
    pub(crate) fn insert_specialized_structure(
        &mut self,
        generic_type_id: DeclaredTypeId,
        generic_type_arguments: Vec<GenericTypeArgument>,
        fields: Vec<StructureField>,
    ) -> SpecializedStructureId {
        // If a specialized structure already exists with the same type ID and arguments, then we can return it.
        if let Some((id, _)) = self.specialized_structures.iter().find(|(_, it)| {
            it.generic_type_id == generic_type_id && it.generic_type_arguments == generic_type_arguments
        }) {
            trace!("A specialized structure ({}) already exists for generic declared type {}", id, generic_type_id);
            return *id;
        }

        let id = SpecializedStructureId(self.specialized_structures.len());

        self.specialized_structures
            .insert(id, SpecializedStructure { generic_type_id, generic_type_arguments, fields });

        id
    }

    /// Creates a [`SpecializedFunction`] from a [`FunctionId`] which maps to a generic function declaration.
    pub(crate) fn insert_specialized_function(
        &mut self,
        generic_function_id: FunctionId,
        generic_type_arguments: Vec<GenericTypeArgument>,
        parameters: Vec<FunctionParameter>,
        return_type: Type,
    ) -> SpecializedFunctionId {
        // If a specialized structure already exists with the same type ID and arguments, then we can return it.
        if let Some((id, _)) = self.specialized_functions.iter().find(|(_, it)| {
            it.generic_function_id == generic_function_id
                && it.generic_type_arguments == generic_type_arguments
                && it.parameters == parameters
                && it.return_type == return_type
        }) {
            trace!(
                "A specialized function ({}) already exists for generic function declaration {}",
                id, generic_function_id
            );

            return *id;
        }

        let id = SpecializedFunctionId(self.specialized_functions.len());

        self.specialized_functions
            .insert(id, SpecializedFunction { generic_function_id, generic_type_arguments, parameters, return_type });

        id
    }

    /// Creates a [`DeclaredEnum`] and inserts it into this [`TypecheckerContext`].
    pub(crate) fn insert_enum(&mut self, declared_type_id: DeclaredTypeId, variants: Vec<EnumVariant>) -> EnumId {
        let id = EnumId(self.enums.len());

        self.enums.insert(id, DeclaredEnum { declared_type_id, variants });

        id
    }
}

/// A function which has been verified by the typechecker.
#[derive(Debug, Clone)]
pub struct CheckedFunction {
    /// The ID of the module that this function belongs to.
    pub module_id: ModuleId,

    /// The unique identifier for this function.
    pub function_id: FunctionId,

    /// The namespace that the function was declared in. `None` for the root namespace of the module.
    pub namespace: Option<String>,

    /// The name of the type which owns the function.
    pub owner_type_name: Option<String>,

    /// The declared name of the function.F
    pub name: String,

    /// The parameters to the function.
    pub parameters: Vec<FunctionParameter>,

    /// The generic type parameters of this function.
    pub generic_type_parameters: Vec<GenericTypeParameter>,

    /// The return type of the function.
    pub return_type: Type,

    /// The modifiers of the function.
    pub modifiers: Vec<DeclarationModifier>,
}

impl CheckedFunction {
    /// Creates a new [`CheckedFunction`].
    pub fn new(
        module_id: ModuleId,
        function_id: FunctionId,
        namespace: Option<String>,
        owner_type_name: Option<String>,
        name: String,
        parameters: Vec<FunctionParameter>,
        generic_type_parameters: Vec<GenericTypeParameter>,
        return_type: Type,
        modifiers: Vec<DeclarationModifier>,
    ) -> Self {
        Self {
            module_id,
            function_id,
            namespace,
            owner_type_name,
            name,
            parameters,
            generic_type_parameters,
            return_type,
            modifiers,
        }
    }

    /// Returns whether this [`CheckedFunction`] is visible to the provided module ID.
    ///
    /// By default, all functions are private, and can only be accessed by the module that they are defined in. If a
    /// function is marked with the 'public' modifier, then it can be accessed by any module.
    pub fn is_visible_to_module(&self, other_module_id: ModuleId) -> bool {
        if self.modifiers.contains(&DeclarationModifier::Public) {
            return true;
        }

        self.module_id == other_module_id
    }
}

/// A type which has been declared by the user.
#[derive(Debug, Clone)]
pub struct DeclaredType {
    /// The ID of this [`DeclaredType`].
    pub id: DeclaredTypeId,

    /// The module that the type was declared in.
    pub module_id: ModuleId,

    /// The namespace that the type was defined in. `None` for the root namespace of its module.
    pub namespace: Option<String>,

    /// The name of the type.
    pub name: String,

    /// The actual [`Type`].
    pub r#type: Type,

    /// The generic type parameters of this type.
    pub generic_type_parameters: Vec<GenericTypeParameter>,

    /// The modifiers of the type declaration.
    pub modifiers: Vec<DeclarationModifier>,
}

impl DeclaredType {
    /// Returns whether this [`DeclaredType`] is visible to the provided module ID.
    /// By default, all types are private, and can only be accessed by the module that they are defined in.
    pub fn is_visible_to_module(&self, other_module_id: ModuleId) -> bool {
        if self.modifiers.contains(&DeclarationModifier::Public) {
            return true;
        }

        self.module_id == other_module_id
    }
}

/// A structure type which has been declared in the source code.
#[derive(Debug, Clone)]
pub struct DeclaredStructure {
    /// The declared type ID associated with this structure.
    pub declared_type_id: DeclaredTypeId,

    /// The fields within the structure.
    pub fields: Vec<StructureField>,
}

impl DeclaredStructure {
    /// Creates a new [`DeclaredStructure`].
    pub fn new(declared_type_id: DeclaredTypeId, fields: Vec<StructureField>) -> Self {
        Self { declared_type_id, fields }
    }
}

/// The identifier for a [`SpecializedStructure`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SpecializedStructureId(usize);

/// A structure which has been specialized due to it having generic types.
#[derive(Debug, Clone)]
pub struct SpecializedStructure {
    /// The declared type ID associated with the non-specialized variant of this structure.
    pub generic_type_id: DeclaredTypeId,

    /// The generic type arguments applied to this specialization.
    pub generic_type_arguments: Vec<GenericTypeArgument>,

    /// The fields within this structure.
    pub fields: Vec<StructureField>,
}

impl Display for SpecializedStructureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The identiifer for a [`SpecializedFunction`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SpecializedFunctionId(usize);

/// A function which has been specialized due to it having generic types.
#[derive(Debug, Clone)]
pub struct SpecializedFunction {
    /// The ID of the non-specialized variant of this function.
    /// This can be used to get the function's module ID, name, modifiers, namespace, etc.
    pub generic_function_id: FunctionId,

    /// The generic type arguments applied to this specialization.
    pub generic_type_arguments: Vec<GenericTypeArgument>,

    /// The specialized parameters of this function.
    pub parameters: Vec<FunctionParameter>,

    /// The specialzied return type of this function.
    pub return_type: Type,
}

impl Display for SpecializedFunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An enum type which has been declared in the source code.
#[derive(Debug, Clone)]
pub struct DeclaredEnum {
    /// The declared type ID associated with this enum.
    pub declared_type_id: DeclaredTypeId,

    /// The variants within the enum.
    pub variants: Vec<EnumVariant>,
}
