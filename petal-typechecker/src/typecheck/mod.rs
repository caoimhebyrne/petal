use petal_ast::r#type::Type;
use petal_core::{error::Result, source_span::SourceSpan};

use crate::Typechecker;

pub mod expression;
pub mod statement;

pub trait Typecheck {
    fn typecheck(&mut self, typechecker: &mut Typechecker, span: SourceSpan) -> Result<Type>;
}
