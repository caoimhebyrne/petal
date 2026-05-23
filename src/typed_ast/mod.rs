use std::collections::BTreeMap;

use crate::{
    ast::{
        expression::binary_operation::BinaryOperator,
        statement::function_declaration::DeclarationModifier,
    },
    core::span::Span,
    module_registry::ModuleId,
    typed_ast::r#type::db::{
        TypeDb,
        TypeId,
    },
};

pub(crate) mod error;
pub(crate) mod resolver;
pub(crate) mod r#type;
pub(crate) mod visitor;

/// A program is the "output" of the typed AST. It contains all of the functions and types that are used.
#[derive(Default, Debug, Clone)]
pub struct Program {
    /// The functions within this program.
    functions: BTreeMap<FunctionKey, Function>,

    /// The [`TypeDb`] containing the [`Type`]s used by this program.
    type_db: TypeDb,
}

impl Program {
    /// Retrieves a reference to a [`Function`] from its [`FunctionKey`].
    pub fn get_function(&self, function_key: &FunctionKey) -> &Function {
        &self.functions[function_key]
    }

    /// Finds a [`Function`] given its name.
    pub fn find_function(
        &self,
        target: &FunctionCallTarget,
        generic_type_arguments: &[TypeId],
    ) -> Option<(&FunctionKey, &Function)> {
        // todo(resolver): module id
        self.functions.iter().find(|(_, it)| {
            if it.name != target.plain_name() {
                return false;
            }

            // If this is a namespaced function, then the function must be in the same namespace.
            if let FunctionCallTarget::Function { namespace, .. } = target
                && namespace != &it.namespace
            {
                return false;
            }

            // If this is a function call which has a specific type receiver, then this function must be owned by that
            // type.
            let receiver_type_id = match target {
                FunctionCallTarget::Associated { type_id, .. } => Some(*type_id),
                FunctionCallTarget::Function { .. } => None,
                FunctionCallTarget::Method { receiver, .. } => Some(receiver.type_id),
            };

            if receiver_type_id != it.owner_type_id {
                return false;
            }

            if let Some(generic_information) = &it.generic_information {
                // fixme: this might not be the best place, but we need to ignore the implicit `This` type parameter.
                let parameters =
                    generic_information.parameters.iter().filter(|it| it.name != "This").collect::<Vec<_>>();

                if generic_type_arguments.len() != parameters.len() {
                    return false;
                }

                for (parameter, argument_type_id) in parameters.iter().zip(generic_type_arguments) {
                    if parameter.type_id != *argument_type_id {
                        return false;
                    }
                }
            }

            true
        })
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
    /// The modifiers applied to the function definition.
    pub modifiers: Vec<DeclarationModifier>,

    /// The namespace that the function was defined in.
    pub namespace: Option<String>,

    /// The name (as defined in the source code) of this function.
    pub name: String,

    /// The parameters of this function.
    pub parameters: Vec<FunctionParameter>,

    /// The body of this function.
    pub body: Vec<Statement>,

    /// The return type of this function.
    pub return_type_id: TypeId,

    /// The ID of the type that "owns" this function. This means that the type is used to qualify the method call,
    /// either through an instance method call, or a static method call.
    pub owner_type_id: Option<TypeId>,

    /// Information about the generic types within this function, this is typically populated during the
    /// generation of the specialized function, and may be read by later stages.
    pub generic_information: Option<GenericInformation>,

    /// The span that this function was defined at in the source code.
    pub span: Span,
}

/// A type parameter associated with a generic type or function.
#[derive(Debug, Clone, PartialEq)]
pub struct GenericTypeParameter {
    /// The name of this generic type.
    pub name: String,

    /// The type ID allocated to this generic type.
    pub type_id: TypeId,
}

/// Information associated with a generic type or function.
#[derive(Debug, Clone, PartialEq)]
pub struct GenericInformation {
    /// The generic type parameters used when defining this type or function.
    pub parameters: Vec<GenericTypeParameter>,
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

    /// Assigning a value to a reference.
    ReferenceValueAssignment {
        /// The expression which yields the reference to assign the value to.
        target: Expression,

        /// The value being assigned to the reference.
        value: Expression,
    },

    /// A return statement.
    /// The value may or may not exist, and if it does, it should match the current function's return type.
    Return(Option<Expression>),

    /// Setting the value of a field on a structure type.
    StructureFieldAssignment {
        /// The expression providing the structure value.
        target: Box<Expression>,

        /// The index of the field being set.
        field_index: usize,

        /// The value to assign to the field.
        value: Box<Expression>,
    },

    /// A variable assignment.
    VariableAssignment {
        /// The name of the variable being assigned to.
        name: String,

        /// The value being assigned to the variable.
        value: Expression,

        /// The type of the variable.
        variable_type_id: TypeId,
    },

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

    /// Reading the value within a reference.
    Dereference(Box<Expression>),

    /// A function call.
    FunctionCall {
        /// The key of the function being called.
        function_key: FunctionKey,

        /// The arguments of this function call.
        arguments: Vec<Expression>,
    },

    /// A number literal. This can be any integer, float, etc.
    NumberLiteral(f64),

    /// Creating a reference to a variable.
    Reference(Box<Expression>),

    /// A string literal.
    StringLiteral(String),

    /// A reference to a field on a structure type.
    StructureFieldReference {
        /// The expression providing the structure value.
        target: Box<Expression>,

        /// The index of the field being accessed,
        field_index: usize,
    },

    /// An initialization of a structure typed value.
    StructureInitialization {
        /// An ordered [`Vec`] of field values, one for each field within the structure.
        field_values: Vec<Expression>,
    },

    /// A reference to a local variable by name.
    VariableReference(String),
}

#[derive(Debug)]
pub enum FunctionCallTarget {
    /// A callee which is a "function" is free-standing, and is not owned by a type.
    Function { namespace: Option<String>, name: String },

    /// A callee which is a "method" is owned by a type, and the `receiver` must be passed as the first argument.
    Method { receiver: Expression, name: String },

    /// A callee which is "associated" is owned by a type, but does not have a receiver (i.e. a static method call).
    Associated { type_id: TypeId, name: String },
}

impl FunctionCallTarget {
    /// Returns the `name` of this [`FunctionCallTarget`]. The name may not _fully_ describe the functionc all, as it
    /// may be missing information like associated types.
    pub fn plain_name(&self) -> &str {
        match self {
            FunctionCallTarget::Associated { name, .. }
            | FunctionCallTarget::Function { name, .. }
            | FunctionCallTarget::Method { name, .. } => name,
        }
    }
}
