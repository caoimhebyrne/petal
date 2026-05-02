use crate::{
    ast::statement::{
        Statement,
        StatementKind,
        function_declaration::{
            FunctionDeclaration,
            FunctionParameter,
        },
        r#if::If,
        r#return::Return,
        variable_assignment::VariableAssignment,
        variable_declaration::VariableDeclaration,
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

            StatementKind::FunctionCall(function_call) => {
                Ok(format!("{};", CBackend::compile_function_call(function_call, statement.span)?))
            }

            StatementKind::Import(_) => Ok("".into()),

            StatementKind::Return(r#return) => CBackend::compile_return(r#return, statement.span),

            StatementKind::VariableDeclaration(variable_declaration) => {
                CBackend::compile_variable_declaration(variable_declaration, statement.span)
            }

            StatementKind::VariableAssignment(variable_assignment) => {
                CBackend::compile_variable_assignment(variable_assignment, statement.span)
            }

            StatementKind::If(r#if) => CBackend::compile_if(r#if, statement.span),
        }
    }

    /// Compiles a function declaration into C code.
    pub fn compile_function_declaration(
        function_declaration: &FunctionDeclaration,
        span: Span,
    ) -> Result<String, CBackendError> {
        let mut function = String::new();

        let name = function_declaration.name.clone();

        let return_type = CBackend::compile_type(&function_declaration.return_type, span)?;

        let parameters: String = if function_declaration.parameters.is_empty() {
            "void".into()
        } else {
            function_declaration
                .parameters
                .iter()
                .map(CBackend::compile_function_parameter)
                .collect::<Result<Vec<String>, CBackendError>>()?
                .join(", ")
        };

        function.push_str(&format!("{return_type} {name}({parameters}) {{\n"));
        function.push_str(&CBackend::compile_block(&function_declaration.body)?);
        function.push_str("}\n\n");

        Ok(function)
    }

    /// Compiles a block into C code.
    fn compile_block(block: &Vec<Statement>) -> Result<String, CBackendError> {
        let mut string = String::new();

        for statement in block {
            string.push_str(&CBackend::compile_statement(statement)?);
            string.push('\n');
        }

        Ok(string)
    }

    /// Compiles a function parameter into C code.
    pub fn compile_function_parameter(function_parameter: &FunctionParameter) -> Result<String, CBackendError> {
        let name = function_parameter.name.clone();
        let r#type = CBackend::compile_type(&function_parameter.r#type, function_parameter.span)?;
        Ok(format!("{type} {name}"))
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

    /// Compiles a variable declaration into C code.
    pub fn compile_variable_declaration(
        variable_declaration: &VariableDeclaration,
        span: Span,
    ) -> Result<String, CBackendError> {
        let name = variable_declaration.name.clone();
        let r#type = CBackend::compile_type(&variable_declaration.r#type, span)?;
        let value = CBackend::compile_expression(&variable_declaration.value)?;

        Ok(format!("{type} {name} = {value};"))
    }

    /// Compiles a variable assignment into C code.
    pub fn compile_variable_assignment(
        variable_assignment: &VariableAssignment,
        _span: Span,
    ) -> Result<String, CBackendError> {
        let target = CBackend::compile_expression(&variable_assignment.target)?;
        let value = CBackend::compile_expression(&variable_assignment.value)?;
        Ok(format!("{target} = {value};"))
    }

    /// Compiles an if statement into C code.
    pub fn compile_if(r#if: &If, _span: Span) -> Result<String, CBackendError> {
        let condition = CBackend::compile_expression(&r#if.condition)?;
        let mut string = String::new();

        string.push_str(&format!("if ({condition}) {{\n"));
        string.push_str(&CBackend::compile_block(&r#if.block)?);
        string.push_str("}\n");

        Ok(string)
    }
}
