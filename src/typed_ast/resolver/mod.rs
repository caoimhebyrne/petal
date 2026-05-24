use std::cmp::max;

use crate::{
    ast::{
        self,
        expression::binary_operation::BinaryOperatorClass,
        type_expr::TypeExpr,
    },
    core::span::Span,
    module::ParsedModule,
    typed_ast::{
        Expression,
        ExpressionKind,
        Function,
        FunctionCallTarget,
        FunctionKey,
        FunctionParameter,
        GenericInformation,
        GenericTypeParameter,
        Program,
        Statement,
        StatementKind,
        error::{
            TypecheckerError,
            TypecheckerErrorKind,
        },
        resolver::{
            context::{
                TypeResolverContext,
                UnresolvedFunctionDeclaration,
            },
            scope::Scope,
        },
        r#type::{
            Type,
            db::{
                DefinedTypeId,
                TypeId,
            },
            defined::{
                DefinedType,
                DefinedTypeKind,
                Structure,
                StructureField,
            },
        },
    },
};

mod context;
mod scope;

type TypecheckerResult<T> = Result<T, TypecheckerError>;

/// Responsible for creating an initial [`Program`], which contains some basic type information.
///
/// The types within this program may not be fully resolved yet, and later passes should attempt to resolve them if at
/// all possible.
#[derive(Default)]
pub struct TypeResolver {
    /// The context to use while resolving types.
    ///
    /// This contains information about generic functions, types, etc.
    context: TypeResolverContext,

    /// The current scope.
    scope: Scope,

    /// The namespace that this [`TypeResolver`] is currently visiting.
    namespace: Option<String>,

    /// The program being constructed by this [`TypeResolver`].
    program: Program,
}

impl TypeResolver {
    /// Attempts to resolve any basic types within the provided [`Vec`] of [`ParsedModule`]s.
    pub fn resolve(mut self, modules: Vec<ParsedModule>) -> TypecheckerResult<Program> {
        // We first need to note all of the functions that exist in the module.
        for module in &modules {
            self.pre_visit_top_level_declaration_statements(&module.ast)?;
        }

        // Then, we can use that function information to visit their bodies and attempt to compile them.
        for module in modules {
            self.visit_top_level_declaration_statements(module.ast)?;
        }

        Ok(self.program)
    }

    /// Registers any top-level declarations in the [`TypeResolverContext`] to be used when visiting their body later.
    fn pre_visit_top_level_declaration_statements(
        &mut self,
        statements: &Vec<ast::statement::Statement>,
    ) -> TypecheckerResult<()> {
        for statement in statements {
            match &statement.kind {
                ast::statement::StatementKind::NamespaceDeclaration(namespace_declaration) => {
                    self.namespace = Some(namespace_declaration.name.clone());
                    self.pre_visit_top_level_declaration_statements(&namespace_declaration.body)?;
                    self.namespace = None; // todo: nesting?
                }

                ast::statement::StatementKind::FunctionDeclaration(function_declaration) => {
                    self.pre_visit_function_declaration(function_declaration);
                }

                ast::statement::StatementKind::TypeDeclaration(type_declaration) => {
                    self.pre_visit_type_declaration(type_declaration);
                }

                ast::statement::StatementKind::Import(_) => {}

                _ => {
                    panic!(
                        "Unsupported top-level statement ({:?}) at source index {}",
                        statement.kind, statement.span.location.start
                    );
                }
            }
        }

        Ok(())
    }

    /// Visits any valid top-level declarations in the provided [`Vec`] of [`ast::statement::Statement`]s.
    /// Any unsupported statements will cause a warning to be logged, and they will be ignored.
    fn visit_top_level_declaration_statements(
        &mut self,
        statements: Vec<ast::statement::Statement>,
    ) -> TypecheckerResult<()> {
        for statement in statements {
            match statement.kind {
                ast::statement::StatementKind::NamespaceDeclaration(namespace_declaration) => {
                    self.namespace = Some(namespace_declaration.name.clone());
                    self.visit_top_level_declaration_statements(namespace_declaration.body)?;
                    self.namespace = None; // todo: nesting?
                }

                ast::statement::StatementKind::FunctionDeclaration(function_declaration) => {
                    self.visit_function_declaration(function_declaration, statement.span)?;
                }

                ast::statement::StatementKind::TypeDeclaration(type_declaration) => {
                    self.visit_type_declaration(type_declaration, statement.span)?;
                }

                ast::statement::StatementKind::Import(_) => {}

                _ => {
                    panic!(
                        "Unsupported top-level statement ({:?}) at source index {}",
                        statement.kind, statement.span.location.start
                    );
                }
            }
        }

        Ok(())
    }
}

impl TypeResolver {
    /// Sets the [`Scope`] to the one returned by the provided [`supplier`]. The [`supplier`] will be called with
    /// ownership of the current [`Scope`].
    fn set_scope<S>(&mut self, supplier: S)
    where
        S: FnOnce(Scope) -> Scope,
    {
        // todo(threading): Is this thread safe?
        let current_scope = std::mem::take(&mut self.scope);
        self.scope = supplier(current_scope);
    }

    /// Sets the [`Scope`] to the parent of the current [`Scope`].
    /// This function will panic if [`Scope::parent`] is [`None`].
    fn pop_scope(&mut self) {
        self.set_scope(|current| *current.parent.expect("TypeResolver::scope should have a parent"));
    }
}

