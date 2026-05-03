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
        optional_wrap::OptionalWrap,
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
};

impl CBackend {
    /// Compiles an expression into C code.
    pub fn compile_expression(&self, expression: &Expression) -> Result<String, CBackendError> {
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

            ExpressionKind::IdentifierReference(name) => CBackend::compile_identifier_reference(name, expression.span),

            ExpressionKind::OptionalWrap(inner) => self.compile_optional_wrap(inner, expression.span),
        }
    }

    /// Compiles a number literal expression into C code.
    pub fn compile_number_literal(value: &f64, _span: Span) -> Result<String, CBackendError> {
        Ok(value.to_string())
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
    fn compile_reference(&self, value: &Expression, _span: Span) -> Result<String, CBackendError> {
        Ok(format!("&({})", self.compile_expression(value)?))
    }

    /// Compiles a dereference expression into C code.
    fn compile_dereference(&self, value: &Expression, _span: Span) -> Result<String, CBackendError> {
        Ok(format!("*({})", self.compile_expression(value)?))
    }

    /// Compiles a function call expression into C code.
    pub fn compile_function_call(&self, function_call: &FunctionCall, span: Span) -> Result<String, CBackendError> {
        let function_id =
            function_call.resolved_callee.as_ref().ok_or(CBackendErrorKind::MissingFunctionId.at(span))?;

        let function =
            self.functions.get(function_id).ok_or(CBackendErrorKind::MissingFunction(*function_id).at(span))?;

        debug!("Function ID '{function_id}' resolves to function named '{}'", function.name);

        let arguments = &function_call
            .arguments
            .iter()
            .map(|it| self.compile_expression(&it.value))
            .collect::<Result<Vec<String>, CBackendError>>()?
            .join(", ");

        Ok(format!("{}({arguments})", function.name))
    }

    /// Compiles a binary operation expression into C code.
    pub fn compile_binary_operation(
        &self,
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
        &self,
        structure_initialization: &StructureInitialization,
        span: Span,
    ) -> Result<String, CBackendError> {
        // The typechecker should have patched in a structure ID. This lets us know the exact type of the structure
        // that is being initialized.
        let structure_id =
            structure_initialization.structure_id.ok_or(CBackendErrorKind::MissingStructureId.at(span))?;

        // A corresponding type must have been declared already.
        let structure_type =
            self.structures.get(&structure_id).ok_or(CBackendErrorKind::MissingStructure(structure_id).at(span))?;

        let fields = structure_initialization
            .fields
            .iter()
            .map(|it| -> Result<String, CBackendError> {
                let value = self.compile_expression(&it.value)?;
                Ok(format!(".{} = {}", it.name, value))
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");

        Ok(format!("({}) {{ {fields} }}", structure_type.name))
    }

    /// Compiles a member access expression into C code.
    pub fn compile_member_access(&self, member_access: &MemberAccess, _span: Span) -> Result<String, CBackendError> {
        let target = self.compile_expression(&member_access.target)?;
        Ok(format!("({target}).{}", member_access.name))
    }

    /// Compiles an optional wrapping expression into C code.
    pub fn compile_optional_wrap(&self, optional_wrap: &OptionalWrap, _span: Span) -> Result<String, CBackendError> {
        let inner_value = self.compile_expression(&optional_wrap.inner_value)?;
        Ok(format!("(Optional_{}) {{ .has_value = true, .value = {inner_value} }}", optional_wrap.inner_type))
    }
}
