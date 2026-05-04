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
        type_expr::StructureField,
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

/// The identifier for a structure type.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct StructureId(usize);

impl Display for StructureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The context of a [`Typechecker`].
#[derive(Default)]
pub(crate) struct TypecheckerContext {
    /// The functions that have been validated by this [`Typechecker`] instance.
    pub(crate) functions: HashMap<FunctionId, CheckedFunction>,

    /// The current scope. By default, this is the global scope.
    pub(crate) scope: Scope,

    /// The types that have been declared by the user in the current scope.
    pub(crate) types: HashMap<DeclaredTypeId, DeclaredType>,

    /// The structures that have been declared by the user.
    pub(crate) structures: HashMap<StructureId, DeclaredStructure>,

    /// The optional types used during compilation. This is temporary.
    pub(crate) optional_types: HashSet<Type>,
}

/// A scope holds the variables that have been declared, and is typically created for each block.
#[derive(Default, Debug)]
pub(crate) struct Scope {
    /// The variables that have been declared in this scope.
    pub(crate) variables: HashMap<String, Type>,

    /// The parent scope, if this is a child.
    pub(crate) parent: Option<Box<Scope>>,

    /// The type that "result value" of this scope intends to be.
    pub(crate) result_type: Type,
}

impl Scope {
    /// Creates a new scope which is a child of this scope.
    pub(crate) fn create_child(self, result_type: Type) -> Self {
        Self { variables: HashMap::default(), parent: Some(Box::new(self)), result_type }
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
}

impl TypecheckerContext {
    /// Creates a child of the current scope, and makes it the current scope.
    pub fn push_child_scope(&mut self, result_type: Type) {
        self.scope = mem::take(&mut self.scope).create_child(result_type);
    }

    /// Takes the parent of the current scope, and makes it the current scope.
    pub fn pop_child_scope(&mut self) {
        // TODO: Remove the unwrap.
        self.scope = *self.scope.parent.take().unwrap();
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
            .filter(|it| it.declared_name == request.name && it.owner_type_name == request.owner_type_name)
            .filter(|it| it.is_visible_to_module(span.module_id))
            .collect::<Vec<_>>();

        if function_candidates.len() > 1 {
            debug!(
                "Lookup for function name '{}' (owned by '{:?}') in module {} has the following candidates: {}",
                request.name,
                request.owner_type_name,
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

    /// Attempts to get a [`CheckedFunction`] from this [`Typechecker`] by its its [`FunctionId`].
    pub(crate) fn get_checked_function_by_id(&self, id: FunctionId) -> &CheckedFunction {
        self.functions.get(&id).expect("functions.get should always succeed")
    }

    /// Attempts to get a variable from this [`Typechecker`] by its name.
    pub(crate) fn get_variable(&self, name: &str, span: Span) -> Result<&Type, TypecheckerError> {
        self.scope.lookup_variable(name).ok_or(TypecheckerErrorKind::UndeclaredVariable(name.into()).at(span))
    }

    /// Inserts a [`CheckedFunction`] into this [`Typechecker`].
    pub(crate) fn insert_checked_function(
        &mut self,
        function_declaration: &FunctionDeclaration,
        span: Span,
    ) -> Result<FunctionId, TypecheckerError> {
        let function_id = FunctionId(self.functions.len());

        self.functions.insert(
            function_id,
            CheckedFunction::new(
                span.module_id,
                function_id,
                function_declaration.owner_type_name.clone(),
                function_declaration.name.clone(),
                function_declaration.parameters.clone(),
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
        name: String,
        r#type: Type,
        span: Span,
    ) -> Result<DeclaredTypeId, TypecheckerError> {
        let type_id = DeclaredTypeId(self.types.len());

        self.types.insert(type_id, DeclaredType::new(span.module_id, name, r#type));

        Ok(type_id)
    }

    /// Retrieves a [`DeclaredStructure`] from this [`TypecheckerContext`] by its ID.
    pub(crate) fn get_declared_structure(&self, id: &StructureId) -> &DeclaredStructure {
        self.structures.get(id).expect("structures.get should always succeed")
    }

    /// Inserts a [`DeclaredStructure`] into this [`TypecheckerContext`].
    pub(crate) fn insert_declared_structure(
        &mut self,
        name: String,
        fields: Vec<StructureField>,
        span: Span,
    ) -> Result<StructureId, TypecheckerError> {
        let structure_id = StructureId(self.structures.len());

        self.structures.insert(structure_id, DeclaredStructure::new(span.module_id, name, fields));

        Ok(structure_id)
    }
}

/// A function which has been verified by the typechecker.
#[derive(Debug, Clone)]
pub struct CheckedFunction {
    /// The ID of the module that this function belongs to.
    pub module_id: ModuleId,

    /// The unique identifier for this function.
    pub function_id: FunctionId,

    /// The name of the type which owns the function.
    pub owner_type_name: Option<String>,

    /// The name of the function.
    pub name: String,

    /// The declared name of the function.
    pub declared_name: String,

    /// The parameters to the function.
    pub parameters: Vec<FunctionParameter>,

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
        owner_type_name: Option<String>,
        declared_name: String,
        parameters: Vec<FunctionParameter>,
        return_type: Type,
        modifiers: Vec<DeclarationModifier>,
    ) -> Self {
        // FIXME: Add a modifier to function declarations which prevents their names from being mangled.
        let name = if declared_name == "main" {
            declared_name.clone()
        } else if let Some(owner_type_name) = &owner_type_name {
            format!("ptl_mod_{module_id}_fn_{owner_type_name}_{declared_name}")
        } else {
            format!("ptl_mod_{module_id}_fn_{declared_name}")
        };

        Self { module_id, function_id, owner_type_name, name, declared_name, parameters, return_type, modifiers }
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
pub(crate) struct DeclaredType {
    /// The module that the type was declared in.
    pub module_id: ModuleId,

    /// The name of the type.
    pub name: String,

    /// The actual [`Type`].
    pub r#type: Type,
}

impl DeclaredType {
    /// Creates a new [`DeclaredType`].
    pub fn new(module_id: ModuleId, name: String, r#type: Type) -> Self {
        Self { module_id, name, r#type }
    }

    /// Returns whether this [`DeclaredType`] is visible to the provided module ID.
    /// By default, all types are private, and can only be accessed by the module that they are defined in.
    pub fn is_visible_to_module(&self, other_module_id: ModuleId) -> bool {
        self.module_id == other_module_id
    }
}

/// A structure type which has been declared in the source code.
#[derive(Debug, Clone)]
pub struct DeclaredStructure {
    /// The module that the structure was declared in.
    pub _module_id: ModuleId,

    /// The internal name of the structure.
    pub name: String,

    /// The name of the structure, as declared by the user.
    pub declared_name: String,

    /// The fields within the structure.
    pub fields: Vec<StructureField>,
}

impl DeclaredStructure {
    /// Creates a new [`DeclaredStructure`].
    pub fn new(module_id: ModuleId, declared_name: String, fields: Vec<StructureField>) -> Self {
        Self {
            _module_id: module_id,
            name: format!("ptl_mod_{module_id}_struct_{declared_name}"),
            declared_name,
            fields,
        }
    }
}
