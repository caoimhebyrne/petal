use petal_ast::r#type::Type;
use petal_core::error::Result;

use crate::Typechecker;

pub mod expression;
pub mod statement;

pub trait Typecheck {
    fn typecheck(&mut self, typechecker: &mut Typechecker) -> Result<Type>;
}
