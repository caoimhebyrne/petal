use std::collections::HashMap;

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
        Program,
        Statement,
        StatementKind,
        context::{
            GenericFunction,
            TypeResolverContext,
        },
        error::{
            TypecheckerError,
            TypecheckerErrorKind,
        },
        r#type::Ty,
    },
};

type TypecheckerResult<T> = Result<T, TypecheckerError>;

/// The scope of a [`TypeResolver`].
#[derive(Default)]
struct Scope {
    /// The generic type parameters that are available in this scope.
    generic_type_parameters: Vec<String>,

    /// The type of parameters available to this scope.
    parameter_tys: HashMap<String, Ty>,

    /// The parent of this scope, if applicable.
    parent: Option<Box<Scope>>,

    /// The type of variables declared within this scope.
    variable_tys: HashMap<String, Ty>,
}

impl Scope {
    /// Creates an empty scope with a parent.
    pub fn empty_with_parent(parent: Self) -> Self {
        Self {
            generic_type_parameters: Vec::default(),
            parameter_tys: HashMap::default(),
            parent: Some(Box::new(parent)),
            variable_tys: HashMap::default(),
        }
    }

    /// Creates a scope with parameters and a parent.
    pub fn function(
        generic_type_parameters: Vec<String>,
        parameter_tys: HashMap<String, Ty>,
        parent: Option<Self>,
    ) -> Self {
        Self { generic_type_parameters, parameter_tys, parent: parent.map(Box::new), variable_tys: HashMap::default() }
    }

    /// Retrieves the type of an identifier by its name from the current scope.
    ///
    /// If a variable or parameter does not exist with the name in this scope, then the parent scope will be checked
    /// (if present). If one could not be found in any of the parent scopes, then [`None`] will be returned.
    pub fn get_identifier_ty(&self, identifier: &str) -> Option<&Ty> {
        self.parameter_tys
            .get(identifier)
            .or_else(|| self.variable_tys.get(identifier))
            .or_else(|| self.parent.as_ref().and_then(|it| it.get_identifier_ty(identifier)))
    }

