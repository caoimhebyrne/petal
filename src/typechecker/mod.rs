use std::collections::{
    HashMap,
    HashSet,
};

use crate::{
    ast::type_expr::{
        GenericTypeArgument,
        GenericTypeParameter,
        StructureField,
        TypeExpr,
    },
    core::span::Span,
    module::{
        CheckedModule,
        ParsedModule,
    },
    typechecker::{
        context::{
            CheckedFunction,
            DeclaredStructure,
            DeclaredType,
            DeclaredTypeId,
            FunctionId,
            IncompleteBuiltinTypes,
            SpecializedStructure,
            SpecializedStructureId,
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
        r#type::{
            StructureReference,
            Type,
        },
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

    /// The types that have been declared during compilation.
    pub declared_types: HashMap<DeclaredTypeId, DeclaredType>,

    /// The functions defined in the source code during compilation.
    pub functions: HashMap<FunctionId, CheckedFunction>,

    /// The modules that have been checked.
    pub modules: Vec<CheckedModule>,

    /// The structures defined in the source code during compilation.
    pub structures: HashMap<StructureId, DeclaredStructure>,

    /// The specialized structures defined in the source code during compilation.
    pub specialized_structures: HashMap<SpecializedStructureId, SpecializedStructure>,

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

/// A context passed to [resolve_type_from_expr]
#[derive(Copy, Clone)]
pub(crate) struct TypeResolvingContext<'a> {
    /// The generic type arguments that are available in this context.
    pub generic_type_parameters: &'a Vec<GenericTypeParameter>,
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
            declared_types: self.context.types,
            functions: self.context.functions,
            modules: modules.into_iter().map(|it| CheckedModule::new(it.id, it.ast)).collect(),
            structures: self.context.structures,
            specialized_structures: self.context.specialized_structures,
            synthetic_types: self.context.synthetic_types,
        };

        Ok(checked_program)
    }

    /// Attempts to resolve the provided [`TypeExpr`] into a [`Type`].
    fn resolve_type_from_expr<'a>(
        &mut self,
        expr: &mut TypeExpr,
        context: TypeResolvingContext<'a>,
        span: Span,
    ) -> Result<Type, TypecheckerError> {
        match expr {
            TypeExpr::Named { name, generic_type_arguments } => {
                for argument in generic_type_arguments.iter_mut() {
                    argument.r#type = self.resolve_type_from_expr(&mut argument.type_expr, context, argument.span)?;
                }

                if !generic_type_arguments.is_empty() {
                    self.resolve_generic_type_from_expr(name, generic_type_arguments.clone(), span)
                } else if let Some(index) = context.generic_type_parameters.iter().position(|it| &it.name == name) {
                    // If we come across any generic types, then we can fill them in as `GenericType`, they will be
                    // replaced with their specialised types later.
                    Ok(Type::GenericType(index))
                } else {
                    self.resolve_type_by_name(name, span)
                }
            }

            TypeExpr::Reference(referenced_expr) => {
                // This is referencing another type, we can construct the [`Type`] by resolving the referenced type.
                let referenced = self.resolve_type_from_expr(referenced_expr, context, span)?;
                return Ok(Type::Reference(referenced.into()));
            }

            TypeExpr::Optional(inner_expr) => {
                // This is wrapping another type, we can construct the [`Type`] by resolving the referenced type.
                let inner = self.resolve_type_from_expr(inner_expr, context, span)?;

                // FIXME: This is temporary.
                self.context.insert_synthetic_type(SyntheticType::Optional { inner_type: inner.clone() });

                return Ok(Type::Optional(inner.into()));
            }

            TypeExpr::Structure { .. } => {
                return Err(TypecheckerErrorKind::UnableToResolveType("Unexpected raw structure type?".into()).at(span));
            }
        }
    }

    /// Attempts to create a (or re-use an existing) specialization of a generic type.
    fn resolve_generic_type_from_expr(
        &mut self,
        name: &str,
        mut generic_type_arguments: Vec<GenericTypeArgument>,
        span: Span,
    ) -> Result<Type, TypecheckerError> {
        // A user-declared type must exist with the provided name. If not, then the type either does not exist, or it
        // doesn't support generic type parameters.
        let declared_type = self
            .context
            .get_declared_type_by_name(name, span)
            .ok_or(TypecheckerErrorKind::UnknownOrUnsupportedGenericType(name.to_string()).at(span))?
            .clone();

        // The number of generic parameters on the declared type must equal the number of generic arguments passed.
        if declared_type.generic_type_parameters.len() != generic_type_arguments.len() {
            return Err(TypecheckerErrorKind::GenericArgumentSizeMismatch {
                type_name: declared_type.name.clone(),
                parameters: declared_type.generic_type_parameters.len(),
                arguments: generic_type_arguments.len(),
            }
            .at(span));
        }

        // The declared type must correspond to a structure.
        let Type::Structure(StructureReference::Plain(structure_id)) = declared_type.r#type else {
            return Err(TypecheckerErrorKind::UnknownOrUnsupportedGenericType(name.to_string()).at(span));
        };

        // We can then apply the generic type arguments to the structures fields, and create a specialized variant of
        // the structure.
        let generic_structure_fields = self.context.structures[&structure_id].fields.clone();
        let mut specialized_fields: Vec<StructureField> = Vec::new();

        for mut field in generic_structure_fields {
            // If any of the fields have a generic type, we can substitute it for the argument provided.
            let Type::GenericType(generic_type_index) = field.r#type else {
                specialized_fields.push(field);
                continue;
            };

            // We can then resolve the generic type to its argument type.
            let argument_type = self.resolve_type_from_expr(
                &mut generic_type_arguments[generic_type_index].type_expr,
                TypeResolvingContext { generic_type_parameters: &declared_type.generic_type_parameters },
                span,
            )?;

            field.r#type = argument_type;
            specialized_fields.push(field);
        }

        let id =
            self.context.insert_specialized_structure(declared_type.id, generic_type_arguments, specialized_fields);

        Ok(Type::Structure(StructureReference::Specialized(id)))
    }

    /// Attempts to resolve the provided [`name`] into a [`Type`].
    fn resolve_type_by_name(&self, name: &str, span: Span) -> Result<Type, TypecheckerError> {
        let r#type = match name {
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

            // `prelude::CompileTimeStr` should resolve to this.
            // TODO: Namespaced types
            "str" => self.resolve_type_by_name("CompileTimeStr", span)?,

            // The built in types do not match, we can try to check for any user-defined types.
            _ => self
                .context
                .get_declared_type_by_name(name, span)
                .map(|it| it.r#type.clone())
                .ok_or(TypecheckerErrorKind::UnknownType(name.to_string()).at(span))?,
        };

        Ok(r#type)
    }

    /// Attempts to get the [`DeclaredType`] associated with a [`StructureReference`].
    fn get_declared_type_for_structure_ref(&self, reference: &StructureReference) -> &DeclaredType {
        match reference {
            StructureReference::Plain(plain_id) => {
                let structure = &self.context.structures[plain_id];
                &self.context.types[&structure.declared_type_id]
            }

            StructureReference::Specialized(specialized_id) => {
                let structure = &self.context.specialized_structures[specialized_id];
                &self.context.types[&structure.generic_type_id]
            }
        }
    }

    /// Attempts to get a [`Vec`] of [`StructureField`]s for the provided [`StructureReference`].
    fn get_structure_fields(&self, reference: &StructureReference) -> &Vec<StructureField> {
        match reference {
            StructureReference::Plain(plain_id) => &self.context.structures[plain_id].fields,
            StructureReference::Specialized(specialized_id) => {
                &self.context.specialized_structures[specialized_id].fields
            }
        }
    }
}
