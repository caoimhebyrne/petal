use crate::{
    ast::{
        self,
        type_expr::TypeExpr,
    },
    core::span::Span,
    module::ParsedModule,
    typed_ast::{
        Expression,
        ExpressionKind,
        Function,
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
            context::TypeResolverContext,
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
                    self.pre_visit_top_level_declaration_statements(&namespace_declaration.body)?;
                }

                ast::statement::StatementKind::FunctionDeclaration(function_declaration) => {
                    self.pre_visit_function_declaration(function_declaration);
                }

                ast::statement::StatementKind::TypeDeclaration(type_declaration) => {
                    self.pre_visit_type_declaration(type_declaration);
                }

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
                    self.visit_top_level_declaration_statements(namespace_declaration.body)?;
                }

                ast::statement::StatementKind::FunctionDeclaration(function_declaration) => {
                    self.visit_function_declaration(function_declaration, statement.span)?;
                }

                ast::statement::StatementKind::TypeDeclaration(type_declaration) => {
                    self.visit_type_declaration(type_declaration, statement.span)?;
                }

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
        name: &str,
        generic_type_arguments: &[TypeId],
        span: Span,
    ) -> TypecheckerResult<FunctionKey> {
        // If a function exists that satisfies our restrictions, then we can use it.
        if let Some(tuple) = self.program.find_function(name, generic_type_arguments) {
            return Ok(*tuple.0);
        }

        // We can attempt to find an existing function declaration. This may or may not be generic.
        let Some(function_declaration) = self.context.find_function_declaration(name) else {
            return Err(TypecheckerErrorKind::UndeclaredFunction(name.to_string()).at(span));
        };

        // The number of generic type arguments must equal the number of generic type parameters in the function. At a
        // later point in time, we may be able to infer these.
        if generic_type_arguments.len() != function_declaration.generic_type_parameters.len() {
            return Err(TypecheckerErrorKind::GenericTypeArgumentCountMismatch {
                expected: function_declaration.generic_type_parameters.len(),
                got: generic_type_arguments.len(),
            }
            .at(span));
        }

        // todo(resolver): `TypeResolvingContext`
        let generic_type_parameters = function_declaration
            .generic_type_parameters
            .iter()
            .zip(generic_type_arguments)
            .map(|(parameter, argument_type_id)| GenericTypeParameter {
                name: parameter.name.clone(),
                type_id: *argument_type_id,
            })
            .collect::<Vec<GenericTypeParameter>>();

        self.compile_function_declaration(function_declaration.clone(), &generic_type_parameters, span)
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
        self.context.insert_function_declaration(function_declaration.clone());
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
        if self.program.find_function(&function_declaration.name, &[]).is_some() {
            return Ok(());
        }

        // Otherwise, we can compile the function as normal.
        self.compile_function_declaration(function_declaration, &[], span)?;
        Ok(())
    }

    /// Compiles the provided [`ast::statement::function_declaration::FunctionDeclaration`].
    fn compile_function_declaration(
        &mut self,
        function_declaration: ast::statement::function_declaration::FunctionDeclaration,
        generic_type_parameters: &[GenericTypeParameter],
        span: Span,
    ) -> TypecheckerResult<FunctionKey> {
        let parameters = function_declaration
            .parameters
            .into_iter()
            .map(|it| self.visit_function_parameter(generic_type_parameters, it))
            .collect::<TypecheckerResult<Vec<_>>>()?;

        let return_type_id = function_declaration
            .return_type_expr
            .map(|it| self.visit_type_expr(generic_type_parameters, &it, span))
            .transpose()?
            .unwrap_or(self.program.type_db.void_type_id());

        self.set_scope(|current| {
            let parameter_tys = parameters.iter().map(|it| (it.name.clone(), it.type_id)).collect();
            Scope::function(generic_type_parameters.into(), parameter_tys, Some(current))
        });

        let body = self.visit_statements(function_declaration.body)?;

        self.pop_scope();

        let function_key = self.program.insert_function(
            span.module_id,
            Function {
                name: function_declaration.name,
                parameters,
                body,
                return_type_id,
                // If no generic type parameters were provided, then we should not attach any generic information.
                generic_information: if generic_type_parameters.is_empty() {
                    None
                } else {
                    Some(GenericInformation { parameters: generic_type_parameters.into() })
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
        // todo(resolver): modifiers

        let defined_type_kind =
            self.visit_type_expr_on_declaration(generic_type_parameters, type_declaration.type_expr, span)?;

        let defined_type_id = self.program.type_db.insert_defined_type(DefinedType {
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

            ast::statement::StatementKind::Return(r#return) => self.visit_statement_return(r#return)?,

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
    ) -> TypecheckerResult<StatementKind> {
        // todo: self.scope.return_type
        let value = r#return.value.map(|it| self.visit_expression(it, None)).transpose()?;
        Ok(StatementKind::Return(value))
    }

    /// Visits the provided AST [`VariableAssignment`] statement.
    fn visit_statement_variable_assignment(
        &mut self,
        variable_assignment: ast::statement::variable_assignment::VariableAssignment,
        span: Span,
    ) -> TypecheckerResult<StatementKind> {
        match variable_assignment.target.kind {
            ast::expression::ExpressionKind::IdentifierReference(variable_name) => {
                let Some(variable_type_id) = self.scope.get_variable_ty(&variable_name).copied() else {
                    return Err(TypecheckerErrorKind::UnresolvableIdentifierReference(variable_name).at(span));
                };

                let value = self.visit_expression(*variable_assignment.value, Some(variable_type_id))?;
                Ok(StatementKind::VariableAssignment { name: variable_name, value, variable_type_id })
            }

            ast::expression::ExpressionKind::Dereference(target_expression) => {
                let target = self.visit_expression(*target_expression, None)?;
                let value = self.visit_expression(*variable_assignment.value, None)?;
                Ok(StatementKind::ReferenceValueAssignment { target, value })
            }

            _ => Err(TypecheckerErrorKind::InvalidAssignmentTarget.at(variable_assignment.target.span)),
        }
    }

    /// Visits the provided AST [`VariableDeclaration`] statement.
    fn visit_statement_variable_declaration(
        &mut self,
        variable_declaration: ast::statement::variable_declaration::VariableDeclaration,
        span: Span,
    ) -> TypecheckerResult<StatementKind> {
        // FIXME: Remove the clone
        let generic_type_parameters = &self.scope.generic_type_parameters.clone();
        let type_id = self.visit_type_expr(generic_type_parameters, &variable_declaration.type_expr, span)?;

        self.scope.set_variable_ty(variable_declaration.name.clone(), type_id);

        let value = self.visit_expression(variable_declaration.value, Some(type_id))?;
        Ok(StatementKind::VariableDeclaration { name: variable_declaration.name, value, type_id })
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
                self.visit_expression_binary_operation(binary_operation)?
            }

            ast::expression::ExpressionKind::Dereference(reference) => self.visit_expression_dereference(*reference)?,

            ast::expression::ExpressionKind::FunctionCall(function_call) => {
                let (function_key, arguments, type_id) =
                    self.visit_expression_function_call(function_call, expression.span)?;

                (ExpressionKind::FunctionCall { function_key, arguments }, type_id)
            }

            ast::expression::ExpressionKind::IdentifierReference(identifier) => {
                self.visit_expression_identifier_reference(identifier, expression.span)?
            }

            ast::expression::ExpressionKind::NumberLiteral(value) => self.visit_expression_number_literal(value),

            ast::expression::ExpressionKind::Reference(value) => self.visit_expression_reference(*value)?,

            ast::expression::ExpressionKind::StructureInitialization(structure_initialization) => self
                .visit_expression_structure_initialization(
                    &structure_initialization,
                    expected_type_id,
                    expression.span,
                )?,

            _ => todo!(),
        };

        Ok(Expression { kind, type_id, span: expression.span })
    }

    /// Visits the provided binary operation expression.
    fn visit_expression_binary_operation(
        &mut self,
        binary_operation: ast::expression::binary_operation::BinaryOperation,
    ) -> TypecheckerResult<(ExpressionKind, TypeId)> {
        let left = self.visit_expression(*binary_operation.left, None)?;
        let right = self.visit_expression(*binary_operation.right, Some(left.type_id))?;

        // The type of the expression (for now) will be the type of the expression on the left-hand side.
        // This will be refined and verified at later stages, once we verify that the types are actually compatible with each other.
        let type_id = left.type_id;

        Ok((
            ExpressionKind::BinaryOperation {
                left: Box::new(left),
                right: Box::new(right),
                operator: binary_operation.operator,
            },
            type_id,
        ))
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
        // todo(resolver): resolve_function_callee?
        let ast::expression::ExpressionKind::IdentifierReference(identifier) = function_call.callee.kind else {
            panic!("Unsupported function callee: {function_call:?}");
        };

        // todo(resolver): inference based on type?
        let generic_type_arguments = function_call
            .generic_type_arguments
            .iter()
            .map(|it| {
                // FIXME: Remove this clone.
                self.visit_type_expr(&self.scope.generic_type_parameters.clone(), &it.type_expr, it.span)
            })
            .collect::<TypecheckerResult<Vec<TypeId>>>()?;

        let function_key = self.compute_function(&identifier, &generic_type_arguments, span)?;

        // todo(resolver): named vs positional argumenmts
        let arguments = function_call
            .arguments
            .into_iter()
            // todo: expected type id
            .map(|it| self.visit_expression(it.value, None))
            .collect::<TypecheckerResult<_>>()?;

        let function = self.program.get_function(&function_key);
        Ok((function_key, arguments, function.return_type_id))
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

    /// Visits the provided number literal expression.
    /// The type returned will be the "lowest" possible integer type supported by the literal.
    fn visit_expression_number_literal(&mut self, value: f64) -> (ExpressionKind, TypeId) {
        let ty = if value < 0.0 {
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

        let type_id = self.program.type_db.get_or_insert_type(ty);
        (ExpressionKind::NumberLiteral(value), type_id)
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

        // The initialization's fields may not be in order, we need to find them individually based on their name.
        let mut field_values: Vec<Expression> = Vec::new();

        for field in &structure.fields {
            // A corresponding initialization field must exist.
            let initialization_field =
                structure_initialization.fields.iter().find(|it| it.name == field.name).ok_or_else(|| {
                    TypecheckerErrorKind::MissingStructureFieldInInitializer(field.name.clone()).at(span)
                })?;

            let field_value = self.visit_expression(*initialization_field.value.clone(), Some(field.type_id))?;
            field_values.push(field_value);
        }

        // The number of fields on the structure initialization must match the number of values passed.
        if structure.fields.len() != field_values.len() {
            return Err(TypecheckerErrorKind::StructureInitializationFieldCountMismatch {
                expected: structure.fields.len(),
                got: field_values.len(),
            }
            .at(span));
        }

        Ok((ExpressionKind::StructureInitialization { field_values }, expected_type_id))
    }
}
