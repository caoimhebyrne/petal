use crate::{
    ast::expression::{
        Expression,
        ExpressionKind,
        binary_operation::{
            BinaryOperation,
            BinaryOperator,
        },
        function_call::FunctionCall,
        member_access::MemberAccess,
        optional_wrap::{
            OptionalEmpty,
            OptionalForceUnwrap,
            OptionalHasValue,
            OptionalUnwrap,
            OptionalWrap,
        },
        structure_initialization::StructureInitialization,
    },
    backend::c::{
        CBackend,
        error::{
            CBackendError,
            CBackendErrorKind,
        },
    },
    core::span::Span,
    typechecker::r#type::StructureReference,
};

impl CBackend {
    /// Compiles an expression into C code.
    pub fn compile_expression(&mut self, expression: &Expression) -> Result<String, CBackendError> {
        match &expression.kind {
            ExpressionKind::FunctionCall(function_call) => self.compile_function_call(function_call, expression.span),

            ExpressionKind::BinaryOperation(binary_operation) => {
                self.compile_binary_operation(binary_operation, expression.span)
            }

            ExpressionKind::StructureInitialization(fields) => {
                self.compile_structure_initialization(fields, expression.span)
            }

            ExpressionKind::MemberAccess(member_access) => self.compile_member_access(member_access, expression.span),

            ExpressionKind::Reference(inner) => self.compile_reference(inner, expression.span),

            ExpressionKind::Dereference(inner) => self.compile_dereference(inner, expression.span),

            ExpressionKind::BooleanLiteral(value) => CBackend::compile_boolean_literal(value, expression.span),

            ExpressionKind::NumberLiteral(value) => CBackend::compile_number_literal(value, expression.span),

            ExpressionKind::StringLiteral(value) => self.compile_string_literal(value, expression.span),

            ExpressionKind::IdentifierReference(name) => CBackend::compile_identifier_reference(name, expression.span),

            ExpressionKind::OptionalEmpty(optional_empty) => {
                self.compile_optional_empty(optional_empty, expression.span)
            }

            ExpressionKind::OptionalWrap(optional_wrap) => self.compile_optional_wrap(optional_wrap, expression.span),

            ExpressionKind::OptionalHasValue(optional_has_value) => {
                self.compile_optional_has_value(optional_has_value, expression.span)
            }

            ExpressionKind::OptionalForceUnwrap(optional_force_unwrap) => {
                self.compile_optional_force_unwrap(optional_force_unwrap, expression.span)
            }

            ExpressionKind::OptionalUnwrap(optional_unwrap) => {
                self.compile_optional_unwrap(optional_unwrap, expression.span)
            }

            ExpressionKind::NamespaceQualifier(_) => Ok("".into()),
        }
    }

    /// Compiles a number literal expression into C code.
    pub fn compile_number_literal(value: &f64, _span: Span) -> Result<String, CBackendError> {
        Ok(value.to_string())
    }

    /// Compiles a string literal expression into C code.
    pub fn compile_string_literal(&self, value: &str, _span: Span) -> Result<String, CBackendError> {
        // Sequences like `\n` will be treated as their literal value by the C compiler, as they are not escaped.
        // `value.len()` would return `2`, whereas `decoded_byte_length` would equal `1` in this case.
        let mut string_bytes = value.bytes();
        let mut decoded_byte_length = 0;
        while let Some(byte) = string_bytes.next() {
            if byte == b'\\' {
                string_bytes.next();
            }

            decoded_byte_length += 1;
        }

        let structure_name =
            self.get_name_for_structure_reference(&StructureReference::Plain(self.builtin_types.compile_time_str))?;

        Ok(format!("({}){{ .data = (uint8_t*) \"{}\", .length = {} }}", structure_name, value, decoded_byte_length))
    }

    /// Compiles a boolean literal expression into C code.
    pub fn compile_boolean_literal(value: &bool, _span: Span) -> Result<String, CBackendError> {
        Ok(value.to_string())
    }

    /// Compiles an identifier reference expression into C code.
    pub fn compile_identifier_reference(value: &String, _span: Span) -> Result<String, CBackendError> {
        Ok(value.to_string())
    }

    /// Compiles a reference expression into C code.
    fn compile_reference(&mut self, value: &Expression, _span: Span) -> Result<String, CBackendError> {
        Ok(format!("&({})", self.compile_expression(value)?))
    }

    /// Compiles a dereference expression into C code.
    fn compile_dereference(&mut self, value: &Expression, _span: Span) -> Result<String, CBackendError> {
        Ok(format!("*({})", self.compile_expression(value)?))
    }

