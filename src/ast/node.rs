use crate::core::location::Location;

#[derive(Debug, Clone)]
pub enum NodeKind {
    FunctionDefinition {
        name: String,
        return_type: Option<String>,
        location: Location,
    },
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
}

impl Node {
    pub fn function_definition(
        name: String,
        return_type: Option<String>,
        location: Location,
    ) -> Node {
        Node {
            kind: NodeKind::FunctionDefinition {
                name,
                return_type,
                location,
            },
        }
    }
}