impl TypeResolver {
    /// Visits the provided [`TypeExpr`], resolving it into a [`TypeId`].
    fn visit_type_expr(
        &mut self,
        generic_type_parameters: &[GenericTypeParameter],
        expr: &TypeExpr,
        span: Span,
    ) -> TypecheckerResult<TypeId> {
        match expr {
            TypeExpr::Named { name, generic_type_arguments } => {
                // If the type corresponds with a generic type parameter available in this scope, then it should take
                // precedence over all other types.
                if let Some(generic_type_parameter) = generic_type_parameters.iter().find(|it| &it.name == name) {
                    return Ok(generic_type_parameter.type_id);
                }

                let generic_type_arguments = generic_type_arguments
                    .iter()
                    .map(|it| self.visit_type_expr(generic_type_parameters, &it.type_expr, it.span))
                    .collect::<TypecheckerResult<Vec<TypeId>>>()?;

                self.resolve_type_by_name(&generic_type_arguments, name, span)
            }

            TypeExpr::Reference(inner_type_expr) => {
                let inner_type_id = self.visit_type_expr(generic_type_parameters, inner_type_expr, span)?;
                let ty = Type::Reference(inner_type_id);
                Ok(self.program.type_db.get_or_insert_type(ty))
            }

            _ => todo!(),
        }
    }

    /// Attempts to resolve a type by the provided plain name, resolving it into a [`Ty`].
    fn resolve_type_by_name(
        &mut self,
        generic_type_arguments: &[TypeId],
        name: &str,
        span: Span,
    ) -> TypecheckerResult<TypeId> {
        let ty = match name {
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

            // FIXME: "str" is an alias for "CompileTimeStr" from the prelude.
            "str" => return self.compute_defined_type("CompileTimeStr", &[], span),

            _ => return self.compute_defined_type(name, generic_type_arguments, span),
        };

        Ok(self.program.type_db.get_or_insert_type(ty))
    }

    /// Attempts to find a defined type given its name and generic information.
    fn compute_defined_type(
        &mut self,
        name: &str,
        generic_type_arguments: &[TypeId],
        span: Span,
    ) -> TypecheckerResult<TypeId> {
        if let Some(defined_type_id) = self.program.type_db.find_defined_type(name, generic_type_arguments) {
            let type_id = self.program.type_db.get_or_insert_type(Type::Defined(defined_type_id));
            return Ok(type_id);
        }

        // If there is no defined type, then we must insert one. This could be a generic type, or it coudl be a type
        // that was declared after this one in the source code.
        let Some(type_declaration) = self.context.find_type_declaration(name).cloned() else {
            return Err(TypecheckerErrorKind::UndeclaredTypeName(name.to_string()).at(span));
        };

        // The number of generic type arguments provided must equal the number of parameters on the type.
        if type_declaration.generic_type_parameters.len() != generic_type_arguments.len() {
            return Err(TypecheckerErrorKind::GenericTypeArgumentCountMismatch {
                expected: type_declaration.generic_type_parameters.len(),
                got: generic_type_arguments.len(),
            }
            .at(span));
        }

        // todo(resolver): `TypeResolvingContext`
        let generic_type_parameters = type_declaration
            .generic_type_parameters
            .iter()
            .zip(generic_type_arguments)
            .map(|(parameter, argument_type_id)| GenericTypeParameter {
                name: parameter.name.clone(),
                type_id: *argument_type_id,
            })
            .collect::<Vec<GenericTypeParameter>>();

        let defined_type_id = self.compile_type_declaration(type_declaration, &generic_type_parameters, span)?;

        Ok(self.program.type_db.get_or_insert_type(Type::Defined(defined_type_id)))
    }
}

impl TypeResolver {
    /// Attempts to find a function given its name, parameters, and expected return type.
    /// If one does not exist, a function will be cloned from the context and compiled via [`compile_function`].
    fn compute_function(
        &mut self,
        target: &FunctionCallTarget,
        generic_type_arguments: &[TypeId],
        span: Span,
    ) -> TypecheckerResult<FunctionKey> {
        // If a function exists that satisfies our restrictions, then we can use it.
        if let Some(tuple) = self.program.find_function(target, generic_type_arguments) {
            return Ok(*tuple.0);
        }

        trace!(
            "Function call target '{target:?}' does not yet have a matching function declaration, checking for an unresolved one...",
        );

        // We can attempt to find an existing function declaration. This may or may not be generic.
        let mut candidates = self.context.find_function_declaration(target);
        if candidates.is_empty() {
            return Err(TypecheckerErrorKind::UndeclaredFunction(target.plain_name().to_string()).at(span));
        }

        if candidates.len() > 1 {
            candidates.retain(|it| {
                // If the target of the function call does not have an associated type ID, then we don't need to filter
                // the candidates any further.
                let receiver_type_id = match target {
                    FunctionCallTarget::Associated { type_id, .. } => type_id,
                    FunctionCallTarget::Function { .. } => return true,
                    FunctionCallTarget::Method { receiver, .. } => &receiver.type_id,
                };

                let Some(owner_type_expr) = it.declaration.owner_type_expr.as_ref() else { return false };

                // TODO: What should we do if an error occurs here?
                let owner_type_id = self.visit_type_expr(&[], owner_type_expr, span).expect("visit_type_expr");
                receiver_type_id == &owner_type_id
            });
        }

        // If there are still multiple candidates, then we must return an error: this function call is ambiguous and
        // we cannot narrow it down any more.
        if candidates.len() > 1 {
            return Err(TypecheckerErrorKind::AmbiguousFunctionCall(candidates.len()).at(span));
        }

        let function = candidates
            .first()
            .ok_or_else(|| TypecheckerErrorKind::UndeclaredFunction(target.plain_name().to_string()).at(span))?;

        // The number of generic type arguments must equal the number of generic type parameters in the function. At a
        // later point in time, we may be able to infer these.
        if generic_type_arguments.len() != function.declaration.generic_type_parameters.len() {
            return Err(TypecheckerErrorKind::GenericTypeArgumentCountMismatch {
                expected: function.declaration.generic_type_parameters.len(),
                got: generic_type_arguments.len(),
            }
            .at(span));
        }

        // todo(resolver): `TypeResolvingContext`
        let generic_type_parameters = function
            .declaration
            .generic_type_parameters
            .iter()
            .zip(generic_type_arguments)
            .map(|(parameter, argument_type_id)| GenericTypeParameter {
                name: parameter.name.clone(),
                type_id: *argument_type_id,
            })
            .collect::<Vec<GenericTypeParameter>>();

        self.compile_function_declaration(
            function.namespace.clone(),
            function.declaration.clone(),
            &generic_type_parameters,
            span,
        )
    }
}

