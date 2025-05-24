use crate::ast::node::{expression::Expression, statement::Statement};
use context::CodegenContext;
use error::CodegenError;
use expression::ExpressionCodegen;
use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    module::Module,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    values::BasicValueEnum,
};
use statement::StatementCodegen;
use std::{path::PathBuf, process::Command};

pub mod context;
pub mod error;
pub mod expression;
pub mod statement;
pub mod r#type;

pub struct Codegen<'a> {
    nodes: &'a Vec<Statement>,
    output_path: &'a PathBuf,

    pub context: CodegenContext<'a>,

    pub(crate) llvm_context: &'a Context,
    pub(crate) llvm_module: Module<'a>,
    pub(crate) llvm_builder: Builder<'a>,
}

impl<'a> Codegen<'a> {
    pub fn new(output_path: &'a PathBuf, context: &'a Context, nodes: &'a Vec<Statement>) -> Self {
        Self {
            nodes,
            output_path,
            context: CodegenContext::new(),
            llvm_context: context,
            llvm_module: context.create_module(
                &output_path
                    .file_prefix()
                    .expect("Failed to get filename from output path")
                    .to_string_lossy(),
            ),
            llvm_builder: context.create_builder(),
        }
    }

    pub fn compile(&mut self) -> Result<(), CodegenError> {
        self.visit_block(self.nodes)?;

        self.llvm_module
            .verify()
            .map_err(|error| CodegenError::verification_error(error.to_string(), None))?;

        Target::initialize_all(&InitializationConfig::default());

        let target_triple = TargetMachine::get_default_triple();

        let cpu = TargetMachine::get_host_cpu_name();

        let target = Target::from_triple(&target_triple)
            .map_err(|error| CodegenError::internal_error(error.to_string(), None))?;

        let target_machine = target
            .create_target_machine(
                &target_triple,
                cpu.to_str()
                    .map_err(|error| CodegenError::internal_error(error.to_string(), None))?,
                "",
                OptimizationLevel::None,
                RelocMode::PIC,
                CodeModel::Default,
            )
            .ok_or(CodegenError::internal_error(
                "Failed to create LLVM target machine".to_owned(),
                None,
            ))?;

        let object_file = self.output_path.with_added_extension("o");

        target_machine
            .write_to_file(&self.llvm_module, FileType::Object, &object_file)
            .map_err(|error| CodegenError::internal_error(error.to_string(), None))?;

        Command::new("clang")
            .args([
                "-fuse-ld=lld",
                "-o",
                &self.output_path.to_string_lossy(),
                &object_file.to_string_lossy(),
            ])
            .spawn()
            .map(|_| {})
            .map_err(|error| CodegenError::internal_error(error.to_string(), None))
    }

    pub fn visit_block(&mut self, block: &Vec<Statement>) -> Result<(), CodegenError> {
        for node in block {
            self.visit_statement(node)?;
        }

        Ok(())
    }

    pub fn visit_expression(&mut self, expression: &Expression) -> Result<BasicValueEnum<'a>, CodegenError> {
        match expression {
            Expression::IntegerLiteral(integer_literal) => integer_literal.codegen(self),
            Expression::IdentifierReference(identifier_reference) => identifier_reference.codegen(self),
            Expression::BinaryOperation(binary_operation) => binary_operation.codegen(self),
            Expression::FunctionCall(function_call) => function_call.codegen(self),
        }
    }

    pub fn visit_statement(&mut self, statement: &Statement) -> Result<(), CodegenError> {
        match statement {
            Statement::FunctionDefinition(function_definition) => function_definition.codegen(self),
            Statement::VariableDeclaration(variable_declaration) => variable_declaration.codegen(self),
            Statement::Return(r#return) => r#return.codegen(self),
        }
    }
}
