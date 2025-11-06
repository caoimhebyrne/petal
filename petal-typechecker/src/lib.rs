use petal_ast::{
    expression::{Expression, ExpressionKind},
    statement::{Statement, StatementKind},
    visitor::ASTVisitor,
};
use petal_core::{
    error::Result,
    string_intern::StringInternPool,
    r#type::{ResolvedType, Type, TypeReference, pool::TypePool},
};

use crate::{context::TypecheckerContext, error::TypecheckerError, typecheck::Typecheck};

pub(crate) mod context;
pub(crate) mod error;
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

    /// Checks and resolves all types involved in a [Statement].
    ///
    /// Returns:
    /// The [Type] that this statement produces. If the statement does not result in a type, then [Type::void] will be
    /// returned.
    pub fn check_statement(&mut self, statement: &mut Statement) -> Result<Type> {
        match &mut statement.kind {
            StatementKind::FunctionDeclaration(declaration) => declaration.typecheck(self, statement.span),
            StatementKind::ReturnStatement(r#return) => r#return.typecheck(self, statement.span),
            StatementKind::FunctionCall(function_call) => function_call.typecheck(self, statement.span),
            StatementKind::VariableDeclaration(declaration) => declaration.typecheck(self, statement.span),
            StatementKind::VariableAssignment(assignment) => assignment.typecheck(self, statement.span),

            #[allow(unreachable_patterns)]
            _ => TypecheckerError::unsupported_statement(statement.clone()).into(),
        }
    }

    /// Checks and resolves all types involved in an [Expression]. This also sets the [Expression::type] to the resolved
    /// type.
    ///
    /// Returns:
    /// The [Type] that this expression produces.
    pub fn check_expression(&mut self, expression: &mut Expression) -> Result<Type> {
        let r#type = match &mut expression.kind {
            ExpressionKind::FunctionCall(function_call) => function_call.typecheck(self, expression.span)?,
            ExpressionKind::BinaryOperation(operation) => operation.typecheck(self, expression.span)?,

            ExpressionKind::IdentifierReference(reference) => {
                // A variable must've been declared with the provided name already.
                let variable = self
                    .context
                    .function_context(expression.span)?
                    .get_variable(reference, expression.span)?;

                variable.r#type
            }

            // TODO: When multiple integer widths are supported, we will need to add inference by passing an "expected"
            // type to `check_expression`.
            ExpressionKind::IntegerLiteral(_) => Type::Resolved(ResolvedType::Integer(32)),

            #[allow(unreachable_patterns)]
            _ => return TypecheckerError::unsupported_expression(expression.clone()).into(),
        };

        // All expressions have an associated 'type' field which should always be present after typechecking.
        expression.r#type = Some(r#type);
        Ok(r#type)
    }

    /// Attempts to resolve the provided [Type] if it has not been resolved already.
    pub fn resolve_type(&self, reference: TypeReference) -> Result<Type> {
        let r#type = self.type_pool.get_type_mut_or_err(&reference.id, reference.span)?;

        // If the provided type has been resolved already, then we don't need to do anything else.
        let type_name_reference = match r#type {
            Type::Unresolved(reference) => reference,
            Type::Resolved(_) => return Ok(*r#type),
            _ => panic!(),
        };

        // Otherwise, we can attempt to resolve the type from its name.
        let type_name = self
            .string_intern_pool
            .resolve_reference_or_err(&type_name_reference, reference.span)?;

        let resolved_kind = match type_name {
            "i32" => ResolvedType::Integer(32),
            "void" => ResolvedType::Void,

            _ => return TypecheckerError::unable_to_resolve_type(r#type).into(),
        };

        // We can then set the type to the resolved type.
        *r#type = Type::Resolved(resolved_kind);
        Ok(*r#type)
    }
}

impl<'a> ASTVisitor for Typechecker<'a> {
    fn visit(&mut self, statement: &mut Statement) -> Result<()> {
        self.check_statement(statement).map(|_| ())
    }
}
