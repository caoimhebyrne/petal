use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    I32,
    Void,

    Unresolved(String),
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeKind::I32 => write!(f, "i32"),
            TypeKind::Void => write!(f, "void"),
            TypeKind::Unresolved(name) => write!(f, "{} (unresolved)", name),
        }
    }
}
