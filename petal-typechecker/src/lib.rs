use petal_ast::{
    statement::{Statement, StatementKind, function_declaration::FunctionDeclaration},
    r#type::{ResolvedTypeKind, Type, TypeKind},
    visitor::ASTVisitor,
};
use petal_core::{error::Result, string_intern::StringInternPool};

use crate::{error::TypecheckerErrorKind, r#trait::Typecheck};

pub mod error;
pub mod r#trait;

/// Responsible for resolving all types within an AST.
pub struct Typechecker<'a> {
    /// The StringInternPool implementation to read identifiers from.
    string_intern_pool: &'a dyn StringInternPool,
}

impl<'a> Typechecker<'a> {
    /// Creates a new [Typechecker].
    pub fn new(string_intern_pool: &'a dyn StringInternPool) -> Self {
        Typechecker { string_intern_pool }
    }

    /// Resolves the provided type if it has not yet been resolved.
    pub fn resolve(&self, r#type: &mut Type) -> Result<Type> {
        // If the provided type is resolved, we do not need to do anything else.
        let type_name_reference = match r#type.kind {
            TypeKind::Resolved(_) => return Ok(*r#type),
            TypeKind::Unresolved(reference) => reference,
        };

        // Otherwise, we must attempt to resolve it.
        let type_name = self.string_intern_pool.resolve_reference(&type_name_reference).ok_or(
            TypecheckerErrorKind::unresolvable_string_reference(type_name_reference, r#type.span),
        )?;

        let resolved_type_kind = match type_name {
            "void" => ResolvedTypeKind::Void,
            "i32" => ResolvedTypeKind::Integer(32),

            _ => return TypecheckerErrorKind::unresolvable_type(type_name, r#type.span).into(),
        };

        r#type.resolve(resolved_type_kind);
        Ok(*r#type)
    }
}

impl<'a> ASTVisitor for Typechecker<'a> {
    fn visit(&self, statement: &mut Statement) -> Result<()> {
        match &mut statement.kind {
            StatementKind::FunctionDeclaration(declaration) => declaration.typecheck(self)?,

            _ => return TypecheckerErrorKind::unsupported_statement(statement).into(),
        };

        Ok(())
    }
}

impl Typecheck for FunctionDeclaration {
    fn typecheck(&mut self, typechecker: &Typechecker) -> Result<Type> {
        typechecker.resolve(&mut self.return_type)
    }
}

#[cfg(test)]
mod typechecker_tests {
    use petal_core::{source_span::SourceSpan, string_intern::StringInternPoolImpl};

    use super::*;

    #[test]
    fn can_resolve_void() {
        let mut string_intern_pool = StringInternPoolImpl::new();
        let void_string_reference = string_intern_pool.intern("void");

        let mut r#type = Type {
            kind: TypeKind::Unresolved(void_string_reference),
            span: SourceSpan { start: 0, end: 0 },
        };

        let typechecker = Typechecker::new(&string_intern_pool);
        typechecker.resolve(&mut r#type).expect("resolve should succeed");

        assert_eq!(r#type.kind, TypeKind::Resolved(ResolvedTypeKind::Void))
    }

    #[test]
    fn can_resolve_i32() {
        let mut string_intern_pool = StringInternPoolImpl::new();
        let i32_string_reference = string_intern_pool.intern("i32");

        let mut r#type = Type {
            kind: TypeKind::Unresolved(i32_string_reference),
            span: SourceSpan { start: 0, end: 0 },
        };

        let typechecker = Typechecker::new(&string_intern_pool);
        typechecker.resolve(&mut r#type).expect("resolve should succeed");

        assert_eq!(r#type.kind, TypeKind::Resolved(ResolvedTypeKind::Integer(32)))
    }
}
