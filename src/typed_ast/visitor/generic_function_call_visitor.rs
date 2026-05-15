use crate::{
    ast::expression::binary_operation::BinaryOperator,
    typed_ast::{
        Expression,
        FunctionKey,
        Program,
        r#type::Ty,
        visitor::ProgramVisitor,
    },
};

pub struct GenericFunctionCallVisitor {
    program: Program,
}

impl GenericFunctionCallVisitor {
    /// Creates a new [`GenericFunctionCallVisitor`] with the provided [`Program`].
    pub fn new(program: Program) -> Self {
        Self { program }
    }
}

impl ProgramVisitor for GenericFunctionCallVisitor {
    fn visit_expression_function_call(
        &mut self,
        function_key: &FunctionKey,
        _arguments: &mut [Expression],
        ty: &mut Ty,
    ) {
        if !matches!(ty, Ty::Generic(_)) {
            return;
        }

        let function = &self.program.functions[function_key];

        if let Ty::Generic(_) = function.return_ty {
            panic!(
                "Function named '{}' ({function_key:?}) still has a generic return type! (ty = {:?})",
                function.name, function.return_ty
            )
        }

        trace!(
            "Call to function named '{}' ({function_key:?}) still has a generic return type, attempting to resolve it",
            function.name
        );

        *ty = function.return_ty;
    }

    fn visit_statement_variable_declaration(&mut self, _name: &str, value: &mut Expression, _ty: &mut Ty) {
        self.visit_expression(value);
    }

    fn visit_statement_return(&mut self, value: Option<&mut Expression>) {
        if let Some(value) = value {
            self.visit_expression(value);
        }
    }

    fn visit_expression_binary_operation(
        &mut self,
        left: &mut Expression,
        right: &mut Expression,
        _operator: &mut BinaryOperator,
        _ty: &mut Ty,
    ) {
        self.visit_expression(left);
        self.visit_expression(right);
    }

    fn visit_expression_number_literal(&mut self, _value: &mut f64, _ty: &mut Ty) {}

    fn visit_variable_reference(&mut self, _variable_name: &mut str, _ty: &mut Ty) {}
}
