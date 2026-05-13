use crate::{
    ast::statement::{
        Statement,
        StatementKind,
        function_declaration::{
            DeclarationModifier,
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
    pub fn compile_statement(&mut self, statement: &Statement) -> Result<(), CBackendError> {
        match &statement.kind {
            StatementKind::FunctionDeclaration(function_declaration) => {
                self.compile_function_declaration(function_declaration, statement.span)
            }

            StatementKind::FunctionCall(function_call) => {
                let expression = self.compile_function_call(function_call, statement.span)?;
                self.writer.append(format!("{expression};",));

                Ok(())
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

            StatementKind::TypeDeclaration(_) | StatementKind::Import(_) => Ok(()),
        }
    }

    /// Compiles a function declaration into C code.
    pub fn compile_function_declaration(
        &mut self,
        function_declaration: &FunctionDeclaration,
        span: Span,
    ) -> Result<(), CBackendError> {
        if function_declaration.modifiers.contains(&DeclarationModifier::Extern) {
            trace!("Skipping generation of function '{}' as it is marked as external", function_declaration.name);
            return Ok(());
        }

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

        self.writer.append(format!("{return_type} {name}({parameters}) {{"));
        self.compile_block(&function_declaration.body)?;
        self.writer.append("}\n");

        Ok(())
    }

    /// Compiles a block into C code.
    fn compile_block(&mut self, block: &Vec<Statement>) -> Result<(), CBackendError> {
        self.writer.increase_indent();

        for statement in block {
            self.compile_statement(statement)?;
        }

        self.writer.decrease_indent();
        Ok(())
    }

    /// Compiles a function parameter into C code.
    pub fn compile_function_parameter(&self, function_parameter: &FunctionParameter) -> Result<String, CBackendError> {
        let name = function_parameter.name.clone();
        let r#type = self.compile_type(&function_parameter.r#type, function_parameter.span)?;
        Ok(format!("{type} {name}"))
    }

    /// Compiles a return statement into C code.
    pub fn compile_return(&mut self, r#return: &Return, _span: Span) -> Result<(), CBackendError> {
        match r#return.value.as_ref().map(|it| self.compile_expression(it)).transpose()? {
            Some(value) => self.writer.append(format!("return {};", value)),
            None => self.writer.append("return;"),
        };

        Ok(())
    }

    /// Compiles a variable declaration into C code.
    pub fn compile_variable_declaration(
        &mut self,
        variable_declaration: &VariableDeclaration,
        span: Span,
    ) -> Result<(), CBackendError> {
        let name = variable_declaration.name.clone();
        let r#type = self.compile_type(&variable_declaration.r#type, span)?;
        let value = self.compile_expression(&variable_declaration.value)?;

        self.writer.append(format!("{type} {name} = {value};"));
        Ok(())
    }

    /// Compiles a variable assignment into C code.
    pub fn compile_variable_assignment(
        &mut self,
        variable_assignment: &VariableAssignment,
        _span: Span,
    ) -> Result<(), CBackendError> {
        let target = self.compile_expression(&variable_assignment.target)?;
        let value = self.compile_expression(&variable_assignment.value)?;

        self.writer.append(format!("{target} = {value};"));
        Ok(())
    }

    /// Compiles an if statement into C code.
    pub fn compile_if(&mut self, r#if: &If, _span: Span) -> Result<(), CBackendError> {
        let condition = self.compile_expression(&r#if.condition)?;

        self.writer.append(format!("if ({condition}) {{"));
        self.compile_block(&r#if.block)?;
        self.writer.append("}");

        Ok(())
    }

    /// Compiles a namespace declaration into C code.
    fn compile_namespace_declaration(
        &mut self,
        namespace_declaration: &NamespaceDeclaration,
        _span: Span,
    ) -> Result<(), CBackendError> {
        for statement in &namespace_declaration.body {
            self.compile_statement(statement)?;
        }

        Ok(())
    }
}
