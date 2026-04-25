use crate::{
    ast::r#type::Type,
    backend::c::error::{
        CBackendError,
        CBackendErrorKind,
    },
    core::span::Span,
    module::ParsedModule,
};

pub mod error;
pub mod expression;
pub mod statement;

/// The C codegen backend.
pub struct CBackend;

impl CBackend {
    /// Compiles a [ParsedModule] to C code.
    pub fn compile(module: &ParsedModule) -> Result<String, CBackendError> {
        let mut code = String::new();

        code.push_str("#include <stdint.h>\n\n");

        for statement in &module.ast {
            code.push_str(&CBackend::compile_statement(statement)?);
        }

        Ok(code)
    }

    /// Converts a [Type] into a C type.
    fn compile_type(r#type: &Type, span: Span) -> Result<String, CBackendError> {
        let Type::Named(name) = r#type;

        let value: String = match name.as_str() {
            "i32" => "int32_t".into(),
            _ => return Err(CBackendErrorKind::UnsupportedType(name.clone()).at(span)),
        };

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        ast::statement::{
            Statement,
            StatementKind,
            function_declaration::FunctionDeclaration,
        },
        core::span::Span,
    };

    fn assert_compiles(kinds: Vec<StatementKind>, compiled: &str) {
        let statements = kinds.into_iter().map(|kind| Statement::from(kind, Span::default())).collect();
        assert_eq!(CBackend::compile(&ParsedModule::new(statements)), Ok(compiled.into()))
    }

    #[test]
    fn compile_empty_function() {
        assert_compiles(
            vec![FunctionDeclaration::new("foo".into(), vec![], vec![], None).into()],
            "#include <stdint.h>\n\nvoid foo(void) {\n}\n",
        );
    }

    #[test]
    fn compile_empty_function_i32_return_type() {
        assert_compiles(
            vec![FunctionDeclaration::new("foo".into(), vec![], vec![], Some(Type::Named("i32".into()))).into()],
            "#include <stdint.h>\n\nint32_t foo(void) {\n}\n",
        );
    }
}
