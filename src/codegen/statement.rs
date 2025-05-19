use super::{Codegen, r#type::TypeCodegen};
use crate::ast::node::kind::{FunctionDefinitionNode, ReturnNode, VariableDeclarationNode};

pub trait StatementCodegen {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>);
}

impl StatementCodegen for FunctionDefinitionNode {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) {
        // TODO: Handle a function's parameters.
        let param_types = vec![];

        // A function's type includes its return type, parameter types, and whether it is varadic.
        let function_type = match &self.return_type {
            Some(return_type) => return_type.resolve_fn_type(codegen, &param_types, false),
            None => codegen
                .llvm_context
                .void_type()
                .fn_type(&param_types, false),
        };

        // We now know the type of the function, we can add it to the module.
        let function = codegen
            .llvm_module
            .add_function(&self.name, function_type, None);

        // With the function created, we can create the entry block and start adding statements from its body.
        let block = codegen.llvm_context.append_basic_block(function, "entry");
        codegen.llvm_builder.position_at_end(block);

        codegen.context.start_function_scope();
        codegen.visit_block(&self.body);
        codegen.context.end_function_scope();
    }
}

impl StatementCodegen for VariableDeclarationNode {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) {
        // In order to declare a variable, we need to know its type.
        let variable_type = self.declared_type.resolve_value_type(codegen);

        // Now that we know the declared type, we can attempt to generate a value for
        // the variable's value expression.
        let value = codegen.visit_expression(&self.value);

        // We have all the required information, we can allocate space for this variable on the stack,
        // and then store the value into it.
        let pointer = codegen
            .llvm_builder
            .build_alloca(variable_type, &self.name)
            .expect("Failed to build alloca");

        codegen
            .llvm_builder
            .build_store(pointer, value)
            .expect("Failed to build store");

        let function_scope = match codegen.context.function_scope.as_mut() {
            Some(value) => value,
            None => panic!("Identifier reference outside of function scope?"),
        };

        function_scope.variables.insert(self.name.clone(), pointer);
    }
}

impl StatementCodegen for ReturnNode {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) {
        // A return node can have an optional value associated with it.
        if let Some(value_node) = &self.value {
            let value = codegen.visit_expression(&*value_node);

            codegen
                .llvm_builder
                .build_return(Some(&value))
                .expect("Failed to build return statement with value");
        } else {
            codegen
                .llvm_builder
                .build_return(None)
                .expect("Failed to build return statement without value");
        }
    }
}