    /// Inserts the type of a variable into the current scope, returning `true` if successful.
    ///
    /// This function will return `false` if a variable exists with the same name in this [`Scope`], or any of its
    /// parent [`Scope`]s.
    pub fn set_variable_ty(&mut self, variable_name: String, ty: Ty) -> bool {
        // todo(resolver): what should we do about parameters here?

        if self.get_identifier_ty(&variable_name).is_some() {
            false
        } else {
            self.variable_tys.insert(variable_name, ty);
            true
        }
    }
}

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
        for module in modules {
            self.visit_top_level_declaration_statements(module.ast)?;
        }

        Ok(self.program)
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

                _ => {
                    warn!(
                        "Ignoring unsupported top-level statement ({:?}) at source index {}",
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

    /// Sets the [`Scope`] to an empty one, making it a child of the current [`Scope`].
    fn push_scope(&mut self) {
        self.set_scope(Scope::empty_with_parent);
    }

    /// Sets the [`Scope`] to the parent of the current [`Scope`].
    /// This function will panic if [`Scope::parent`] is [`None`].
    fn pop_scope(&mut self) {
        self.set_scope(|current| *current.parent.expect("TypeResolver::scope should have a parent"));
    }
}

impl TypeResolver {
    /// Visits the provided [`TypeExpr`], resolving it into a [`Ty`].
    // TODO: Should `Span` be on `TypeExpr`?
    fn visit_type_expr(generic_type_parameters: &[String], expr: &TypeExpr, span: Span) -> TypecheckerResult<Ty> {
        match expr {
            TypeExpr::Named { name, .. } => TypeResolver::resolve_ty_by_name(generic_type_parameters, name, span),
            _ => todo!(),
        }
    }

    /// Attempts to resolve a type by the provided plain name, resolving it into a [`Ty`].
    fn resolve_ty_by_name(generic_type_parameters: &[String], name: &str, span: Span) -> TypecheckerResult<Ty> {
        let ty = match name {
            "i8" => Ty::SignedInteger(8),
            "i16" => Ty::SignedInteger(16),
            "i32" => Ty::SignedInteger(32),
            "i64" => Ty::SignedInteger(64),

            "u8" => Ty::UnsignedInteger(8),
            "u16" => Ty::UnsignedInteger(16),
            "u32" => Ty::UnsignedInteger(32),
            "u64" => Ty::UnsignedInteger(64),

            _ => {
                if let Some(index) = generic_type_parameters.iter().position(|it| it == name) {
                    Ty::Generic(index)
                } else {
                    return Err(TypecheckerErrorKind::UndeclaredTypeName(name.to_string()).at(span));
                }
            }
        };

        Ok(ty)
    }
}

impl TypeResolver {
    /// Attempts to find a function given its name, parameters, and expected return type.
    ///
    /// If a non-generic function cannot be found, a generic function will be resolved and specialized for the generic
    /// type parameters.
    // FIXME: Should return references
    fn compute_function(
        &mut self,
        name: &str,
        generic_type_arguments: Vec<Ty>,
        span: Span,
    ) -> TypecheckerResult<(FunctionKey, Function)> {
        // If a function exists that satisfies our restrictions, then we can use it.
        if let Some(tuple) = self.program.find_function(name) {
            return Ok((*tuple.0, tuple.1.clone()));
        }

        // Otherwise, we can attempt to find a generic function with the same/similar signature, and create a
        // specialization of it to be used.
        let Some((_, generic_function)) = self.context.find_generic_function(name) else {
            return Err(TypecheckerErrorKind::UnresolvableIdentifierReference(name.to_string()).at(span));
        };

        // The number of generic type arguments must equal the number of generic type parameters in the function. At a
        // later point in time, we may be able to infer these.
        if generic_type_arguments.len() != generic_function.generic_type_parameters.len() {
            return Err(TypecheckerErrorKind::GenericTypeArgumentCountMismatch {
                expected: generic_function.generic_type_parameters.len(),
                got: generic_type_arguments.len(),
            }
            .at(span));
        }

        let generic_types = generic_function
            .generic_type_parameters
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let generic_type_argument = generic_type_arguments[index];
                (name.clone(), generic_type_argument)
            })
            .collect();

        // The specialization that we generate will still have its generic types within its body, parameters, etc.
        // but we will make a note within the function to walk through it and resolve the generic types at a later
        // stage.
        let function = Function {
            name: generic_function.name.clone(),
            parameters: generic_function.parameters.clone(),
            body: generic_function.body.clone(),
            return_ty: generic_function.return_ty,
            generic_information: Some(GenericInformation { types: generic_types }),
            span: generic_function.span,
        };

        let function_key = self.program.insert_function(span.module_id, function.clone());
        Ok((function_key, function))
    }
}

