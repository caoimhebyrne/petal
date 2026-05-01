use crate::{
    ast::statement::{
        StatementKind,
        function_declaration::{
            FunctionDeclaration,
            FunctionParameter,
        },
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
            .map(|it| Typechecker::resolve_type_from_expr(it, span))
            .transpose()?
            .unwrap_or(Type::Void);

        for parameter in &mut function_declaration.parameters {
            DeclarationPass::check_function_parameter(parameter)?;
        }

        self.typechecker.context.insert_checked_function(function_declaration, span)
    }

    /// Checks and resolves any [`Type`]s referenced in the provided [`FunctionParameter`].
    fn check_function_parameter(function_parameter: &mut FunctionParameter) -> Result<Type, TypecheckerError> {
        let r#type = Typechecker::resolve_type_from_expr(&function_parameter.type_expr, function_parameter.span)?;
        function_parameter.r#type = r#type;
        Ok(r#type)
    }
}