impl TypeResolver {
    /// Visits the provided [`ast::statement::function_declaration::FunctionDeclaration`], and inserts it into the
    /// [`TypeResolverContext`].
    ///
    /// This insertion will later be used when we visit the body of the function.
    fn pre_visit_function_declaration(
        &mut self,
        function_declaration: &ast::statement::function_declaration::FunctionDeclaration,
    ) {
        self.context.insert_function_declaration(UnresolvedFunctionDeclaration {
            namespace: self.namespace.clone(),
            declaration: function_declaration.clone(),
        });
    }

    /// Visits the provided [`ast::statement::function_declaration::FunctionDeclaration`].
    ///
    /// If the function has generic type parameters, it will not be appended to the [`Program`], and will instead be
    /// stored to undergo monomorphization once a call is made to it.
    ///
    /// If the function does not have generic type parameters, [`compile_function_declaration`] will be called.
    fn visit_function_declaration(
        &mut self,
        function_declaration: ast::statement::function_declaration::FunctionDeclaration,
        span: Span,
    ) -> TypecheckerResult<()> {
        if !function_declaration.generic_type_parameters.is_empty() {
            // This is a generic function, we don't want to generate code for it until someone calls it.
            return Ok(());
        }

        // The function might have already been compiled, so if one already exists in the program: we can exit.
        if self
            .program
            .find_function(
                &FunctionCallTarget::Function {
                    namespace: self.namespace.clone(),
                    name: function_declaration.name.clone(),
                },
                &[],
            )
            .is_some()
        {
            return Ok(());
        }

        // Otherwise, we can compile the function as normal.
        self.compile_function_declaration(self.namespace.clone(), function_declaration, &[], span)?;
        Ok(())
    }

    /// Compiles the provided [`ast::statement::function_declaration::FunctionDeclaration`].
    fn compile_function_declaration(
        &mut self,
        namespace: Option<String>,
        function_declaration: ast::statement::function_declaration::FunctionDeclaration,
        generic_type_parameters: &[GenericTypeParameter],
        span: Span,
    ) -> TypecheckerResult<FunctionKey> {
        let owner_type_id = function_declaration
            .owner_type_expr
            .map(|it| self.visit_type_expr(generic_type_parameters, &it, span))
            .transpose()?;

        // If the function has an owner type, its type becomes an implicit 'This' generic type parameter.
        let mut generic_type_parameters: Vec<GenericTypeParameter> = generic_type_parameters.into();
        if let Some(type_id) = owner_type_id {
            generic_type_parameters.insert(0, GenericTypeParameter { name: "This".to_string(), type_id });
        }

        let parameters = function_declaration
            .parameters
            .into_iter()
            .map(|it| self.visit_function_parameter(&generic_type_parameters, it))
            .collect::<TypecheckerResult<Vec<_>>>()?;

        let return_type_id = function_declaration
            .return_type_expr
            .map(|it| self.visit_type_expr(&generic_type_parameters, &it, span))
            .transpose()?
            .unwrap_or(self.program.type_db.void_type_id());

        self.set_scope(|current| {
            let parameter_tys = parameters.iter().map(|it| (it.name.clone(), it.type_id)).collect();
            Scope::function(generic_type_parameters.clone(), parameter_tys, Some(current), return_type_id)
        });

        let body = self.visit_statements(function_declaration.body)?;

        self.pop_scope();

        let function_key = self.program.insert_function(
            span.module_id,
            Function {
                modifiers: function_declaration.modifiers,
                namespace,
                name: function_declaration.name,
                parameters,
                body,
                return_type_id,
                owner_type_id,
                // If no generic type parameters were provided, then we should not attach any generic information.
                generic_information: if generic_type_parameters.is_empty() {
                    None
                } else {
                    Some(GenericInformation { parameters: generic_type_parameters })
                },
                span,
            },
        );

        Ok(function_key)
    }

    /// Visits the provided [`ast::statement::function_declaration::FunctionParameter`].
    /// The type declared by the parameter will be resolved, and transformed into a typed [`FunctionParameter`].
    fn visit_function_parameter(
        &mut self,
        generic_type_parameters: &[GenericTypeParameter],
        parameter: ast::statement::function_declaration::FunctionParameter,
    ) -> TypecheckerResult<FunctionParameter> {
        Ok(FunctionParameter {
            name: parameter.name,
            type_id: self.visit_type_expr(generic_type_parameters, &parameter.type_expr, parameter.span)?,
            is_named: parameter.is_named,
            span: parameter.span,
        })
    }
}

