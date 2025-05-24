use inkwell::module::Linkage;

use super::{Codegen, error::CodegenError, r#type::TypeCodegen};
use crate::ast::node::{
    expression::FunctionCall,
    statement::{FunctionDefinition, Return, VariableDeclaration, VariableReassignment},
};

pub trait StatementCodegen {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> Result<(), CodegenError>;
}

impl StatementCodegen for FunctionDefinition {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> Result<(), CodegenError> {
        let mut param_types = vec![];
        for parameter in &self.parameters {
            param_types.push(parameter.expected_type.resolve_value_type(codegen).into());
        }

        // A function's type includes its return type, parameter types, and whether it is varadic.
        let function_type = match &self.return_type {
            Some(return_type) => return_type.resolve_fn_type(codegen, &param_types, false),
            None => codegen.llvm_context.void_type().fn_type(&param_types, false),
        };

        // We now know the type of the function, we can add it to the module.
        let linkage = if self.is_extern { Some(Linkage::External) } else { None };
        let function = codegen.llvm_module.add_function(&self.name, function_type, linkage);

        if !self.is_extern {
            // With the function created, we can create the entry block and start adding statements from its body.
            let block = codegen.llvm_context.append_basic_block(function, "entry");
            codegen.llvm_builder.position_at_end(block);

            codegen.context.start_function_scope();

            for (index, parameter) in function.get_param_iter().enumerate() {
                // We must first set the name of the function parameter.
                let function_parameter = self.parameters.get(index).unwrap();
                parameter.set_name(&function_parameter.name);

                let parameter_type = function_parameter.expected_type.resolve_value_type(codegen);
                if parameter_type.is_pointer_type() {
                    // If this is a pointer, we can just store that.
                    let scope = codegen.context.function_scope.as_mut().unwrap();

                    scope
                        .variables
                        .insert(function_parameter.name.clone(), parameter.into_pointer_value());
                } else {
                    // We can then allocate some space for it, and store the parameter into that space.
                    let pointer = codegen
                        .llvm_builder
                        .build_alloca(parameter_type, &self.name)
                        .map_err(|error| CodegenError::internal_error(error.to_string(), None))?;

                    codegen
                        .llvm_builder
                        .build_store(pointer, parameter)
                        .map_err(|error| CodegenError::internal_error(error.to_string(), None))?;

                    // We can then declare the parameter as a variable.
                    let scope = codegen.context.function_scope.as_mut().unwrap();
                    scope.variables.insert(function_parameter.name.clone(), pointer);
                }
            }

            codegen.visit_block(&self.body)?;
            codegen.context.end_function_scope();
        }

        Ok(())
    }
}

impl StatementCodegen for VariableDeclaration {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> Result<(), CodegenError> {
        // In order to declare a variable, we need to know its type.
        let variable_type = self.declared_type.resolve_value_type(codegen);

        // Now that we know the declared type, we can attempt to generate a value for
        // the variable's value expression.
        let value = codegen.visit_expression(&self.value)?;

        // We have all the required information, we can allocate space for this variable on the stack,
        // and then store the value into it.
        let pointer = codegen
            .llvm_builder
            .build_alloca(variable_type, &self.name)
            .map_err(|error| CodegenError::internal_error(error.to_string(), None))?;

        codegen
            .llvm_builder
            .build_store(pointer, value)
            .map_err(|error| CodegenError::internal_error(error.to_string(), None))?;

        #[rustfmt::skip]
        let function_scope = codegen.context.function_scope.as_mut()
            .ok_or(CodegenError::internal_error(
                "Unable to declare variable outside of a function block".to_owned(),
                Some(self.node.location),
            ))?;

        function_scope.variables.insert(self.name.clone(), pointer);
        Ok(())
    }
}

impl StatementCodegen for Return {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> Result<(), CodegenError> {
        // A return node can have an optional value associated with it.
        if let Some(value_node) = &self.value {
            let value = codegen.visit_expression(value_node)?;

            codegen
                .llvm_builder
                .build_return(Some(&value))
                .map(|_| ())
                .map_err(|error| CodegenError::internal_error(error.to_string(), None))
        } else {
            codegen
                .llvm_builder
                .build_return(None)
                .map(|_| ())
                .map_err(|error| CodegenError::internal_error(error.to_string(), None))
        }
    }
}

impl StatementCodegen for VariableReassignment {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> Result<(), CodegenError> {
        // We must be within a function's scope.
        let function_scope = codegen
            .context
            .function_scope
            .as_mut()
            .ok_or(CodegenError::internal_error(
                "Attempted to re-assign a variable outside of a function's scope!".to_owned(),
                Some(self.node.location),
            ))?;

        // The variable must be defined already.
        let pointer = *function_scope
            .variables
            .get(&self.name)
            .ok_or(CodegenError::internal_error(
                format!("Undefined variable: '{}', possible typechecker bug?", self.name),
                Some(self.node.location),
            ))?;

        // Now that we know where to store the value, we need to generate the value.
        let value = Codegen::visit_expression(codegen, &self.value)?;

        codegen
            .llvm_builder
            .build_store(pointer, value)
            .map_err(|error| CodegenError::internal_error(error.to_string(), None))?;

        Ok(())
    }
}

// TODO: This is really bad. Statements and expressions behave differently in terms of function calls,
//       but they are also really similar. I need to figure out a better way to do this.
impl StatementCodegen for FunctionCall {
    fn codegen<'ctx>(&self, codegen: &mut Codegen<'ctx>) -> Result<(), CodegenError> {
        let function = codegen
            .llvm_module
            .get_function(&self.name)
            .ok_or(CodegenError::internal_error(
                format!("Failed to find a function with the name '{}'", self.name),
                None,
            ))?;

        let mut arguments = vec![];
        for argument in &self.arguments {
            arguments.push(Codegen::visit_expression(codegen, argument)?.into());
        }

        codegen
            .llvm_builder
            .build_call(function, &arguments, &self.name)
            .map_err(|error| CodegenError::internal_error(error.to_string(), None))?;

        Ok(())
    }
}
