use std::collections::HashMap;

use crate::{
    ast::{
        statement::{
            function_declaration::{
                FunctionDeclaration,
                FunctionParameter,
            },
            variable_declaration::VariableDeclaration,
        },
        type_expr::TypeExpr,
    },
    core::span::Span,
    module::{
        CheckedModule,
        ParsedModule,
    },
    typechecker::{
        error::{
            TypecheckerError,
            TypecheckerErrorKind,
        },
        r#type::Type,
    },
};

pub mod error;
pub mod expression;
pub mod statement;
pub mod r#type;

/// A function which has been verified by the typechecker.
#[derive(Debug, Clone)]
pub(crate) struct CheckedFunction {
    /// The parameters to the function.
    pub parameters: Vec<FunctionParameter>,

    /// The return type of the function.
    pub return_type: Type,
}

impl CheckedFunction {
    /// Creates a new [`CheckedFunction`].
    pub fn new(parameters: Vec<FunctionParameter>, return_type: Type) -> Self {
        Self { parameters, return_type }
    }
}

/// The typechecker.
///
/// This is responsible for resolving and validating the types within a [`ParsedModule`].
#[derive(Default)]
pub struct Typechecker {
    /// The expected return type of the current function.
    expected_return_type: Type,

    // TODO: Function IDs?
    /// The functions that have been validated by this [`Typechecker`] instance.
    functions: HashMap<String, CheckedFunction>,

    /// The variables that have been declared in the current scope.
    variables: HashMap<String, Type>,
}

impl Typechecker {
    /// Checks and resolved any [`Type`]s referenced in the provided [`ParsedModule`].
    pub fn check(&mut self, mut module: ParsedModule) -> Result<CheckedModule, TypecheckerError> {
        for statement in &mut module.ast {
            self.check_statement(statement)?;
        }

        Ok(CheckedModule::new(module.id, module.ast))
    }

    /// Attempts to get a [`CheckedFunction`] from this [`Typechecker`] by its name.
    fn get_checked_function(&self, name: &str, span: Span) -> Result<&CheckedFunction, TypecheckerError> {
        self.functions.get(name).ok_or(TypecheckerErrorKind::UndeclaredFunction(name.into()).at(span))
    }

    /// Attempts to get a variable from this [`Typechecker`] by its name.
    fn get_variable(&self, name: &str, span: Span) -> Result<&Type, TypecheckerError> {
        self.variables.get(name).ok_or(TypecheckerErrorKind::UndeclaredVariable(name.into()).at(span))
    }

    /// Inserts a [`CheckedFunction`] into this [`Typechecker`].
    fn insert_checked_function(
        &mut self,
        function_declaration: &FunctionDeclaration,
        span: Span,
    ) -> Result<(), TypecheckerError> {
        if self.functions.contains_key(&function_declaration.name) {
            return Err(TypecheckerErrorKind::DuplicateFunctionDeclaration(function_declaration.name.clone()).at(span));
        }

        self.functions.insert(
            function_declaration.name.clone(),
            CheckedFunction::new(function_declaration.parameters.clone(), function_declaration.return_type),
        );

        Ok(())
    }

    /// Inserts a variable into this [`Typechecker`].
    fn insert_variable_from_declaration(
        &mut self,
        variable_declaration: &VariableDeclaration,
        span: Span,
    ) -> Result<(), TypecheckerError> {
        self.insert_variable(variable_declaration.name.clone(), variable_declaration.r#type, span)
    }

    /// Inserts a variable into this [`Typechecker`].
    fn insert_variable(&mut self, name: String, r#type: Type, span: Span) -> Result<(), TypecheckerError> {
        if self.variables.contains_key(&name) {
            return Err(TypecheckerErrorKind::DuplicateVariableDeclaration(name).at(span));
        }

        self.variables.insert(name, r#type);
        Ok(())
    }

    /// Attempts to resolve the provided [`TypeExpr`] into a [`Type`].
    fn resolve_type_from_expr(expr: &TypeExpr, span: Span) -> Result<Type, TypecheckerError> {
        let TypeExpr::Named(name) = expr;

        let r#type = match name.as_str() {
            "i8" => Type::SignedInteger(8),
            "i16" => Type::SignedInteger(16),
            "i32" => Type::SignedInteger(32),
            "i64" => Type::SignedInteger(64),

            "u8" => Type::UnsignedInteger(8),
            "u16" => Type::UnsignedInteger(16),
            "u32" => Type::UnsignedInteger(32),
            "u64" => Type::UnsignedInteger(64),

            "bool" => Type::Boolean,

            _ => return Err(TypecheckerErrorKind::UnknownType(name.clone()).at(span)),
        };

        Ok(r#type)
    }
}