impl TypeResolver {
    /// Visits the provided reference to a [`ast::statement::type_declaration::TypeDeclaration`], registering it with
    /// the context to be compiled later.
    fn pre_visit_type_declaration(&mut self, type_declaration: &ast::statement::type_declaration::TypeDeclaration) {
        self.context.insert_type_declaration(type_declaration.clone());
    }

    /// Visits the provided [`ast::statement::type_declaration::TypeDeclaration`].
    fn visit_type_declaration(
        &mut self,
        type_declaration: ast::statement::type_declaration::TypeDeclaration,
        span: Span,
    ) -> TypecheckerResult<()> {
        if !type_declaration.generic_type_parameters.is_empty() {
            // The type is generic, we will not insert it directly into the program. Instead, it will be compiled
            // into the program via a generic type use.
            return Ok(());
        }

        // The type declaration might have already been compiled, so if one already exists in the program: we can exit.
        if self.program.type_db.find_defined_type(&type_declaration.name, &[]).is_some() {
            return Ok(());
        }

        self.compile_type_declaration(type_declaration, &[], span)?;
        Ok(())
    }

    /// Compiles the provided.
    fn compile_type_declaration(
        &mut self,
        type_declaration: ast::statement::type_declaration::TypeDeclaration,
        generic_type_parameters: &[GenericTypeParameter],
        span: Span,
    ) -> TypecheckerResult<DefinedTypeId> {
        let defined_type_kind =
            self.visit_type_expr_on_declaration(generic_type_parameters, type_declaration.type_expr, span)?;

        let defined_type_id = self.program.type_db.insert_defined_type(DefinedType {
            modifiers: type_declaration.modifiers,
            name: type_declaration.name,
            kind: defined_type_kind,
            // If no generic type parameters were provided, then we should not attach any generic information.
            generic_information: if generic_type_parameters.is_empty() {
                None
            } else {
                Some(GenericInformation { parameters: generic_type_parameters.into() })
            },
            span,
        });

        Ok(defined_type_id)
    }

    /// Visits a [`TypeExpr`] that is part of a type declaration.
    fn visit_type_expr_on_declaration(
        &mut self,
        generic_type_parameters: &[GenericTypeParameter],
        type_expr: TypeExpr,
        span: Span,
    ) -> TypecheckerResult<DefinedTypeKind> {
        let TypeExpr::Structure { fields } = type_expr else {
            return Err(TypecheckerErrorKind::ExpectedTypeDefinition.at(span));
        };

        let fields = fields
            .into_iter()
            .map(|it| {
                let type_id = self.visit_type_expr(generic_type_parameters, &it.type_expr, it.span)?;
                Ok(StructureField { name: it.name, span: it.span, type_id })
            })
            .collect::<TypecheckerResult<Vec<StructureField>>>()?;

        Ok(DefinedTypeKind::Structure(Structure { fields }))
    }
}

impl TypeResolver {
    /// Visits the provided [`Vec`] of AST [`Statement`]s.
    /// The returned [`Vec`] will be the typed variants of the statements.
    fn visit_statements(&mut self, statements: Vec<ast::statement::Statement>) -> TypecheckerResult<Vec<Statement>> {
        let mut vec: Vec<Statement> = Vec::new();

        for statement in statements {
            vec.push(self.visit_statement(statement)?);
        }

        Ok(vec)
    }

