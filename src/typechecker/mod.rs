use statement::StatmentTypecheck;
use r#type::{Type, kind::TypeKind};

use crate::ast::node::{Node, kind::NodeKind};

pub mod statement;
pub mod r#type;

pub struct Typechecker<'a> {
    nodes: &'a mut Vec<Node>,
}

impl<'a> Typechecker<'a> {
    pub fn new(nodes: &'a mut Vec<Node>) -> Typechecker<'a> {
        return Typechecker { nodes };
    }

    pub fn check(&mut self) {
        Typechecker::check_block(&mut self.nodes);
    }

    pub fn check_block(block: &mut Vec<Node>) {
        for node in block {
            match &mut node.kind {
                NodeKind::VariableDeclaration(variable_declaration) => {
                    variable_declaration.resolve();
                }

                NodeKind::FunctionDefinition(function_definition) => {
                    function_definition.resolve();
                }

                NodeKind::Return(r#return) => {
                    r#return.resolve();
                }

                _ => todo!(),
            }
        }
    }

    pub fn resolve_type(r#type: Type) -> Type {
        let name = match r#type.kind {
            TypeKind::Unresolved(name) => name,
            _ => return r#type,
        };

        let resolved_kind = match name.as_str() {
            "i32" => TypeKind::I32,
            _ => panic!("Unable to resolve type: '{}'", name),
        };

        Type::new(resolved_kind, r#type.location)
    }
}
