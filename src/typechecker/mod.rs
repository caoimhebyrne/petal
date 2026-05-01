use crate::{
    ast::type_expr::TypeExpr,
    core::span::Span,
    module::{
        CheckedModule,
        ParsedModule,
    },
    typechecker::{
        context::TypecheckerContext,
        error::{
            TypecheckerError,
            TypecheckerErrorKind,
        },
        pass::{
            body::BodyPass,
            declaration::DeclarationPass,
        },
        r#type::Type,
    },
};

pub(crate) mod context;
pub(crate) mod error;
pub(crate) mod pass;
pub mod r#type;

/// The typechecker.
///
/// This is responsible for resolving and validating the types within a [`ParsedModule`].
#[derive(Default)]
pub struct Typechecker {
    context: TypecheckerContext,
}

impl Typechecker {
    /// Checks and resolved any [`Type`]s referenced in the provided [`ParsedModule`].
    pub fn check(&mut self, modules: Vec<ParsedModule>) -> Result<Vec<CheckedModule>, TypecheckerError> {
        let mut modules = modules;

        DeclarationPass::new(self).run(&mut modules)?;
        BodyPass::new(self).run(&mut modules)?;

        Ok(modules.into_iter().map(|it| CheckedModule::new(it.id, it.ast)).collect())
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
            "void" => Type::Void,

            _ => return Err(TypecheckerErrorKind::UnknownType(name.clone()).at(span)),
        };

        Ok(r#type)
    }
}
