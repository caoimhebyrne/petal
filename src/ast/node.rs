use crate::core::location::Location;

#[derive(Debug, Clone)]
pub enum NodeKind {
    IntegerLiteral(u64),

    FunctionDefinition {
        name: String,
        return_type: Option<String>,
        body: Vec<Node>,
    },

    VariableDeclaration {
        name: String,
        r#type: String,
        value: Box<Node>,
    },
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub location: Location,
}

impl Node {
    pub fn integer_literal(value: u64, location: Location) -> Node {
        Node {
            kind: NodeKind::IntegerLiteral(value),
            location,
        }
    }

    pub fn function_definition(
        name: String,
        return_type: Option<String>,
        body: Vec<Node>,
        location: Location,
    ) -> Node {
        Node {
            kind: NodeKind::FunctionDefinition {
                name,
                return_type,
                body,
            },
            location,
        }
    }

    pub fn variable_declaration(
        name: String,
        r#type: String,
        value: Box<Node>,
        location: Location,
    ) -> Node {
        Node {
            kind: NodeKind::VariableDeclaration {
                name,
                r#type,
                value,
            },
            location,
        }
    }
}
