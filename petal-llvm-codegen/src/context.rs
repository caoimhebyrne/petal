use std::collections::HashMap;

use inkwell::{
    types::BasicTypeEnum,
    values::{BasicValueEnum, PointerValue},
};
use petal_core::{error::Result, source_span::SourceSpan, string_intern::StringReference};

use crate::error::LLVMCodegenErrorKind;

/// The context for an LLVM code generator.
#[derive(Debug)]
pub struct CodegenContext<'ctx> {
    /// The scope context that is currently bound. If one is not present, a scope has not been started yet.
    scope_context: Option<ScopeContext<'ctx>>,
}

impl<'ctx> CodegenContext<'ctx> {
    /// Creates a new [CodegenContext].
    pub fn new() -> Self {
        CodegenContext { scope_context: None }
    }

    /// Starts a new scope within this context.
    pub fn start_scope_context(&mut self) {
        self.scope_context = Some(ScopeContext::new());
    }

    /// Returns a reference to the current [ScopeContext].
    ///
    /// This function will return an error if a [ScopeContext] is not yet bound.
    pub fn scope_context(&mut self, span: SourceSpan) -> Result<&mut ScopeContext<'ctx>> {
        self.scope_context
            .as_mut()
            .ok_or(LLVMCodegenErrorKind::missing_scope_context(span).into())
    }

    /// Destroys the scope within this context.
    pub fn end_scope_context(&mut self) {
        self.scope_context = None;
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
    Local(PointerValue<'ctx>),
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

#[derive(Debug)]
pub struct ScopeContext<'ctx> {
    /// The variables that are declared within this scope.
    variables: HashMap<StringReference, Variable<'ctx>>,
}

impl<'ctx> ScopeContext<'ctx> {
    /// Creates a new [ScopeContext].
    pub fn new() -> Self {
        ScopeContext {
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
        self.variables
            .get(identifier)
            .ok_or(LLVMCodegenErrorKind::undeclared_variable(*identifier, span).into())
    }
}
