use std::collections::HashMap;

use crate::error::LLVMCodegenError;
use inkwell::{
    types::BasicTypeEnum,
    values::{BasicValueEnum, PointerValue},
};
use petal_core::{
    error::Result,
    source_span::SourceSpan,
    string_intern::{StringInternPool, StringReference},
};

pub struct ScopeContext<'ctx> {
    /// A reference to the string intern pool to read string values from.
    string_intern_pool: &'ctx dyn StringInternPool,

    /// The variables that are declared within this scope.
    variables: HashMap<StringReference, Variable<'ctx>>,
}

impl<'ctx> ScopeContext<'ctx> {
    /// Creates a new [ScopeContext].
    pub fn new(string_intern_pool: &'ctx dyn StringInternPool) -> Self {
        ScopeContext {
            string_intern_pool,
            variables: HashMap::new(),
        }
    }

    /// Declares a variable within this [ScopeContext].
    pub fn declare_variable(&mut self, identifier: StringReference, variable: Variable<'ctx>) {
        self.variables.insert(identifier, variable);
    }

    /// Returns a [Variable] that has been declared within this [ScopeContext].
    ///
    /// This function will return an error if a variable with the provided identifier was not declared.
    pub fn get_variable(&mut self, identifier: &StringReference, span: SourceSpan) -> Result<&Variable<'ctx>> {
        match self.variables.get(identifier) {
            Some(value) => Ok(value),
            None => {
                let variable_name = self.string_intern_pool.resolve_reference_or_err(identifier, span)?;
                LLVMCodegenError::undeclared_variable(variable_name, span).into()
            }
        }
    }
}

/// A variable declared within a scope.
#[derive(Debug)]
pub struct Variable<'ctx> {
    /// The value type for the variable.
    pub value_type: BasicTypeEnum<'ctx>,

    /// The kind of variable this is (also includes its value).
    pub kind: VariableKind<'ctx>,
}

/// Represents the kinds of variables within a scope.
#[derive(Debug)]
pub enum VariableKind<'ctx> {
    /// A variable defined within the scope.
    Local(PointerValue<'ctx>),

    /// A variable passed as a parameter to the scope.
    Parameter(BasicValueEnum<'ctx>),
}

impl<'ctx> Variable<'ctx> {
    /// Creates a new [Variable] with the kind of [VariableKind::Local].
    pub fn local(value_type: BasicTypeEnum<'ctx>, value: PointerValue<'ctx>) -> Self {
        Variable {
            value_type,
            kind: VariableKind::Local(value),
        }
    }

    /// Creates a new [Variable] with the kind [VariableKind::Parameter].
    pub fn parameter(value_type: BasicTypeEnum<'ctx>, value: BasicValueEnum<'ctx>) -> Self {
        Variable {
            value_type,
            kind: VariableKind::Parameter(value),
        }
    }
}
