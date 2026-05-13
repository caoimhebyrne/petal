use std::collections::{
    HashMap,
    HashSet,
};

use crate::{
    ast::type_expr::TypeExpr,
    core::span::Span,
    module::{
        CheckedModule,
        ParsedModule,
    },
    typechecker::{
        context::{
            CheckedFunction,
            DeclaredStructure,
            FunctionId,
            IncompleteBuiltinTypes,
            StructureId,
            SyntheticType,
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

/// The result of a typechecker's execution.
pub struct CheckedProgram {
    /// The built-in types that have been recognized during compilation.
    pub builtin_types: BuiltinTypes,

    /// The functions defined in the source code during compilation.
    pub functions: HashMap<FunctionId, CheckedFunction>,

    /// The modules that have been checked.
    pub modules: Vec<CheckedModule>,

    /// The structures defined in the source code during compilation.
    pub structures: HashMap<StructureId, DeclaredStructure>,

    /// The types that have been synthesised during compilation.
    ///
    /// This could include: optional type implementations and generic type implementations.
    pub synthetic_types: HashSet<SyntheticType>,
}

/// The built-in types resolved during the typechecking process.
pub struct BuiltinTypes {
    /// The `str` type.
    pub compile_time_str: StructureId,
}

impl BuiltinTypes {
    fn from(incomplete: IncompleteBuiltinTypes) -> Result<Self, TypecheckerError> {
        Ok(Self {
            compile_time_str: incomplete
                .compile_time_str
                .ok_or(TypecheckerErrorKind::MissingBuiltinType("compile_time_str".to_string()).without_span())?,
        })
    }
}

/// The typechecker.
///
/// This is responsible for resolving and validating the types within a [`ParsedModule`].
#[derive(Default)]
pub(crate) struct Typechecker {
    context: TypecheckerContext,
}

impl Typechecker {
    /// Checks and resolved any [`Type`]s referenced in the provided [`ParsedModule`].
    pub fn check(mut self, modules: Vec<ParsedModule>) -> Result<CheckedProgram, TypecheckerError> {
        let mut modules = modules;

        DeclarationPass::new(&mut self).run(&mut modules)?;
        BodyPass::new(&mut self).run(&mut modules)?;

        let checked_program = CheckedProgram {
            builtin_types: BuiltinTypes::from(self.context.builtin_types)?,
            functions: self.context.functions,
            modules: modules.into_iter().map(|it| CheckedModule::new(it.id, it.ast)).collect(),
            structures: self.context.structures,
            synthetic_types: self.context.synthetic_types,
        };

        Ok(checked_program)
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

            TypeExpr::Optional(inner_expr) => {
                // This is wrapping another type, we can construct the [`Type`] by resolving the referenced type.
                let inner = self.resolve_type_from_expr(inner_expr, span)?;

                // FIXME: This is temporary.
                self.context.insert_synthetic_type(SyntheticType::Optional { inner_type: inner.clone() });

                return Ok(Type::Optional(inner.into()));
            }

            TypeExpr::Structure { .. } => {
                return Err(TypecheckerErrorKind::UnableToResolveType("Unexpected raw structure type?".into()).at(span));
            }
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

            "str" => {
                // `string::CompileTimeStr` should resolve to this.
                self.resolve_type_from_expr(&TypeExpr::named("CompileTimeStr"), span)?
            }

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
