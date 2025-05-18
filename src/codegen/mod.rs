use std::path::Path;

use crate::{
    ast::node::{Node, NodeKind},
    core::location::Location,
};
use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    module::Module,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
};

pub struct Codegen<'a> {
    nodes: &'a Vec<Node>,

    context: &'a Context,
    module: Module<'a>,
    builder: Builder<'a>,
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

    pub fn compile(&mut self) {
        self.visit_block(&self.nodes);

        match self.module.verify() {
            Err(message) => println!("Failed to verify generated module: '{:}'", message),
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
            FileType::Assembly,
            Path::new("./build/00_hello_world"),
        ) {
            Ok(_) => return,
            Err(error) => println!("error: '{}'", error),
        }
    }
}

impl<'a> Codegen<'a> {
    pub fn visit_block(&mut self, block: &Vec<Node>) {
        for node in block {
            self.visit_statement(node.to_owned());
        }
    }

    pub fn visit_statement(&mut self, node: Node) {
        match node.kind {
            NodeKind::FunctionDefinition {
                name,
                return_type,
                body,
            } => self.visit_function_declaration(name, return_type, body, node.location),

            NodeKind::ReturnStatement { value } => self.visit_return_statement(value),

            _ => {}
        }
    }

    pub fn visit_function_declaration(
        &mut self,
        name: String,
        _return_type: Option<String>,
        body: Vec<Node>,
        _location: Location,
    ) {
        // TODO: return_type

        let function_type = self.context.void_type().fn_type(&[], false);
        let function = self.module.add_function(&name, function_type, None);

        let block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(block);

        for node in body {
            self.visit_statement(node);
        }
    }

    pub fn visit_return_statement(&mut self, _value: Option<Box<Node>>) {
        // TODO: use _value
        self.builder.build_return(None).unwrap();
    }
}