    /// Compiles a function call expression into C code.
    pub fn compile_function_call(&mut self, function_call: &FunctionCall, span: Span) -> Result<String, CBackendError> {
        let function_id =
            function_call.resolved_callee.as_ref().ok_or(CBackendErrorKind::MissingFunctionId.at(span))?;

        let function_name = self.function_name(function_id)?;

        debug!("Function ID '{function_id}' resolves to function named '{}'", function_name);

        let arguments = &function_call
            .arguments
            .iter()
            .map(|it| self.compile_expression(&it.value))
            .collect::<Result<Vec<String>, CBackendError>>()?
            .join(", ");

        Ok(format!("{function_name}({arguments})"))
    }

    /// Compiles a binary operation expression into C code.
    pub fn compile_binary_operation(
        &mut self,
        binary_operation: &BinaryOperation,
        _span: Span,
    ) -> Result<String, CBackendError> {
        let left = self.compile_expression(&binary_operation.left)?;
        let right = self.compile_expression(&binary_operation.right)?;

        let operand = match binary_operation.operator {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Equals => "==",
            BinaryOperator::NotEquals => "!=",
        };

        Ok(format!("{left} {operand} {right}"))
    }

    /// Compiles a structure initialization into C code.
    pub fn compile_structure_initialization(
        &mut self,
        structure_initialization: &StructureInitialization,
        span: Span,
    ) -> Result<String, CBackendError> {
        // The typechecker should have patched in a structure ID. This lets us know the exact type of the structure
        // that is being initialized.
        let structure_reference =
            structure_initialization.structure_reference.ok_or(CBackendErrorKind::MissingStructureId.at(span))?;

        let fields = structure_initialization
            .fields
            .iter()
            .map(|it| -> Result<String, CBackendError> {
                let value = self.compile_expression(&it.value)?;
                Ok(format!(".{} = {}", it.name, value))
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");

        let structure_name = self.get_name_for_structure_reference(&structure_reference)?;
        Ok(format!("({structure_name}) {{ {fields} }}"))
    }

    /// Compiles a member access expression into C code.
    pub fn compile_member_access(
        &mut self,
        member_access: &MemberAccess,
        _span: Span,
    ) -> Result<String, CBackendError> {
        let target = self.compile_expression(&member_access.target)?;
        Ok(format!("({target}).{}", member_access.name))
    }

    /// Compiles an optional wrapping expression into C code.
    pub fn compile_optional_wrap(
        &mut self,
        optional_wrap: &OptionalWrap,
        _span: Span,
    ) -> Result<String, CBackendError> {
        let inner_value = self.compile_expression(&optional_wrap.inner_value)?;
        Ok(format!("(Optional_{}) {{ .has_value = true, .value = {inner_value} }}", optional_wrap.inner_type))
    }

    /// Compiles an optional empty expression into C code.
    pub fn compile_optional_empty(
        &mut self,
        optional_empty: &OptionalEmpty,
        _span: Span,
    ) -> Result<String, CBackendError> {
        Ok(format!("(Optional_{}) {{0}}", optional_empty.inner_type))
    }

    /// Compiles an optional has value expression into C code.
    pub fn compile_optional_has_value(
        &mut self,
        optional_has_value: &OptionalHasValue,
        _span: Span,
    ) -> Result<String, CBackendError> {
        Ok(format!("({}).has_value", self.compile_expression(&optional_has_value.optional_value)?))
    }

    pub fn compile_optional_force_unwrap(
        &mut self,
        optional_force_unwrap: &OptionalForceUnwrap,
        _span: Span,
    ) -> Result<String, CBackendError> {
        let optional_value = self.compile_expression(&optional_force_unwrap.optional_value)?;

        // FIXME: This should be enforced at a different level. If an unwrap expression is encountered, an if
        //        statement similar to this should be generated at an AST/IR level.
        self.writer.append(format!("if (!({optional_value}).has_value) {{"));
        self.writer.increase_indent();
        self.writer.append(format!(r#"__ptl_internal_fn_panic("Optional of type '{optional_value}' had no value");"#));
        self.writer.decrease_indent();
        self.writer.append("}");

        Ok(format!("({optional_value}).value"))
    }

    pub fn compile_optional_unwrap(
        &mut self,
        optional_unwrap: &OptionalUnwrap,
        _span: Span,
    ) -> Result<String, CBackendError> {
        let optional_value = self.compile_expression(&optional_unwrap.optional_value)?;
        Ok(format!("({optional_value}).value"))
    }
}