    /// Visits the provided AST [`Statement`]. The returned [`Statement`] will be the typed variant of it.
    fn visit_statement(&mut self, statement: ast::statement::Statement) -> TypecheckerResult<Statement> {
        let kind = match statement.kind {
            ast::statement::StatementKind::FunctionCall(function_call) => {
                let (function_key, arguments, return_type_id) =
                    self.visit_expression_function_call(function_call, statement.span)?;

                StatementKind::FunctionCall { function_key, arguments, return_type_id }
            }

            ast::statement::StatementKind::Return(r#return) => self.visit_statement_return(r#return, statement.span)?,

            ast::statement::StatementKind::VariableAssignment(variable_assignment) => {
                self.visit_statement_variable_assignment(variable_assignment, statement.span)?
            }

            ast::statement::StatementKind::VariableDeclaration(variable_declaration) => {
                self.visit_statement_variable_declaration(variable_declaration, statement.span)?
            }

            _ => todo!(),
        };

        Ok(kind.at(statement.span))
    }

    /// Visits the proivded AST [`Return`] statement.
    fn visit_statement_return(
        &mut self,
        r#return: ast::statement::r#return::Return,
        span: Span,
    ) -> TypecheckerResult<StatementKind> {
        // If the scope does not have an expected return type, we can assume that it is `void`.
        let return_type_id = self.scope.get_return_type_id().unwrap_or(self.program.type_db.void_type_id());

        let value = r#return.value.map(|it| self.visit_expression(it, Some(return_type_id))).transpose()?;

        // If the value does not exist, we can treat it as a 'void' value.
        let value_type_id = value.as_ref().map_or(self.program.type_db.void_type_id(), |it| it.type_id);

        // The return type of the scope must match the value type.
        if return_type_id != value_type_id {
            return Err(
                TypecheckerErrorKind::type_mismatch(&self.program.type_db, return_type_id, value_type_id).at(span)
            );
        }

        Ok(StatementKind::Return(value))
    }

    /// Visits the provided AST [`VariableAssignment`] statement.
    fn visit_statement_variable_assignment(
        &mut self,
        variable_assignment: ast::statement::variable_assignment::VariableAssignment,
        span: Span,
    ) -> TypecheckerResult<StatementKind> {
        match variable_assignment.target.kind {
            ast::expression::ExpressionKind::Dereference(inner_expression) => {
                let target = self.visit_expression(*inner_expression, None)?;

                // The type of the `reference_expr` should be a reference type.
                let Type::Reference(inner_type_id) = *self.program.type_db.get_type(target.type_id) else {
                    return Err(TypecheckerErrorKind::InvalidDereferenceTarget.at(target.span));
                };

                let value = self.visit_expression(*variable_assignment.value, Some(inner_type_id))?;

                if inner_type_id != value.type_id {
                    return Err(TypecheckerErrorKind::type_mismatch(
                        &self.program.type_db,
                        inner_type_id,
                        value.type_id,
                    )
                    .at(value.span));
                }

                Ok(StatementKind::ReferenceValueAssignment { target, value })
            }

            ast::expression::ExpressionKind::IdentifierReference(variable_name) => {
                let Some(variable_type_id) = self.scope.get_variable_ty(&variable_name).copied() else {
                    return Err(TypecheckerErrorKind::UnresolvableIdentifierReference(variable_name).at(span));
                };

                let value = self.visit_expression(*variable_assignment.value, Some(variable_type_id))?;
                if value.type_id != variable_type_id {
                    return Err(TypecheckerErrorKind::type_mismatch(
                        &self.program.type_db,
                        variable_type_id,
                        value.type_id,
                    )
                    .at(value.span));
                }

                Ok(StatementKind::VariableAssignment { name: variable_name, value, variable_type_id })
            }

            ast::expression::ExpressionKind::MemberAccess(member_access) => {
                // The target of the member access expression must be a structure type.
                let target = self.visit_expression(*member_access.target, None)?;

                let Type::Defined(defined_type_id) = *self.program.type_db.get_type(target.type_id) else {
                    return Err(TypecheckerErrorKind::ExpectedStructureType.at(span));
                };

                let DefinedTypeKind::Structure(structure) =
                    &self.program.type_db.get_defined_type(defined_type_id).kind;

                let (field_index, field_type_id) =
                    structure.find_field_by_name(&member_access.name).ok_or_else(|| {
                        TypecheckerErrorKind::UnresolvableIdentifierReference(member_access.name).at(span)
                    })?;

                let value = self.visit_expression(*variable_assignment.value, Some(field_type_id))?;

                if value.type_id != field_type_id {
                    return Err(TypecheckerErrorKind::type_mismatch(
                        &self.program.type_db,
                        field_type_id,
                        value.type_id,
                    )
                    .at(value.span));
                }

                Ok(StatementKind::StructureFieldAssignment {
                    target: Box::new(target),
                    field_index,
                    value: Box::new(value),
                })
            }

            _ => {
                trace!("Encountered invalid assignment target: '{:?}'", variable_assignment.target.kind);
                Err(TypecheckerErrorKind::InvalidAssignmentTarget.at(variable_assignment.target.span))
            }
        }
    }

    /// Visits the provided AST [`VariableDeclaration`] statement.
    fn visit_statement_variable_declaration(
        &mut self,
        variable_declaration: ast::statement::variable_declaration::VariableDeclaration,
        span: Span,
    ) -> TypecheckerResult<StatementKind> {
        // FIXME: Remove the clone
        let generic_type_parameters = self.scope.generic_type_parameters.clone();

        let variable_type_id = self.visit_type_expr(&generic_type_parameters, &variable_declaration.type_expr, span)?;
        let value = self.visit_expression(variable_declaration.value, Some(variable_type_id))?;

        if value.type_id != variable_type_id {
            return Err(TypecheckerErrorKind::type_mismatch(&self.program.type_db, variable_type_id, value.type_id)
                .at(value.span));
        }

        self.scope.set_variable_ty(variable_declaration.name.clone(), variable_type_id);
        Ok(StatementKind::VariableDeclaration { name: variable_declaration.name, value, type_id: variable_type_id })
    }
}

impl TypeResolver {
    /// Visits the provided AST [`Expression`]. The returned [`Expression`] will be the typed variant of it.
    fn visit_expression(
        &mut self,
        expression: ast::expression::Expression,
        expected_type_id: Option<TypeId>,
    ) -> TypecheckerResult<Expression> {
        let (kind, type_id) = match expression.kind {
            ast::expression::ExpressionKind::BinaryOperation(binary_operation) => {
                self.visit_expression_binary_operation(binary_operation, expected_type_id)?
            }

            ast::expression::ExpressionKind::BooleanLiteral(value) => self.visit_expression_boolean_literal(value),

            ast::expression::ExpressionKind::Dereference(reference) => self.visit_expression_dereference(*reference)?,

            ast::expression::ExpressionKind::FunctionCall(function_call) => {
                let (function_key, arguments, type_id) =
                    self.visit_expression_function_call(function_call, expression.span)?;

                (ExpressionKind::FunctionCall { function_key, arguments }, type_id)
            }

            ast::expression::ExpressionKind::IdentifierReference(identifier) => {
                self.visit_expression_identifier_reference(identifier, expression.span)?
            }

            ast::expression::ExpressionKind::MemberAccess(member_access) => {
                self.visit_expression_member_access(member_access, expression.span)?
            }

            ast::expression::ExpressionKind::NumberLiteral(value) => {
                self.visit_expression_number_literal(value, expected_type_id)
            }

            ast::expression::ExpressionKind::Reference(value) => self.visit_expression_reference(*value)?,

            ast::expression::ExpressionKind::StringLiteral(value) => {
                self.visit_expression_string_literal(value, expression.span)?
            }

            ast::expression::ExpressionKind::StructureInitialization(structure_initialization) => self
                .visit_expression_structure_initialization(
                    &structure_initialization,
                    expected_type_id,
                    expression.span,
                )?,

            _ => todo!("{expression:?}"),
        };

        Ok(Expression { kind, type_id, span: expression.span })
    }

