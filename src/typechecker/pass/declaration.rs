use crate::{
    ast::{
        statement::{
            StatementKind,
            function_declaration::{
                FunctionDeclaration,
                FunctionParameter,
            },
            type_declaration::TypeDeclaration,
        },
        type_expr::TypeExpr,
    },
    core::span::Span,
    module::ParsedModule,
    typechecker::{
        Typechecker,
        error::TypecheckerError,
        r#type::Type,
    },
};

pub(crate) struct DeclarationPass<'a> {
    typechecker: &'a mut Typechecker,
}

impl<'a> DeclarationPass<'a> {
    /// Creates a new [`DeclarationPass`] with the given [`Typechecker]`.
    pub fn new(typechecker: &'a mut Typechecker) -> Self {
        Self { typechecker }
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

                    // We don't have to do anything at this pass for imports.
                    StatementKind::Import(_) => {}

                    _ => panic!("Statement '{:?}' not supported at declaration pass", statement.kind),
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
        function_declaration.return_type = function_declaration
            .return_type_expr
            .as_ref()
            .map(|it| self.typechecker.resolve_type_from_expr(it, span))
            .transpose()?
            .unwrap_or(Type::Void);

        for parameter in &mut function_declaration.parameters {
            self.check_function_parameter(parameter)?;
        }

        let function_id = self.typechecker.context.insert_checked_function(function_declaration, span)?;

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
                // We must ensure that the fields have a resolvable type.
                for field in &mut *fields {
                    field.r#type = self.typechecker.resolve_type_from_expr(&field.type_expr, field.span)?;
                }

                // Then, we can register the struct.
                let structure_id = self.typechecker.context.insert_declared_structure(
                    type_declaration.name.clone(),
                    fields.clone(),
                    span,
                )?;

                Type::Structure(structure_id)
            }

            expr => self.typechecker.resolve_type_from_expr(expr, span)?,
        };

        // We have resolved the type, we can insert it into the type declarations.
        self.typechecker.context.insert_declared_type(type_declaration.name.clone(), resolved_type, span)?;

        Ok(())
    }
}
