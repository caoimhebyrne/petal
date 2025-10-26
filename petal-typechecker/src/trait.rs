use petal_ast::r#type::Type;
use petal_core::error::Result;

use crate::Typechecker;

pub trait Typecheck {
    fn typecheck(&mut self, typechecker: &Typechecker) -> Result<Type>;
}
