use crate::{
    ast::statement::{
        Statement,
        StatementKind,
        function_declaration::{
            FunctionDeclaration,
            FunctionParameter,
        },
        r#if::If,
        namespace_declaration::NamespaceDeclaration,
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
    pub fn compile_statement(&self, statement: &Statement) -> Result<String, CBackendError> {
        match &statement.kind {
            StatementKind::FunctionDeclaration(function_declaration) => {
                self.compile_function_declaration(function_declaration, statement.span)
            }

            StatementKind::FunctionCall(function_call) => {
                Ok(format!("{};", self.compile_function_call(function_call, statement.span)?))
            }

            StatementKind::Return(r#return) => self.compile_return(r#return, statement.span),

            StatementKind::VariableDeclaration(variable_declaration) => {
                self.compile_variable_declaration(variable_declaration, statement.span)
            }

            StatementKind::VariableAssignment(variable_assignment) => {
                self.compile_variable_assignment(variable_assignment, statement.span)
            }

            StatementKind::If(r#if) => self.compile_if(r#if, statement.span),

            StatementKind::NamespaceDeclaration(namespace_declaration) => {
                self.compile_namespace_declaration(namespace_declaration, statement.span)
            }

            StatementKind::Import(_) => Ok("".into()),
            StatementKind::TypeDeclaration(_) => Ok("".into()),
        }
    }

    /// Compiles a function declaration into C code.
    pub fn compile_function_declaration(
        &self,
        function_declaration: &FunctionDeclaration,
        span: Span,
    ) -> Result<String, CBackendError> {
        let mut function = String::new();

        let name = function_declaration.name.clone();

        let return_type = self.compile_type(&function_declaration.return_type, span)?;

        let parameters: String = if function_declaration.parameters.is_empty() {
            "void".into()
        } else {
            function_declaration
                .parameters
                .iter()
                .map(|it| self.compile_function_parameter(it))
                .collect::<Result<Vec<String>, CBackendError>>()?
                .join(", ")
        };

        function.push_str(&format!("{return_type} {name}({parameters}) {{\n"));
        function.push_str(&self.compile_block(&function_declaration.body)?);
        function.push_str("}\n\n");

        Ok(function)
    }

    /// Compiles a block into C code.
    fn compile_block(&self, block: &Vec<Statement>) -> Result<String, CBackendError> {
        let mut string = String::new();

        for statement in block {
            string.push_str(&self.compile_statement(statement)?);
            string.push('\n');
        }

        Ok(string)
    }

    /// Compiles a function parameter into C code.
    pub fn compile_function_parameter(&self, function_parameter: &FunctionParameter) -> Result<String, CBackendError> {
        let name = function_parameter.name.clone();
        let r#type = self.compile_type(&function_parameter.r#type, function_parameter.span)?;
        Ok(format!("{type} {name}"))
    }

    /// Compiles a return statement into C code.
    pub fn compile_return(&self, r#return: &Return, _span: Span) -> Result<String, CBackendError> {
        let expression = r#return.value.as_ref().map(|it| self.compile_expression(it)).transpose()?;

        let string = match expression {
            Some(value) => format!("return {};", value),
            None => "return;".into(),
        };

        Ok(string)
    }

    /// Compiles a variable declaration into C code.
    pub fn compile_variable_declaration(
        &self,
        variable_declaration: &VariableDeclaration,
        span: Span,
    ) -> Result<String, CBackendError> {
        let name = variable_declaration.name.clone();
        let r#type = self.compile_type(&variable_declaration.r#type, span)?;

        let mut declaration = format!("{type} {name} = ");

        if let Some(value) = &variable_declaration.value {
            declaration.push_str(&self.compile_expression(value)?);
        } else {
            declaration.push_str("{0}");
        }

        declaration.push(';');
        Ok(declaration)
    }

    /// Compiles a variable assignment into C code.
    pub fn compile_variable_assignment(
        &self,
        variable_assignment: &VariableAssignment,
        _span: Span,
    ) -> Result<String, CBackendError> {
        let target = self.compile_expression(&variable_assignment.target)?;
        let value = self.compile_expression(&variable_assignment.value)?;
        Ok(format!("{target} = {value};"))
    }

    /// Compiles an if statement into C code.
    pub fn compile_if(&self, r#if: &If, _span: Span) -> Result<String, CBackendError> {
        let condition = self.compile_expression(&r#if.condition)?;
        let mut string = String::new();

        string.push_str(&format!("if ({condition}) {{\n"));
        string.push_str(&self.compile_block(&r#if.block)?);
        string.push_str("}\n");

        Ok(string)
    }

    /// Compiles a namespace declaration into C code.
    fn compile_namespace_declaration(
        &self,
        namespace_declaration: &NamespaceDeclaration,
        _span: Span,
    ) -> Result<String, CBackendError> {
        let mut string = String::new();

        for statement in &namespace_declaration.body {
            string.push_str(&self.compile_statement(statement)?);
        }

        Ok(string)
    }
}
