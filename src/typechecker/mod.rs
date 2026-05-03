use std::collections::HashMap;

use crate::{
    ast::type_expr::TypeExpr,
    core::span::Span,
    module::{
        CheckedModule,
        ParsedModule,
    },
    typechecker::{
        context::{
            DeclaredStructure,
            StructureId,
            TypecheckerContext,
        },
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
    pub fn check(
        &mut self,
        modules: Vec<ParsedModule>,
    ) -> Result<(Vec<CheckedModule>, HashMap<StructureId, DeclaredStructure>), TypecheckerError> {
        let mut modules = modules;

        DeclarationPass::new(self).run(&mut modules)?;
        BodyPass::new(self).run(&mut modules)?;

        Ok((modules.into_iter().map(|it| CheckedModule::new(it.id, it.ast)).collect(), self.context.structures.clone()))
    }

    /// Resolves the provided [`TypeExpr`] and registers it as a declared type in this [`Typechecker`]'s context.
    fn resolve_and_declare_type_from_expr(
        &mut self,
        name: String,
        expr: &mut TypeExpr,
        span: Span,
    ) -> Result<(), TypecheckerError> {
        // TODO: If a type already exists with the same name, then we must not declare another.

        // If this is a structure type, then we must assign a structure ID for it.
        let resolved_type = match expr {
            TypeExpr::Structure { fields } => {
                // We must ensure that the fields have a resolvable type.
                for field in fields.iter_mut() {
                    field.r#type = self.resolve_type_from_expr(&field.type_expr, field.span)?;
                }

                // Then, we can register the struct.
                let structure_id = self.context.insert_declared_structure(name.clone(), fields.clone(), span)?;
                Type::Structure(structure_id)
            }

            _ => self.resolve_type_from_expr(expr, span)?,
        };

        // We have resolved the type, we can insert it into the type declarations.
        self.context.insert_declared_type(name, resolved_type, span)?;
        Ok(())
    }

    /// Attempts to resolve the provided [`TypeExpr`] into a [`Type`].
    fn resolve_type_from_expr(&mut self, expr: &TypeExpr, span: Span) -> Result<Type, TypecheckerError> {
        let name = match expr {
            TypeExpr::Named(value) => value,

            TypeExpr::Reference(referenced_expr) => {
                // This is referencing another type, we can construct the [`Type`] by resolving the referenced type.
                let referenced = self.resolve_type_from_expr(referenced_expr, span)?;
                return Ok(Type::Reference(referenced.into()));
            }

            TypeExpr::Structure { .. } => panic!(),
        };

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

            // The built in types do not match, we can try to check for any user-defined types.
            _ => self
                .context
                .get_declared_type_by_name(name, span)
                .map(|it| it.r#type.clone())
                .ok_or(TypecheckerErrorKind::UnknownType(name.clone()).at(span))?,
        };

        Ok(r#type)
    }
}
