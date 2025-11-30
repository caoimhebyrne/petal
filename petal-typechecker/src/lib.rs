use petal_ast::{
    expression::{ExpressionNode, ExpressionNodeKind},
    statement::{StatementNode, StatementNodeKind, TopLevelStatementNode, TopLevelStatementNodeKind},
};
use petal_core::{
    error::Result,
    string_intern::StringInternPool,
    r#type::{ResolvedType, Type, TypeReference, pool::TypePool},
};

use crate::{
    context::TypecheckerContext, error::TypecheckerError, temp_resolved_module::ResolvedModule, typecheck::Typecheck,
};

pub(crate) mod context;
pub(crate) mod error;
pub mod temp_resolved_module;
pub(crate) mod typecheck;

/// The Petal typechecker is not only a typechecker, it also ensures that all types are resolvable and are understood
/// at compile time.
///
/// To typecheck an AST, call [petal_ast::StatementStream::visit] with this [Typechecker].
pub struct Typechecker<'a> {
    /// The [TypecheckerContext] containing extra information about the current type-checking session.
    pub(crate) context: TypecheckerContext<'a>,

    /// The [TypePool] to read types from.
    pub(crate) type_pool: &'a mut TypePool,

    /// The [StringInternPool] to read strings from.
    pub(crate) string_intern_pool: &'a dyn StringInternPool,
}

impl<'a> Typechecker<'a> {
    /// Creates a new [Typechecker].
    pub fn new(type_pool: &'a mut TypePool, string_intern_pool: &'a dyn StringInternPool) -> Self {
        Typechecker {
            context: TypecheckerContext::new(string_intern_pool),
            type_pool,
            string_intern_pool,
        }
    }

    /// Checks and resolves all types in the provided [ResolvedModule]s.
    pub fn check_modules(&mut self, modules: &mut Vec<ResolvedModule>) -> Result<()> {
        for module in modules {
            for statement in &mut module.statements {
                self.check_top_level_statement(statement)?;
            }
        }

        Ok(())
    }

    /// Checks and resolves all types involved in a [TopLevelStatementNode].
    pub fn check_top_level_statement(&mut self, statement: &mut TopLevelStatementNode) -> Result<()> {
        match &mut statement.kind {
            TopLevelStatementNodeKind::FunctionDeclaration(function) => function.typecheck(self, None, statement.span),
            TopLevelStatementNodeKind::Import(_) => return Ok(()),

            #[allow(unreachable_patterns)]
            _ => return TypecheckerError::unsupported_top_level_statement(statement.clone()).into(),
        }?;

        Ok(())
    }

    /// Checks and resolves all types involved in a [StatementNode].
    pub fn check_statement(&mut self, statement: &mut StatementNode) -> Result<()> {
        match &mut statement.kind {
            StatementNodeKind::Return(r#return) => r#return.typecheck(self, None, statement.span),
            StatementNodeKind::VariableDeclaration(variable) => variable.typecheck(self, None, statement.span),
            StatementNodeKind::FunctionCall(function_call) => function_call.typecheck(self, None, statement.span),
            StatementNodeKind::VariableAssignment(assignment) => assignment.typecheck(self, None, statement.span),

            #[allow(unreachable_patterns)]
            _ => TypecheckerError::unsupported_statement(statement.clone()).into(),
        }?;

        Ok(())
    }

    /// Checks and resolves all types involved in an [Expression]. This also sets the [Expression::type] to the resolved
    /// type.
    ///
    /// Returns:
    /// The [Type] that this expression produces.
    pub fn check_expression(
        &mut self,
        expression: &mut ExpressionNode,
        expected_type: Option<&ResolvedType>,
    ) -> Result<ResolvedType> {
        let r#type = match &mut expression.kind {
            ExpressionNodeKind::IntegerLiteral { .. } => {
                // If the expected type is an integer type, then we can smart-cast this literal to that type. Otherwise,
                // we can just assume that it is an i32.
                match expected_type {
                    Some(ResolvedType::SignedInteger(size)) => ResolvedType::SignedInteger(*size),
                    Some(ResolvedType::UnsignedInteger(size)) => ResolvedType::UnsignedInteger(*size),
                    _ => ResolvedType::SignedInteger(32),
                }
            }

            ExpressionNodeKind::IdentifierReference { identifier } => self
                .context
                .function_context(expression.span)?
                .get_variable(&identifier, expression.span)?
                .r#type
                .clone(),

            ExpressionNodeKind::BinaryOperation(binary_operation) => {
                binary_operation.typecheck(self, expected_type, expression.span)?
            }

            ExpressionNodeKind::FunctionCall(function_call) => function_call.typecheck(self, None, expression.span)?,

            ExpressionNodeKind::Reference(reference) => {
                // The value of the inner expression must be resolvable.
                let inner_type = self.check_expression(&mut reference.value, expected_type)?;
                ResolvedType::Reference(self.type_pool.allocate(Type::Resolved(inner_type)))
            }

            ExpressionNodeKind::StringLiteral { .. } => ResolvedType::Reference(
                self.type_pool
                    .allocate(Type::Resolved(ResolvedType::UnsignedInteger(8))),
            ),

            #[allow(unreachable_patterns)]
            _ => return TypecheckerError::unsupported_expression(expression.clone()).into(),
        };

        let type_id = self.type_pool.allocate(Type::Resolved(r#type.clone()));
        expression.r#type = Some(TypeReference::new(type_id, expression.span));

        Ok(r#type)
    }

    /// Attempts to resolve the provided [Type] if it has not been resolved already.
    pub fn resolve_type(&mut self, reference: &TypeReference) -> Result<ResolvedType> {
        let r#type = self.type_pool.get_type_mut_or_err(&reference.id, reference.span)?;

        // If the provided type has been resolved already, then we don't need to do anything else.
        let type_name_reference = match r#type {
            Type::Unresolved(reference) => reference,

            Type::Resolved(r#type) => {
                let resolved_type = r#type.clone();

                if let ResolvedType::Reference(referenced_type_id) = resolved_type {
                    self.resolve_type(&TypeReference::new(referenced_type_id, reference.span))?;
                }

                return Ok(resolved_type);
            }
        };

        // Otherwise, we can attempt to resolve the type from its name.
        let type_name = self
            .string_intern_pool
            .resolve_reference_or_err(&type_name_reference, reference.span)?;

        let resolved_kind = match type_name {
            "i8" => ResolvedType::SignedInteger(8),
            "i16" => ResolvedType::SignedInteger(16),
            "i32" => ResolvedType::SignedInteger(32),

            "u8" => ResolvedType::UnsignedInteger(8),
            "u16" => ResolvedType::UnsignedInteger(16),
            "u32" => ResolvedType::UnsignedInteger(32),

            "void" => ResolvedType::Void,

            _ => {
                // Otherwise, we can attempt to look it up in the current context.
                self.context
                    .get_type_declaration(&type_name_reference, reference.span)?
                    .clone()
            }
        };

        // We can then set the type to the resolved type.
        *r#type = Type::Resolved(resolved_kind.clone());
        Ok(resolved_kind)
    }
}
