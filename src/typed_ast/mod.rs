#![allow(dead_code)]

/// The typed AST is emitted by the typechecker as it resolves the types involved in the normal AST.
use std::collections::BTreeMap;

use crate::{
    ast::expression::binary_operation::BinaryOperator,
    core::span::Span,
    module_registry::ModuleId,
    typed_ast::r#type::{
        TypeDb,
        TypeId,
    },
};

pub(super) mod context;
pub(super) mod error;
pub(super) mod resolver;
pub(super) mod r#type;
pub(super) mod visitor;

/// A program is the "output" of the typed AST. It contains all of the functions and types that are used.
#[derive(Default, Debug, Clone)]
pub struct Program {
    /// The functions within this program.
    functions: BTreeMap<FunctionKey, Function>,

    /// The [`TypeDb`] containing the [`Type`]s used by this program.
    type_db: TypeDb,
}

impl Program {
    /// Finds a [`Function`] given its name.
    pub fn find_function(&self, name: &str) -> Option<(&FunctionKey, &Function)> {
        // todo(resolver): find function request
        // todo(resolver): module id
        // todo(resolver): namespace
        // todo(resolver): etc
        self.functions.iter().find(|(_, it)| it.name == name)
    }

    /// Inserts a [`Function`] into this [`Program`].
    pub fn insert_function(&mut self, module_id: ModuleId, function: Function) -> FunctionKey {
        let key = FunctionKey { id: self.functions.len(), module_id };

        // TODO: What should we do if a function already exists with (basically) the same information?
        self.functions.insert(key, function);

        key
    }
}

/// A key for a function within the typed AST. This is a combination of the function ID and the ID of the module that
/// it was declared in.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FunctionKey {
    /// The ID of the function.
    id: usize,

    /// The ID of the module.
    pub module_id: ModuleId,
}

/// A function within a typed AST.
#[derive(Debug, Clone)]
pub struct Function {
    /// The name (as defined in the source code) of this function.
    pub name: String,

    /// The parameters of this function.
    pub parameters: Vec<FunctionParameter>,

    /// The body of this function.
    pub body: Vec<Statement>,

    /// The return type of this function.
    pub return_type_id: TypeId,

    /// Information about the generic types within this function, this is typically populated during the
    /// generation of the specialized function, and may be read by later stages.
    pub generic_information: Option<GenericInformation>,

    /// The span that this function was defined at in the source code.
    pub span: Span,
}

/// Information associated with a generic type or function.
#[derive(Debug, Clone)]
pub struct GenericInformation {
    /// A [`Vec`] of [`TypeId`]s, which correspond to the generic type arguments for each generic type parameter.
    pub types: Vec<TypeId>,
}

/// A parameter to a [`Function`].
#[derive(Debug, Clone)]
pub struct FunctionParameter {
    /// The name of the parameter.
    pub name: String,

    /// The type of the parameter.
    pub type_id: TypeId,

    /// Whether the parameter is named.
    ///
    /// A named parameter requires the function call to specify its name when providing an argument. By default, all
    /// function parameters are positional (un-named).
    pub is_named: bool,

    /// The span that the parameter was defined at in the source code.
    pub span: Span,
}

/// A statement within the typed AST is very similar to a statement in the regular AST. It may contain additional
/// information about the types involved in the program.
#[derive(Debug, Clone)]
pub struct Statement {
    /// The kind of statement that this is.
    pub kind: StatementKind,

    /// The span that this statement occurred at within the source code.
    pub span: Span,
}

/// The different kinds of typed [`Statement`]s that exist within the typed AST.
#[derive(Debug, Clone)]
pub enum StatementKind {
    /// A function call.
    FunctionCall {
        /// The key of the function being called.
        function_key: FunctionKey,

        /// The arguments of this function call.
        arguments: Vec<Expression>,

        /// The expected return type of this function call.
        return_type_id: TypeId,
    },

    /// A return statement.
    /// The value may or may not exist, and if it does, it should match the current function's return type.
    Return(Option<Expression>),

    /// A variable declaration.
    /// The identifier provided must not already be assigned to a variable.
    VariableDeclaration {
        /// The name of the variable being declared.
        name: String,

        /// The expression containing the initial value of the variable.
        value: Expression,

        /// The declared type of the variable.
        type_id: TypeId,
    },
}

impl StatementKind {
    /// Creates a [`Statement`] from this [`StatementKind`] and the provided [`Span`].
    pub fn at(self, span: Span) -> Statement {
        Statement { kind: self, span }
    }
}

/// An expression within the typed AST is very similar to an expression in the regular AST. It may contain
/// additional information about the types involved in the program.
#[derive(Debug, Clone)]
pub struct Expression {
    /// The kind of expression that this is.
    pub kind: ExpressionKind,

    /// The type that this expression is expected to produce once evaluated.
    pub type_id: TypeId,

    /// The span that this expression occurred at within the source code.
    pub span: Span,
}

/// The different kinds of typed [`Expression`]s that exist within the typed AST.
#[derive(Debug, Clone)]
pub enum ExpressionKind {
    /// A binary operation between two expressions.
    BinaryOperation {
        /// The left-hand side of the expression.
        left: Box<Expression>,

        /// The right-hand side of the expression.
        right: Box<Expression>,

        /// The operator to use on the [`left`]-hand and [`right`]-hand sides of the expression.
        operator: BinaryOperator,
    },

    /// A function call.
    FunctionCall {
        /// The key of the function being called.
        function_key: FunctionKey,

        /// The arguments of this function call.
        arguments: Vec<Expression>,
    },

    /// A number literal. This can be any integer, float, etc.
    NumberLiteral(f64),

    /// A reference to a local variable by name.
    VariableReference(String),
}