impl TypeResolver {
    /// Visits the provided [`FunctionDeclaration`].
    ///
    /// If the function has generic type parameters, it will not be appended to the [`Program`], and will instead be
    /// stored to undergo monomorphization once a call is made to it. If any references to a generic type parameter are
    /// found within the function's declaration (parameter types, return type, body), then they will be stubbed with a
    /// generic type reference.
    ///
    /// If the function does not have generic type parameters, the function will be appended to the [`Program`].
    fn visit_function_declaration(
        &mut self,
        function_declaration: ast::statement::function_declaration::FunctionDeclaration,
        span: Span,
    ) -> TypecheckerResult<()> {
        let is_generic = !function_declaration.generic_type_parameters.is_empty();
        if is_generic {
            trace!(
                "Function named '{}' is generic. It will not be inserted into the program until monomorphized",
                function_declaration.name
            );
        } else {
            trace!(
                "Function named '{}' is not generic. It will be inserted directly into the program",
                function_declaration.name
            );
        }

        // todo(resolver): `TypeResolvingContext`
        let generic_type_parameters =
            function_declaration.generic_type_parameters.into_iter().map(|it| it.name).collect::<Vec<_>>();

        let parameters = function_declaration
            .parameters
            .into_iter()
            .map(|it| self.visit_function_parameter(&generic_type_parameters, it))
            .collect::<TypecheckerResult<Vec<_>>>()?;

        let return_ty = function_declaration
            .return_type_expr
            .map(|it| TypeResolver::visit_type_expr(&generic_type_parameters, &it, span))
            .transpose()?
            .unwrap_or(Ty::Void);

        self.set_scope(|current| {
            let parameter_tys = parameters.iter().map(|it| (it.name.clone(), it.ty)).collect();
            Scope::function(generic_type_parameters.clone(), parameter_tys, Some(current))
        });

        let body = self.visit_statements(function_declaration.body)?;

        self.pop_scope();

        if is_generic {
            self.context.insert_generic_function(
                span.module_id,
                GenericFunction {
                    name: function_declaration.name,
                    parameters,
                    body,
                    return_ty,
                    generic_type_parameters,
                    span,
                },
            );
        } else {
            self.program.insert_function(
                span.module_id,
                Function {
                    name: function_declaration.name,
                    parameters,
                    body,
                    return_ty,
                    generic_information: None,
                    span,
                },
            );
        }

        Ok(())
    }

    /// Visits the provided [`ast::statement::function_declaration::FunctionParameter`].
    /// The type declared by the parameter will be resolved, and transformed into a typed [`FunctionParameter`].
    fn visit_function_parameter(
        &mut self,
        generic_type_parameters: &[String],
        parameter: ast::statement::function_declaration::FunctionParameter,
    ) -> TypecheckerResult<FunctionParameter> {
        Ok(FunctionParameter {
            name: parameter.name,
            ty: TypeResolver::visit_type_expr(generic_type_parameters, &parameter.type_expr, parameter.span)?,
            is_named: parameter.is_named,
            span: parameter.span,
        })
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
            ast::statement::StatementKind::Return(r#return) => self.visit_statement_return(r#return),

            ast::statement::StatementKind::VariableDeclaration(variable_declaration) => {
                self.visit_statement_variable_declaration(variable_declaration, statement.span)
            }

            _ => todo!(),
        }?;

        Ok(kind.at(statement.span))
    }

    /// Visits the proivded AST [`Return`] statement.
    fn visit_statement_return(
        &mut self,
        r#return: ast::statement::r#return::Return,
    ) -> TypecheckerResult<StatementKind> {
        let value = r#return.value.map(|it| self.visit_expression(it)).transpose()?;
        Ok(StatementKind::Return(value))
    }

    /// Visits the provided AST [`VariableDeclaration`] statement.
    fn visit_statement_variable_declaration(
        &mut self,
        variable_declaration: ast::statement::variable_declaration::VariableDeclaration,
        span: Span,
    ) -> TypecheckerResult<StatementKind> {
        let ty =
            TypeResolver::visit_type_expr(&self.scope.generic_type_parameters, &variable_declaration.type_expr, span)?;
        let value = self.visit_expression(variable_declaration.value)?;

        self.scope.set_variable_ty(variable_declaration.name.clone(), ty);

        Ok(StatementKind::VariableDeclaration { name: variable_declaration.name, value, ty })
    }
}

