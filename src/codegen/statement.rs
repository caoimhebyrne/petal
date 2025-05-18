use inkwell::types::BasicType;

use super::Codegen;
use crate::ast::node::kind::{FunctionDefinitionNode, ReturnNode, VariableDeclarationNode};

pub trait StatementCodegen {
    fn codegen<'ctx>(&self, codegen: &Codegen<'ctx>);
}

impl StatementCodegen for FunctionDefinitionNode {
    fn codegen<'ctx>(&self, codegen: &Codegen<'ctx>) {
        // TODO: Handle a function's parameters.
        let param_types = vec![];

        // A function's type includes its return type, parameter types, and whether it is varadic.
        // TODO: A more "generic" type system with `to_function_type`, etc.
        let function_type = match &self.return_type {
            Some(value) => match value.as_str() {
                "i32" => codegen.context.i32_type().fn_type(&param_types, false),

                _ => panic!("Unable to use type '{}' as function return type!", value),
            },

            None => codegen.context.void_type().fn_type(&param_types, false),
        };

        // We now know the type of the function, we can add it to the module.
        let function = codegen.module.add_function(&self.name, function_type, None);

        // With the function created, we can create the entry block and start adding statements from its body.
        let block = codegen.context.append_basic_block(function, "entry");
        codegen.builder.position_at_end(block);
        codegen.visit_block(&self.body);
    }
}

impl StatementCodegen for VariableDeclarationNode {
    fn codegen<'ctx>(&self, codegen: &Codegen<'ctx>) {
        // In order to declare a variable, we need to know its type.
        let variable_type = match self.declared_type.as_str() {
            "i32" => codegen.context.i32_type().as_basic_type_enum(),

            _ => panic!(
                "Unable to use type '{}' as variable type",
                self.declared_type
            ),
        };

        // Now that we know the declared type, we can attempt to generate a value for
        // the variable's value expression.
        let value = codegen.visit_expression(&self.value, Some(variable_type));

        // We have all the required information, we can allocate space for this variable on the stack,
        // and then store the value into it.
        let pointer = codegen
            .builder
            .build_alloca(variable_type, &self.name)
            .expect("Failed to build alloca");

        codegen
            .builder
            .build_store(pointer, value)
            .expect("Failed to build store");
    }
}

impl StatementCodegen for ReturnNode {
    fn codegen<'ctx>(&self, codegen: &Codegen<'ctx>) {
        // A return node can have an optional value associated with it.
        if let Some(value_node) = &self.value {
            // TODO: To aid type inference, we should pass the function's return type.
            //       Although, since this is just a workaround for the fact we don't have a typechecker/resolver,
            //       it's not that important in the long run.
            let value = codegen.visit_expression(&*value_node, None);

            codegen
                .builder
                .build_return(Some(&value))
                .expect("Failed to build return statement with value");
        } else {
            codegen
                .builder
                .build_return(None)
                .expect("Failed to build return statement without value");
        }
    }
}
