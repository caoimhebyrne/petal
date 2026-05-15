use crate::{
    ast::{
        statement::{
            StatementKind,
            function_declaration::{
                FunctionDeclaration,
                FunctionParameter,
            },
            namespace_declaration::NamespaceDeclaration,
            type_declaration::TypeDeclaration,
        },
        type_expr::TypeExpr,
    },
    core::span::Span,
    module::ParsedModule,
    typechecker::{
        TypeResolvingContext,
        Typechecker,
        error::{
            TypecheckerError,
            TypecheckerErrorKind,
        },
        r#type::{
            StructureReference,
            Type,
        },
    },
};

pub(crate) struct DeclarationPass<'a> {
    /// The name of the current namespace.
    current_namespace: Option<String>,

    typechecker: &'a mut Typechecker,
}

impl<'a> DeclarationPass<'a> {
    /// Creates a new [`DeclarationPass`] with the given [`Typechecker]`.
    pub fn new(typechecker: &'a mut Typechecker) -> Self {
        Self { current_namespace: None, typechecker }
    }

    /// Runs the declaration pass on the provided [`ParsedModule`]s.
    pub fn run(&mut self, modules: &mut Vec<ParsedModule>) -> Result<(), TypecheckerError> {
        for module in modules {
            for statement in &mut module.ast {
                match &mut statement.kind {
                    StatementKind::FunctionDeclaration(function_declaration) => {
                        self.visit_function_declaration(function_declaration, statement.span)?;
                    }

                    StatementKind::TypeDeclaration(type_declaration) => {
                        self.visit_type_declaration(type_declaration, statement.span)?;
                    }

                    StatementKind::NamespaceDeclaration(namespace_declaration) => {
                        self.visit_namespace_declaration(namespace_declaration, statement.span)?
                    }

                    // We don't have to do anything at this pass for imports.
                    StatementKind::Import(_) => {}

                    _ => {
                        return Err(TypecheckerErrorKind::StatementNotSupportedAtPass {
                            pass_name: "declaration".into(),
                        }
                        .at(statement.span));
                    }
                }
            }
        }

        Ok(())
    }

    /// Visits the provided [`FunctionDeclaration`] and inserts the [`CheckedFunction`] for it into the
    /// [`Typechecker`]'s context.
    fn visit_function_declaration(
        &mut self,
        function_declaration: &mut FunctionDeclaration,
        span: Span,
    ) -> Result<(), TypecheckerError> {
        let type_resolving_context = TypeResolvingContext {
            generic_type_parameters: &function_declaration.generic_type_parameters,
            implicit_this_type: function_declaration.owner_type_name.as_ref(),
        };

        function_declaration.return_type = function_declaration
            .return_type_expr
            .as_mut()
            .map(|it| self.typechecker.resolve_type_from_expr(it, type_resolving_context, span))
            .transpose()?
            .unwrap_or(Type::Void);

        for parameter in &mut function_declaration.parameters {
            self.check_function_parameter(parameter, type_resolving_context)?;
        }

        let function_id = self.typechecker.context.insert_checked_function(
            self.current_namespace.clone(),
            function_declaration,
            span,
        )?;

        function_declaration.function_id = Some(function_id);

        Ok(())
    }

    /// Checks and resolves any [`Type`]s referenced in the provided [`FunctionParameter`].
    fn check_function_parameter(
        &mut self,
        function_parameter: &mut FunctionParameter,
        type_resolving_context: TypeResolvingContext,
    ) -> Result<Type, TypecheckerError> {
        let r#type = self.typechecker.resolve_type_from_expr(
            &mut function_parameter.type_expr,
            type_resolving_context,
            function_parameter.span,
        )?;
        function_parameter.r#type = r#type.clone();
        Ok(r#type)
    }

    /// Visits the provided [`TypeDeclaration`] and inserts it into the [`Typechecker`]'s context.
    fn visit_type_declaration(
        &mut self,
        type_declaration: &mut TypeDeclaration,
        span: Span,
    ) -> Result<(), TypecheckerError> {
        // If this is a structure type, then we must assign a structure ID for it.
        match &mut type_declaration.type_expr {
            TypeExpr::Structure { fields } => {
                // FIXME: Doing this early means that the order which structures are declared in matters. This is
                //        the case regardless, since the codegen does not do any sorting either.
                let type_resolving_context = TypeResolvingContext {
                    generic_type_parameters: &type_declaration.generic_type_parameters,
                    implicit_this_type: None,
                };

                for field in &mut *fields {
                    field.r#type = self.typechecker.resolve_type_from_expr(
                        &mut field.type_expr,
                        type_resolving_context,
                        field.span,
                    )?;
                }

                self.typechecker.context.insert_computed_declared_type(
                    self.current_namespace.clone(),
                    type_declaration.name.clone(),
                    type_declaration.modifiers.clone(),
                    type_declaration.generic_type_parameters.clone(),
                    span,
                    |ctx, declared_type_id| {
                        // All we need to do right now is register the struct. We will do a second pass after all declarations
                        // are run to resolve the types of the structure fields.
                        let structure_id = ctx.insert_declared_structure(declared_type_id, fields.clone(), span);

                        // FIXME: Find a better place for this.
                        if self.current_namespace == Some("prelude".into()) && type_declaration.name == "CompileTimeStr"
                        {
                            ctx.builtin_types.compile_time_str = Some(structure_id);
                        }

                        Type::Structure(StructureReference::Plain(structure_id))
                    },
                )?;
            }

            TypeExpr::Enum { variants } => {
                self.typechecker.context.insert_computed_declared_type(
                    self.current_namespace.clone(),
                    type_declaration.name.clone(),
                    type_declaration.modifiers.clone(),
                    type_declaration.generic_type_parameters.clone(),
                    span,
                    |ctx, declared_type_id| {
                        let enum_id = ctx.insert_enum(declared_type_id, variants.clone());
                        Type::Enum(enum_id)
                    },
                )?;
            }

            expr => {
                let resolved_type = self.typechecker.resolve_type_from_expr(
                    expr,
                    TypeResolvingContext {
                        generic_type_parameters: &type_declaration.generic_type_parameters,
                        implicit_this_type: None,
                    },
                    span,
                )?;

                self.typechecker.context.insert_declared_type(
                    self.current_namespace.clone(),
                    type_declaration.name.clone(),
                    resolved_type,
                    type_declaration.modifiers.clone(),
                    type_declaration.generic_type_parameters.clone(),
                    span,
                )?;
            }
        }

        Ok(())
    }

    /// Visits the provided [`NamespaceDeclaration`].
    fn visit_namespace_declaration(
        &mut self,
        namespace_declaration: &mut NamespaceDeclaration,
        _span: Span,
    ) -> Result<(), TypecheckerError> {
        self.current_namespace = Some(namespace_declaration.name.clone());

        for statement in &mut namespace_declaration.body {
            match &mut statement.kind {
                StatementKind::FunctionDeclaration(function_declaration) => {
                    debug!(
                        "Visiting function declaration for '{}' in namespace '{}'",
                        function_declaration.name, namespace_declaration.name
                    );
                    self.visit_function_declaration(function_declaration, statement.span)?;
                }

                StatementKind::TypeDeclaration(type_declaration) => {
                    self.visit_type_declaration(type_declaration, statement.span)?;
                }

                _ => {
                    return Err(TypecheckerErrorKind::StatementNotSupportedAtPass { pass_name: "declaration".into() }
                        .at(statement.span));
                }
            }
        }

        self.current_namespace = None;

        Ok(())
    }
}