impl TypeResolver {
    /// Visits the provided AST [`Expression`]. The returned [`Expression`] will be the typed variant of it.
    fn visit_expression(&mut self, expression: ast::expression::Expression) -> TypecheckerResult<Expression> {
        let (kind, ty) = match expression.kind {
            ast::expression::ExpressionKind::BinaryOperation(binary_operation) => {
                self.visit_expression_binary_operation(binary_operation)?
            }

            ast::expression::ExpressionKind::FunctionCall(function_call) => {
                self.visit_expression_function_call(function_call, expression.span)?
            }

            ast::expression::ExpressionKind::IdentifierReference(identifier) => {
                self.visit_expression_identifier_reference(identifier, expression.span)?
            }

            ast::expression::ExpressionKind::NumberLiteral(value) => self.visit_expression_number_literal(value),

            _ => todo!(),
        };

        Ok(Expression { kind, ty, span: expression.span })
    }

    /// Visits the provided binary operation expression.
    fn visit_expression_binary_operation(
        &mut self,
        binary_operation: ast::expression::binary_operation::BinaryOperation,
    ) -> TypecheckerResult<(ExpressionKind, Ty)> {
        let left = self.visit_expression(*binary_operation.left)?;
        let right = self.visit_expression(*binary_operation.right)?;

        // The type of the expression (for now) will be the type of the expression on the left-hand side.
        // This will be refined and verified at later stages, once we verify that the types are actually compatible with each other.
        let ty = left.ty;

        Ok((
            ExpressionKind::BinaryOperation {
                left: Box::new(left),
                right: Box::new(right),
                operator: binary_operation.operator,
            },
            ty,
        ))
    }

    /// Visits the provided function call expression.
    fn visit_expression_function_call(
        &mut self,
        function_call: ast::expression::function_call::FunctionCall,
        span: Span,
    ) -> TypecheckerResult<(ExpressionKind, Ty)> {
        // todo(resolver): resolve_function_callee?
        let ast::expression::ExpressionKind::IdentifierReference(identifier) = function_call.callee.kind else {
            panic!("Unsupported function callee: {function_call:?}");
        };

        // todo(resolver): inference based on type?
        let generic_type_arguments = function_call
            .generic_type_arguments
            .iter()
            .map(|it| TypeResolver::visit_type_expr(&self.scope.generic_type_parameters, &it.type_expr, it.span))
            .collect::<TypecheckerResult<_>>()?;

        let (function_key, function) = self.compute_function(&identifier, generic_type_arguments, span)?;

        // todo(resolver): named vs positional argumenmts
        let arguments = function_call
            .arguments
            .into_iter()
            .map(|it| self.visit_expression(it.value))
            .collect::<TypecheckerResult<_>>()?;

        Ok((ExpressionKind::FunctionCall { function_key, arguments }, function.return_ty))
    }

    /// Visits the provided identifier reference expression. An identifier reference will almost always be typed as
    /// [`ExpressionKind::VariableReference`].
    fn visit_expression_identifier_reference(
        &mut self,
        identifier: String,
        span: Span,
    ) -> TypecheckerResult<(ExpressionKind, Ty)> {
        let variable_ty = self
            .scope
            .get_identifier_ty(&identifier)
            .ok_or_else(|| TypecheckerErrorKind::UnresolvableIdentifierReference(identifier.clone()).at(span))?;

        Ok((ExpressionKind::VariableReference(identifier), *variable_ty))
    }

    /// Visits the provided number literal expression.
    /// The type returned will be the "lowest" possible integer type supported by the literal.
    fn visit_expression_number_literal(&mut self, value: f64) -> (ExpressionKind, Ty) {
        let ty = if value < 0.0 {
            let bits = match value {
                v if v >= f64::from(i8::MIN) => 8,
                v if v >= f64::from(i16::MIN) => 16,
                v if v >= f64::from(i32::MIN) => 32,
                _ => 64,
            };

            Ty::SignedInteger(bits)
        } else {
            let bits = match value {
                v if v >= f64::from(u8::MIN) => 8,
                v if v >= f64::from(u16::MIN) => 16,
                v if v >= f64::from(u32::MIN) => 32,
                _ => 64,
            };

            Ty::UnsignedInteger(bits)
        };

        (ExpressionKind::NumberLiteral(value), ty)
    }
}
