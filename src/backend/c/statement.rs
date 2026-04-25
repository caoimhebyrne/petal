use crate::{
    ast::statement::{
        Statement,
        StatementKind,
        function_declaration::FunctionDeclaration,
        r#return::Return,
    },
    backend::c::{
        CBackend,
        error::CBackendError,
    },
    core::span::Span,
};

impl CBackend {
    /// Compiles a statement into C code.
    pub fn compile_statement(statement: &Statement) -> Result<String, CBackendError> {
        match &statement.kind {
            StatementKind::FunctionDeclaration(function_declaration) => {
                CBackend::compile_function_declaration(function_declaration, statement.span)
            }
            StatementKind::Return(r#return) => CBackend::compile_return(r#return, statement.span),
        }
    }

    /// Compiles a function declaration into C code.
    pub fn compile_function_declaration(
        function_declaration: &FunctionDeclaration,
        span: Span,
    ) -> Result<String, CBackendError> {
        let mut function = String::new();

        let name = function_declaration.name.clone();

        let return_type = function_declaration
            .return_type
            .as_ref()
            .map(|it| CBackend::compile_type(it, span))
            .transpose()?
            .unwrap_or("void".into());

        // TODO: Function parameters.

        function.push_str(&format!("{return_type} {name}(void) {{\n"));

        for statement in &function_declaration.body {
            function.push_str(&CBackend::compile_statement(statement)?);
            function.push('\n');
        }

        function.push_str("}\n");

        Ok(function)
    }

    /// Compiles a return statement into C code.
    pub fn compile_return(r#return: &Return, _span: Span) -> Result<String, CBackendError> {
        let expression = r#return.value.as_ref().map(CBackend::compile_expression).transpose()?;

        let string = match expression {
            Some(value) => format!("return {};", value),
            None => "return;".into(),
        };

        Ok(string)
    }
}