    /// Visits the provided binary operation expression.
    fn visit_expression_binary_operation(
        &mut self,
        binary_operation: ast::expression::binary_operation::BinaryOperation,
        expected_type_id: Option<TypeId>,
    ) -> TypecheckerResult<(ExpressionKind, TypeId)> {
        let left = self.visit_expression(*binary_operation.left, expected_type_id)?;
        let right = self.visit_expression(*binary_operation.right, expected_type_id.or(Some(left.type_id)))?;

        // Both operands must be of the same type for them to be comparable.
        //
        // todo: types should have an `Add` | `Subtract` | `Divide` | `Multiply` | `Equals` protocol that they can
        //       conform to. If a type conforms to this protocol, then we should use its implementation here.
        if left.type_id != right.type_id {
            return Err(
                TypecheckerErrorKind::type_mismatch(&self.program.type_db, left.type_id, right.type_id).at(right.span)
            );
        }

        // The result type of the operation depends on the class of the operator.
        let type_id = match binary_operation.operator.class() {
            // The result of the operation should be a common type of the operands.
            BinaryOperatorClass::Arithmetic => left.type_id,

            // The result of the operation should be a boolean.
            BinaryOperatorClass::Comparison => self.program.type_db.boolean_type_id(),
        };

        Ok((
            ExpressionKind::BinaryOperation {
                left: Box::new(left),
                right: Box::new(right),
                operator: binary_operation.operator,
            },
            type_id,
        ))
    }

    /// Visits the provided boolean literal expression.
    fn visit_expression_boolean_literal(&mut self, value: bool) -> (ExpressionKind, TypeId) {
        (ExpressionKind::BooleanLiteral(value), self.program.type_db.boolean_type_id())
    }

    /// Visits the provided dereference expression.
    fn visit_expression_dereference(
        &mut self,
        expression: ast::expression::Expression,
    ) -> TypecheckerResult<(ExpressionKind, TypeId)> {
        // todo: get a reference of the expected type id
        let reference = self.visit_expression(expression, None)?;

        // The type of the reference expression must be a reference type.
        let Type::Reference(inner_type_id) = *self.program.type_db.get_type(reference.type_id) else {
            return Err(TypecheckerErrorKind::InvalidDereferenceTarget.at(reference.span));
        };

        Ok((ExpressionKind::Dereference(Box::new(reference)), inner_type_id))
    }

    /// Visits the provided function call expression.
    fn visit_expression_function_call(
        &mut self,
        function_call: ast::expression::function_call::FunctionCall,
        span: Span,
    ) -> TypecheckerResult<(FunctionKey, Vec<Expression>, TypeId)> {
        let generic_type_arguments = function_call
            .generic_type_arguments
            .iter()
            .map(|it| {
                // FIXME: Remove this clone.
                self.visit_type_expr(&self.scope.generic_type_parameters.clone(), &it.type_expr, it.span)
            })
            .collect::<TypecheckerResult<Vec<TypeId>>>()?;

        let call_target = self.resolve_function_call_target(*function_call.callee, &generic_type_arguments, span)?;
        let function_key = self.compute_function(&call_target, &generic_type_arguments, span)?;

        // TODO: remove clone
        let function = self.program.get_function(&function_key).clone();

        // If there are more arguments provided in the function call then there are of function parameters, then
        // the call is immediately invalid.
        let is_method_function_call = matches!(call_target, FunctionCallTarget::Method { .. });
        let provided_argument_count = function_call.arguments.len() + usize::from(is_method_function_call);

        if provided_argument_count > function.parameters.len() {
            return Err(TypecheckerErrorKind::FunctionCallArgumentCountMismatch {
                expected: function.parameters.len(),
                got: provided_argument_count,
            }
            .at(span));
        }

        let mut arguments = Vec::with_capacity(provided_argument_count);

        // FIXME: This is not perfect.
        //
        // 1. If a named argument is provided for a positional parameter, the error message is not great ("A positional
        //    argument must be provided for parameter '<name>'").
        // 2. A check is performed above to ensure that too many arguments are not passed to the call, this means that
        //    if an extra named argument (like `b` in `func foo(~a: i32)`) is passed, the error message doesn't tell
        //    you that a parameter named `b` does not exist.
        for (index, parameter) in function.parameters.iter().enumerate() {
            // If this is the first parameter, and this is a method function call, then we can provide a _reference_ to
            // the receiver.
            let argument_expression = if index == 0
                && let FunctionCallTarget::Method { receiver, .. } = &call_target
            {
                let reference_type_id = self.program.type_db.get_or_insert_type(Type::Reference(receiver.type_id));
                Expression {
                    kind: ExpressionKind::Reference(Box::new(receiver.clone())),
                    span: receiver.span,
                    type_id: reference_type_id,
                }
            } else {
                // We need to visit the expression first to ensure that it is valid.
                let expression = if parameter.is_named {
                    // If this is a named parameter, the argument corresponding to it should be provided with a name.
                    //
                    // There are multiple cases that we could encounter here, and this is probably not even an
                    // exhaustive list:
                    // 1. A named argument for the parameter doesn't exist
                    // 2. More than one named arguments for the parameter exists
                    let candidates = function_call
                        .arguments
                        .iter()
                        .filter(|it| it.name.as_ref().is_some_and(|name| name == &parameter.name))
                        .map(|it| &it.value)
                        .collect::<Vec<_>>();

                    if candidates.len() > 1 {
                        return Err(TypecheckerErrorKind::AmbiguousFunctionCallArgument(
                            parameter.name.clone(),
                            candidates.len(),
                        )
                        .at(span));
                    }

                    candidates.first().copied().ok_or_else(|| {
                        TypecheckerErrorKind::MissingNamedArgumentInFunctionCall(parameter.name.clone()).at(span)
                    })
                } else {
                    // If this is a method function call, then we can offset the argument index by one, as the first
                    // parameter & argument will be an implicit `this`.
                    let argument_index = index.saturating_sub(usize::from(is_method_function_call));

                    // The parameter is not named, so we can just assume that the argument is positional.
                    function_call
                        .arguments
                        .get(argument_index)
                        .filter(|it| it.name.is_none())
                        .map(|it| &it.value)
                        .ok_or_else(|| {
                            TypecheckerErrorKind::MissingPositionalArgumentInFunctionCall(parameter.name.clone())
                                .at(span)
                        })
                }?;

                self.visit_expression(expression.clone(), Some(parameter.type_id))?
            };

            // The type of the argument expression must match the parameter.
            if argument_expression.type_id != parameter.type_id {
                return Err(TypecheckerErrorKind::type_mismatch(
                    &self.program.type_db,
                    parameter.type_id,
                    argument_expression.type_id,
                )
                .at(argument_expression.span));
            }

            arguments.push(argument_expression);
        }

        Ok((function_key, arguments, function.return_type_id))
    }

