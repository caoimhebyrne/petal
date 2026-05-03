use std::{
    collections::HashMap,
    fmt::Display,
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
    /// The expected return type of the current function.
    pub(crate) expected_return_type: Type,

    /// The functions that have been validated by this [`Typechecker`] instance.
    pub(crate) functions: HashMap<FunctionId, CheckedFunction>,

    /// The variables that have been declared in the current scope.
    pub(crate) variables: HashMap<String, Type>,

    /// The types that have been declared by the user in the current scope.
    pub(crate) types: HashMap<DeclaredTypeId, DeclaredType>,

    /// The structures that have been declared by the user.
    pub(crate) structures: HashMap<StructureId, DeclaredStructure>,
}

impl TypecheckerContext {
    /// Attempts to get a [`CheckedFunction`] from this [`Typechecker`] by its name.
    pub(crate) fn get_checked_function(&self, name: &str, span: Span) -> Result<&CheckedFunction, TypecheckerError> {
        // TODO: We should support function overloads.
        // FIXME: Functions need to be generated and prefixed with their module name. How do we do that?
        let function_candidates = self
            .functions
            .values()
            .filter(|it| it.declared_name == name)
            .filter(|it| it.is_visible_to_module(span.module_id))
            .collect::<Vec<_>>();

        if function_candidates.len() > 1 {
            debug!(
                "Lookup for function name '{}' in module {} has the following candidates: {}",
                name,
                span.module_id,
                function_candidates.iter().map(|it| it.name.clone()).collect::<Vec<_>>().join(", ")
            );
            return Err(TypecheckerErrorKind::AmbiguousFunctionCall(name.into()).at(span));
        }

        function_candidates.first().copied().ok_or(TypecheckerErrorKind::UndeclaredFunction(name.into()).at(span))
    }

    /// Attempts to get a [`CheckedFunction`] from this [`Typechecker`] by its its [`FunctionId`].
    pub(crate) fn get_checked_function_by_id(&self, id: FunctionId) -> &CheckedFunction {
        self.functions.get(&id).expect("functions.get should always succeed")
    }

    /// Attempts to get a variable from this [`Typechecker`] by its name.
    pub(crate) fn get_variable(&self, name: &str, span: Span) -> Result<&Type, TypecheckerError> {
        self.variables.get(name).ok_or(TypecheckerErrorKind::UndeclaredVariable(name.into()).at(span))
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
        if self.variables.contains_key(&name) {
            return Err(TypecheckerErrorKind::DuplicateVariableDeclaration(name).at(span));
        }

        self.variables.insert(name, r#type);
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
pub(crate) struct CheckedFunction {
    /// The ID of the module that this function belongs to.
    pub module_id: ModuleId,

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
        declared_name: String,
        parameters: Vec<FunctionParameter>,
        return_type: Type,
        modifiers: Vec<DeclarationModifier>,
    ) -> Self {
        Self {
            module_id,
            // FIXME: Add a modifier to function declarations which prevents their names from being mangled.
            name: if declared_name == "main" {
                declared_name.clone()
            } else {
                format!("ptl_mod_{module_id}_fn_{declared_name}")
            },
            declared_name,
            parameters,
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

    /// The name of the structure.
    pub name: String,

    /// The fields within the structure.
    pub fields: Vec<StructureField>,
}

impl DeclaredStructure {
    /// Creates a new [`DeclaredStructure`].
    pub fn new(module_id: ModuleId, name: String, fields: Vec<StructureField>) -> Self {
        Self { _module_id: module_id, name, fields }
    }
}
