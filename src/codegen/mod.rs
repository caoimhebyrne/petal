use crate::ast::node::{Node, kind::NodeKind};
use expression::ExpressionCodegen;
use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    module::Module,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    types::BasicTypeEnum,
    values::BasicValueEnum,
};
use statement::StatementCodegen;
use std::path::Path;

pub mod expression;
pub mod statement;

pub struct Codegen<'a> {
    nodes: &'a Vec<Node>,

    pub(crate) context: &'a Context,
    pub(crate) module: Module<'a>,
    pub(crate) builder: Builder<'a>,
}

impl<'a> Codegen<'a> {
    pub fn new(name: &'a str, context: &'a Context, nodes: &'a Vec<Node>) -> Codegen<'a> {
        Codegen {
            nodes: nodes,
            context: context,
            module: context.create_module(name),
            builder: context.create_builder(),
        }
    }

    pub fn compile(&self) {
        self.visit_block(self.nodes);

        match self.module.verify() {
            Err(message) => println!(
                "Failed to verify generated module:\n{:}",
                message.to_str().unwrap()
            ),
            _ => {}
        }

        Target::initialize_all(&InitializationConfig::default());

        let target_triple = TargetMachine::get_default_triple();
        let cpu = TargetMachine::get_host_cpu_name();
        let target = Target::from_triple(&target_triple).expect("Could not get target from triple");

        let target_machine = target
            .create_target_machine(
                &target_triple,
                cpu.to_str().unwrap(),
                "",
                OptimizationLevel::None,
                RelocMode::PIC,
                CodeModel::Default,
            )
            .unwrap();

        match target_machine.write_to_file(
            &self.module,
            FileType::Object,
            Path::new("./build/00_hello_world.o"),
        ) {
            Ok(_) => return,
            Err(error) => println!("Failed to generate object:\n{}", error.to_str().unwrap()),
        }
    }

    pub fn visit_block(&self, block: &Vec<Node>) {
        for node in block {
            self.visit_statement(node);
        }
    }

    pub fn visit_expression(
        &self,
        expression: &Node,
        expected_type: Option<BasicTypeEnum<'a>>,
    ) -> BasicValueEnum<'a> {
        match &expression.kind {
            NodeKind::IntegerLiteral(integer_literal) => {
                integer_literal.codegen(self, expected_type)
            }

            _ => panic!("Unsupported expression node type: {:#?}", expression.kind),
        }
    }

    pub fn visit_statement(&self, statement: &Node) {
        match &statement.kind {
            NodeKind::FunctionDefinition(function_definition) => function_definition.codegen(self),
            NodeKind::VariableDeclaration(variable_declaration) => {
                variable_declaration.codegen(self)
            }
            NodeKind::Return(r#return) => r#return.codegen(self),

            _ => panic!("Unsupported statement node type: {:#?}", statement.kind),
        }
    }
}
