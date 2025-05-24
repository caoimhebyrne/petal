use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Integer(u8),
    Void,

    Reference(Box<TypeKind>),
    Unresolved(String),
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeKind::Reference(r#type) => write!(f, "&{}", r#type),
            TypeKind::Integer(size) => write!(f, "i{}", size),
            TypeKind::Void => write!(f, "void"),
            TypeKind::Unresolved(name) => write!(f, "{} (unresolved)", name),
        }
    }
}