    /// Attempts to resolve a callee from the provided [`ast::expression::Expression`].
    fn resolve_function_call_target(
        &mut self,
        expression: ast::expression::Expression,
        generic_type_arguments: &[TypeId],
        span: Span,
    ) -> TypecheckerResult<FunctionCallTarget> {
        let callee = match expression.kind {
            // If the callee is a plain identifier reference, then this is a regular function call (with no receiver
            // or associated type).
            ast::expression::ExpressionKind::IdentifierReference(identifier) => {
                FunctionCallTarget::Function { namespace: None, name: identifier }
            }

            // If the callee is a member access, then the function could either be:
            // - An instance method, if the target of the member access is a variable,
            // - or, an associated method, if the target of the member access is an identifier which matches the name
            //   of a defined type.
            ast::expression::ExpressionKind::MemberAccess(member_access) => {
                if let ast::expression::ExpressionKind::IdentifierReference(identifier) = &member_access.target.kind
                    && let Some(defined_type_id) =
                        self.program.type_db.find_defined_type(identifier, generic_type_arguments)
                {
                    // The function call is associated with a specific type.
                    let type_id = self.program.type_db.get_or_insert_type(Type::Defined(defined_type_id));
                    FunctionCallTarget::Associated { type_id, name: member_access.name }
                } else {
                    // The function call is not associated with a specific type, we can assume that this is an instance call.
                    let receiver = self.visit_expression(*member_access.target, None)?;
                    FunctionCallTarget::Method { receiver, name: member_access.name }
                }
            }

            // If the callee is a namespace qualifier, then this is a regular function call.
            ast::expression::ExpressionKind::NamespaceQualifier(namespace_qualifier) => FunctionCallTarget::Function {
                namespace: Some(namespace_qualifier.namespace),
                name: namespace_qualifier.identifier,
            },

            _ => return Err(TypecheckerErrorKind::InvalidFunctionCallTarget.at(span)),
        };

        Ok(callee)
    }

    /// Visits the provided identifier reference expression. An identifier reference will almost always be typed as
    /// [`ExpressionKind::VariableReference`].
    fn visit_expression_identifier_reference(
        &mut self,
        identifier: String,
        span: Span,
    ) -> TypecheckerResult<(ExpressionKind, TypeId)> {
        let variable_type_id = self
            .scope
            .get_identifier_ty(&identifier)
            .ok_or_else(|| TypecheckerErrorKind::UnresolvableIdentifierReference(identifier.clone()).at(span))?;

        Ok((ExpressionKind::VariableReference(identifier), *variable_type_id))
    }

    /// Visits the provided member access expression.
    fn visit_expression_member_access(
        &mut self,
        member_access: ast::expression::member_access::MemberAccess,
        span: Span,
    ) -> TypecheckerResult<(ExpressionKind, TypeId)> {
        // The type of the target must be a structure type. Anything else is not supported at the moment.
        let target = self.visit_expression(*member_access.target, None)?;
        let target_type_id = target.type_id;

        let Type::Defined(defined_type_id) = *self.program.type_db.get_type(target_type_id) else {
            return Err(TypecheckerErrorKind::ExpectedStructureType.at(span));
        };

        // The structure type must have a field with the provided name
        let DefinedTypeKind::Structure(structure) = &self.program.type_db.get_defined_type(defined_type_id).kind;

        let (field_index, field_type_id) = structure
            .find_field_by_name(&member_access.name)
            .ok_or_else(|| TypecheckerErrorKind::UnresolvableIdentifierReference(member_access.name).at(span))?;

        Ok((ExpressionKind::StructureFieldReference { target: Box::new(target), field_index }, field_type_id))
    }

