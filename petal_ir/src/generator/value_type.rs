use crate::value::ValueType;
use petal_core::typechecker::r#type::{Type, kind::TypeKind};

impl From<Type> for ValueType {
    fn from(value: Type) -> Self {
        match value.kind {
            TypeKind::Integer(width) => ValueType::Integer { width },
            TypeKind::Reference(_) => ValueType::Reference,
            _ => todo!(),
        }
    }
}
