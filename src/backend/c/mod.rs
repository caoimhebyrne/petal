use std::{
    io::Write,
    path::Path,
    process::{
        Command,
        Stdio,
    },
};

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
    pub fn emit_code(module: &ParsedModule) -> Result<String, CBackendError> {
        let mut code = String::new();

        code.push_str("#include <stdint.h>\n\n");

        for statement in &module.ast {
            code.push_str(&CBackend::compile_statement(statement)?);
        }

        Ok(code)
    }

    /// Compiles C code into a binary.
    pub fn emit_binary(code: &str, output_binary_path: impl AsRef<Path>) -> Result<(), CBackendError> {
        let mut child = Command::new("cc")
            // Tell the compiler that the stdin contains C code.
            .args(["-x", "c"])
            .arg("-o")
            .arg(output_binary_path.as_ref())
            // Tell the compiler to read from stdin.
            .arg("-")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| CBackendErrorKind::CompilerInvocationFailed(e.to_string()).without_span())?;

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(code.as_bytes())
            .map_err(|e| CBackendErrorKind::CompilerInvocationFailed(e.to_string()).without_span())?;

        let status =
            child.wait().map_err(|e| CBackendErrorKind::CompilerInvocationFailed(e.to_string()).without_span())?;

        if !status.success() {
            return Err(CBackendErrorKind::CompilerInvocationFailed(format!(
                "Exited with a non-zero status code: {:?}",
                status.code(),
            ))
            .without_span());
        }

        Ok(())
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
                function_call::FunctionCall,
            },
            statement::{
                Statement,
                StatementKind,
                function_declaration::FunctionDeclaration,
                r#return::Return,
                variable_declaration::VariableDeclaration,
            },
        },
        core::span::Span,
    };

    fn assert_compiles(kinds: Vec<StatementKind>, compiled: &str) {
        let statements = kinds.into_iter().map(|kind| Statement::from(kind, Span::default())).collect();
        assert_eq!(CBackend::emit_code(&ParsedModule::new(statements)), Ok(compiled.into()))
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
    fn compile_function_with_variable_declaration_i32() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .statement(Statement::from(
                        VariableDeclaration::new(
                            "variable",
                            Type::named("i32"),
                            Expression::new(ExpressionKind::NumberLiteral(999.0), Span::default()),
                        ),
                        Span::default(),
                    ))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n\nvoid foo(void) {\nint32_t variable = 999;\n}\n",
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
    fn compile_function_with_variable_declaration_i32_and_return_identifier_reference() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .statement(Statement::from(
                        VariableDeclaration::new(
                            "variable",
                            Type::named("i32"),
                            Expression::new(ExpressionKind::NumberLiteral(999.0), Span::default()),
                        ),
                        Span::default(),
                    ))
                    .statement(Statement::from(
                        Return {
                            value: Some(Expression::new(
                                ExpressionKind::IdentifierReference("variable".into()),
                                Span::default(),
                            )),
                        },
                        Span::default(),
                    ))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n\nvoid foo(void) {\nint32_t variable = 999;\nreturn variable;\n}\n",
        );
    }

    #[test]
    fn compile_function_with_variable_declaration_func_call_no_args() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .parameter("argc", Type::named("i32"), Span::default())
                    .statement(Statement::from(
                        VariableDeclaration::new(
                            "variable",
                            Type::named("i32"),
                            Expression::new(FunctionCall::builder("my_func").build().into(), Span::default()),
                        ),
                        Span::default(),
                    ))
                    .return_type(Type::named("i32"))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n\nint32_t foo(int32_t argc) {\nint32_t variable = my_func();\n}\n",
        );
    }

    #[test]
    fn compile_function_with_variable_declaration_func_call_with_arg() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .parameter("argc", Type::named("i32"), Span::default())
                    .statement(Statement::from(
                        VariableDeclaration::new(
                            "variable",
                            Type::named("i32"),
                            Expression::new(
                                FunctionCall::builder("my_func")
                                    .argument(Expression::new(ExpressionKind::NumberLiteral(1.0), Span::default()))
                                    .build()
                                    .into(),
                                Span::default(),
                            ),
                        ),
                        Span::default(),
                    ))
                    .return_type(Type::named("i32"))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n\nint32_t foo(int32_t argc) {\nint32_t variable = my_func(1);\n}\n",
        );
    }

    #[test]
    fn compile_function_with_variable_declaration_func_call_with_args() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .parameter("argc", Type::named("i32"), Span::default())
                    .statement(Statement::from(
                        VariableDeclaration::new(
                            "variable",
                            Type::named("i32"),
                            Expression::new(
                                FunctionCall::builder("my_func")
                                    .argument(Expression::new(ExpressionKind::NumberLiteral(1.0), Span::default()))
                                    .argument(Expression::new(ExpressionKind::NumberLiteral(2.0), Span::default()))
                                    .argument(Expression::new(ExpressionKind::NumberLiteral(3.0), Span::default()))
                                    .build()
                                    .into(),
                                Span::default(),
                            ),
                        ),
                        Span::default(),
                    ))
                    .return_type(Type::named("i32"))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n\nint32_t foo(int32_t argc) {\nint32_t variable = my_func(1, 2, 3);\n}\n",
        );
    }

    #[test]
    fn compile_function_with_variable_declaration_nested_func_call() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .statement(Statement::from(
                        VariableDeclaration::new(
                            "variable",
                            Type::named("i32"),
                            Expression::new(
                                FunctionCall::builder("foo")
                                    .argument(Expression::new(
                                        FunctionCall::builder("bar").build().into(),
                                        Span::default(),
                                    ))
                                    .build()
                                    .into(),
                                Span::default(),
                            ),
                        ),
                        Span::default(),
                    ))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n\nvoid foo(void) {\nint32_t variable = foo(bar());\n}\n",
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
