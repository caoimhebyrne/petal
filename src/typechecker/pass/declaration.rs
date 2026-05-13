use std::mem;

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
        Typechecker,
        error::{
            TypecheckerError,
            TypecheckerErrorKind,
        },
        r#type::Type,
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

        self.resolve_structure_field_types()?;

        Ok(())
    }

    /// Attempts to resolve the types of all fields in the [`DeclaredStructure`]s registered in the [`Typechecker`].
    fn resolve_structure_field_types(&mut self) -> Result<(), TypecheckerError> {
        // We cannot just iterate over `&mut self.typechecker.context.structures`. We must take ownership of it first,
        // and then set it back once we are finished.
        let mut structures = mem::take(&mut self.typechecker.context.structures);

        for structure in structures.values_mut() {
            for field in &mut structure.fields {
                field.r#type = self.typechecker.resolve_type_from_expr(&field.type_expr, field.span)?;
            }
        }

        self.typechecker.context.structures = structures;
        Ok(())
    }

    /// Visits the provided [`FunctionDeclaration`] and inserts the [`CheckedFunction`] for it into the
    /// [`Typechecker`]'s context.
    fn visit_function_declaration(
        &mut self,
        function_declaration: &mut FunctionDeclaration,
        span: Span,
    ) -> Result<(), TypecheckerError> {
        function_declaration.return_type = function_declaration
            .return_type_expr
            .as_ref()
            .map(|it| self.typechecker.resolve_type_from_expr(it, span))
            .transpose()?
            .unwrap_or(Type::Void);

        for parameter in &mut function_declaration.parameters {
            self.check_function_parameter(parameter)?;
        }

        let function_id = self.typechecker.context.insert_checked_function(
            self.current_namespace.clone(),
            function_declaration,
            span,
        )?;

        // Finally, we need to modify the name of the function that we are generating.
        function_declaration.name = self.typechecker.context.get_checked_function_by_id(function_id).name.clone();

        Ok(())
    }

    /// Checks and resolves any [`Type`]s referenced in the provided [`FunctionParameter`].
    fn check_function_parameter(
        &mut self,
        function_parameter: &mut FunctionParameter,
    ) -> Result<Type, TypecheckerError> {
        let r#type = self.typechecker.resolve_type_from_expr(&function_parameter.type_expr, function_parameter.span)?;
        function_parameter.r#type = r#type.clone();
        Ok(r#type)
    }

    /// Visits the provided [`TypeDeclaration`] and inserts it into the [`Typechecker`]'s context.
    fn visit_type_declaration(
        &mut self,
        type_declaration: &mut TypeDeclaration,
        span: Span,
    ) -> Result<(), TypecheckerError> {
        // TODO: If a type already exists with the same name, then we must not declare another.

        // If this is a structure type, then we must assign a structure ID for it.
        let resolved_type = match &mut type_declaration.type_expr {
            TypeExpr::Structure { fields } => {
                // All we need to do right now is register the struct. We will do a second pass after all declarations
                // are run to resolve the types of the structure fields.
                let structure_id = self.typechecker.context.insert_declared_structure(
                    self.current_namespace.clone(),
                    type_declaration.name.clone(),
                    fields.clone(),
                    span,
                )?;

                if self.current_namespace == Some("prelude".into()) && type_declaration.name == "CompileTimeStr" {
                    self.typechecker.context.builtin_types.compile_time_str = Some(structure_id);
                }

                Type::Structure(structure_id)
            }

            expr => self.typechecker.resolve_type_from_expr(expr, span)?,
        };

        // We have resolved the type, we can insert it into the type declarations.
        self.typechecker.context.insert_declared_type(
            self.current_namespace.clone(),
            type_declaration.name.clone(),
            resolved_type,
            type_declaration.modifiers.clone(),
            span,
        )?;

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

        Ok(())
    }
}