    /// Visits the provided number literal expression.
    /// The type returned will be the "lowest" possible integer type supported by the literal.
    fn visit_expression_number_literal(
        &mut self,
        value: f64,
        expected_type_id: Option<TypeId>,
    ) -> (ExpressionKind, TypeId) {
        let minimum_type = if value < 0.0 {
            let bits = match value {
                v if v >= f64::from(i8::MIN) => 8,
                v if v >= f64::from(i16::MIN) => 16,
                v if v >= f64::from(i32::MIN) => 32,
                _ => 64,
            };

            Type::SignedInteger(bits)
        } else {
            let bits = match value {
                v if v >= f64::from(u8::MIN) => 8,
                v if v >= f64::from(u16::MIN) => 16,
                v if v >= f64::from(u32::MIN) => 32,
                _ => 64,
            };

            Type::UnsignedInteger(bits)
        };

        let coerced_type = match expected_type_id.map(|it| self.program.type_db.get_type(it)) {
            // The expected type suggests that we should use a signed type. All integer types are castable to signed.
            Some(Type::SignedInteger(expected_bits)) => match minimum_type {
                Type::SignedInteger(minimum_bits) | Type::UnsignedInteger(minimum_bits) => {
                    Type::SignedInteger(max(*expected_bits, minimum_bits))
                }

                _ => unreachable!("minimum_type can only be `Type::UnsignedInteger` or `Type::SignedInteger`"),
            },

            // The expected type suggests that we should use an unsigned type. Not all integer types are castable to unsigned.
            Some(Type::UnsignedInteger(expected_bits)) => match minimum_type {
                Type::UnsignedInteger(minimum_bits) => Type::UnsignedInteger(max(*expected_bits, minimum_bits)),

                // We still return the signed integer in this case, even though the hint suggests an unsigned integer.
                // The caller is responsible for checking whether the signed-ness is OK.
                Type::SignedInteger(_) => minimum_type,

                _ => unreachable!("minimum_type can only be `Type::UnsignedInteger` or `Type::SignedInteger`"),
            },

            // If the expected type is not an integer type, then we can just return the minimum integer type. The caller is
            // responsible for checking whether the type is valid.
            _ => minimum_type,
        };

        (ExpressionKind::NumberLiteral(value), self.program.type_db.get_or_insert_type(coerced_type))
    }

    // Visits the provided reference expression.
    fn visit_expression_reference(
        &mut self,
        value: ast::expression::Expression,
    ) -> TypecheckerResult<(ExpressionKind, TypeId)> {
        // todo(resolver): get a de-reference from the expected type id
        let expression = self.visit_expression(value, None)?;

        // The type of the reference expression is a reference to the expression's type.
        let type_id = self.program.type_db.get_or_insert_type(Type::Reference(expression.type_id));

        Ok((ExpressionKind::Reference(Box::new(expression)), type_id))
    }

    /// Visits the provided [`ast::expression::ExpressionKind::StringLiteral`] expression.
    fn visit_expression_string_literal(
        &mut self,
        value: String,
        span: Span,
    ) -> TypecheckerResult<(ExpressionKind, TypeId)> {
        // FIXME: `str` is an alias for `CompileTimeStr`.
        let type_id = self.visit_type_expr(
            &[],
            &TypeExpr::Named { name: "CompileTimeStr".to_string(), generic_type_arguments: vec![] },
            span,
        )?;

        // TODO: Program data section where we have a structure initialization that references it?
        // Ok(ExpressionKind::StructureInitialization { field_values: [ExpressionKind::DataReference(0), ExpressionKind::NumberLiteral(string_len)] })
        Ok((ExpressionKind::StringLiteral(value), type_id))
    }

    /// Visits the provided [`ast::expression::structure_initialization::StructureInitialization`] expression.
    ///
    /// The [`expected_type`] must be a structure type for this visit method to succeed. Otherwise, there is not enough
    /// information available to know which structure is being initialized.
    fn visit_expression_structure_initialization(
        &mut self,
        structure_initialization: &ast::expression::structure_initialization::StructureInitialization,
        expected_type_id: Option<TypeId>,
        span: Span,
    ) -> TypecheckerResult<(ExpressionKind, TypeId)> {
        let Some(expected_type_id) = expected_type_id else {
            panic!("visit_expression_structure_initialization did not get an `expected_type_id`");
        };

        let Type::Defined(defined_type_id) = self.program.type_db.get_type(expected_type_id) else {
            return Err(TypecheckerErrorKind::ExpectedStructureType.at(span));
        };

        let defined_type = self.program.type_db.get_defined_type(*defined_type_id);
        let DefinedTypeKind::Structure(structure) = &defined_type.kind.clone(); // todo: remove this clone

        // The number of fields on the structure initialization must match the number of values passed.
        if structure.fields.len() != structure_initialization.fields.len() {
            return Err(TypecheckerErrorKind::StructureInitializationFieldCountMismatch {
                expected: structure.fields.len(),
                got: structure_initialization.fields.len(),
            }
            .at(span));
        }

        // The initialization's fields may not be in order, we need to find them individually based on their name.
        let mut field_values: Vec<Expression> = Vec::with_capacity(structure.fields.len());

        for field in &structure.fields {
            // A corresponding initialization field must exist.
            let initialization_field =
                structure_initialization.fields.iter().find(|it| it.name == field.name).ok_or_else(|| {
                    TypecheckerErrorKind::MissingStructureFieldInInitializer(field.name.clone()).at(span)
                })?;

            let initializer_value = self.visit_expression(*initialization_field.value.clone(), Some(field.type_id))?;
            if field.type_id != initializer_value.type_id {
                return Err(TypecheckerErrorKind::type_mismatch(
                    &self.program.type_db,
                    field.type_id,
                    initializer_value.type_id,
                )
                .at(initializer_value.span));
            }

            field_values.push(initializer_value);
        }

        Ok((ExpressionKind::StructureInitialization { field_values }, expected_type_id))
    }
}
