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
            "i8" => "int8_t".into(),
            "i16" => "int16_t".into(),
            "i32" => "int32_t".into(),
            "i64" => "int64_t".into(),

            "u8" => "uint8_t".into(),
            "u16" => "uint16_t".into(),
            "u32" => "uint32_t".into(),
            "u64" => "uint64_t".into(),

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
        ast::{
            expression::{
                Expression,
                ExpressionKind,
            },
            statement::{
                Statement,
                StatementKind,
                function_declaration::FunctionDeclaration,
                r#return::Return,
            },
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
            vec![FunctionDeclaration::builder("foo").build().into()],
            "#include <stdint.h>\n\nvoid foo(void) {\n}\n",
        );
    }

    #[test]
    fn compile_function_with_return_void() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .statement(Statement::from(Return { value: None }, Span::default()))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n\nvoid foo(void) {\nreturn;\n}\n",
        );
    }

    #[test]
    fn compile_function_with_return_i32() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .statement(Statement::from(
                        Return { value: Some(Expression::new(ExpressionKind::NumberLiteral(123.0), Span::default())) },
                        Span::default(),
                    ))
                    .return_type(Type::named("i32"))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n\nint32_t foo(void) {\nreturn 123;\n}\n",
        );
    }

    #[test]
    fn compile_function_with_return_identifier_reference() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .parameter("argc", Type::named("i32"), Span::default())
                    .statement(Statement::from(
                        Return {
                            value: Some(Expression::new(
                                ExpressionKind::IdentifierReference("argc".into()),
                                Span::default(),
                            )),
                        },
                        Span::default(),
                    ))
                    .return_type(Type::named("i32"))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n\nint32_t foo(int32_t argc) {\nreturn argc;\n}\n",
        );
    }

    #[test]
    fn compile_empty_function_i32_return_type() {
        assert_compiles(
            vec![FunctionDeclaration::builder("foo").return_type(Type::named("i32")).build().into()],
            "#include <stdint.h>\n\nint32_t foo(void) {\n}\n",
        );
    }

    #[test]
    fn compile_empty_function_with_parameters() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .parameter("a", Type::Named("i32".into()), Span::default())
                    .parameter("b", Type::Named("i32".into()), Span::default())
                    .return_type(Type::named("i32"))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n\nint32_t foo(int32_t a, int32_t b) {\n}\n",
        );
    }
}
