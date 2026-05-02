use std::{
    io::Write,
    path::Path,
    process::{
        Command,
        Stdio,
    },
};

use crate::{
    ast::statement::StatementKind,
    backend::c::error::{
        CBackendError,
        CBackendErrorKind,
    },
    core::span::Span,
    module::CheckedModule,
    typechecker::r#type::Type,
};

pub mod error;
pub mod expression;
pub mod statement;

/// The C codegen backend.
pub struct CBackend;

impl CBackend {
    /// Compiles a [`CheckedModule`] to C code.
    pub fn emit_code(modules: &Vec<CheckedModule>) -> Result<String, CBackendError> {
        let mut code = String::new();

        code.push_str("#include <stdint.h>\n#include <stdbool.h>\n\n");

        // FIXME: It would be nice to introduce passes like the typechecker, but that's not so easy here.
        for module in modules {
            for statement in &module.ast {
                if let StatementKind::FunctionDeclaration(function_declaration) = &statement.kind {
                    let name = function_declaration.name.clone();
                    let return_type = CBackend::compile_type(&function_declaration.return_type, statement.span)?;

                    let parameters: String = if function_declaration.parameters.is_empty() {
                        "void".into()
                    } else {
                        function_declaration
                            .parameters
                            .iter()
                            .map(CBackend::compile_function_parameter)
                            .collect::<Result<Vec<String>, CBackendError>>()?
                            .join(", ")
                    };

                    code.push_str(&format!("{return_type} {name}({parameters});\n"));
                }
            }
        }

        code.push('\n');

        for module in modules {
            for statement in &module.ast {
                code.push_str(&CBackend::compile_statement(statement)?);
            }
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
        let value = match r#type {
            Type::SignedInteger(size) => format!("int{}_t", size),
            Type::UnsignedInteger(size) => format!("uint{}_t", size),
            Type::Boolean => "bool".into(),
            Type::Void => "void".into(),
            Type::Reference(referenced) => format!("{}*", CBackend::compile_type(referenced, span)?),
            Type::Unknown => return Err(CBackendErrorKind::UnknownType.at(span)),
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
                variable_assignment::VariableAssignment,
                variable_declaration::VariableDeclaration,
            },
            type_expr::TypeExpr,
        },
        core::span::Span,
        module_registry::MOCK_MODULE_ID,
    };

    fn assert_compiles(kinds: Vec<StatementKind>, compiled: &str) {
        let statements = kinds.into_iter().map(|kind| Statement::from(kind, Span::new(MOCK_MODULE_ID, 0, 0))).collect();
        assert_eq!(CBackend::emit_code(&vec![CheckedModule::new(MOCK_MODULE_ID, statements)]), Ok(compiled.into()))
    }

    #[test]
    fn compile_empty_function() {
        assert_compiles(
            vec![FunctionDeclaration::builder("foo").return_type(TypeExpr::named("void"), Type::Void).build().into()],
            "#include <stdint.h>\n#include <stdbool.h>\n\nvoid foo(void);\n\nvoid foo(void) {\n}\n\n",
        );
    }

    #[test]
    fn compile_function_with_return_void() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .statement(Statement::from(Return { value: None }, Span::new(MOCK_MODULE_ID, 0, 0)))
                    .return_type(TypeExpr::named("void"), Type::Void)
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n#include <stdbool.h>\n\nvoid foo(void);\n\nvoid foo(void) {\nreturn;\n}\n\n",
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
                            TypeExpr::named("i32"),
                            Type::SignedInteger(32),
                            Expression::new(ExpressionKind::NumberLiteral(999.0), Span::new(MOCK_MODULE_ID, 0, 0)),
                        ),
                        Span::new(MOCK_MODULE_ID, 0, 0),
                    ))
                    .return_type(TypeExpr::named("void"), Type::Void)
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n#include <stdbool.h>\n\nvoid foo(void);\n\nvoid foo(void) {\nint32_t variable = 999;\n}\n\n",
        );
    }

    #[test]
    fn compile_function_with_return_i32() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .statement(Statement::from(
                        Return {
                            value: Some(Expression::new(
                                ExpressionKind::NumberLiteral(123.0),
                                Span::new(MOCK_MODULE_ID, 0, 0),
                            )),
                        },
                        Span::new(MOCK_MODULE_ID, 0, 0),
                    ))
                    .return_type(TypeExpr::named("i32"), Type::SignedInteger(32))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n#include <stdbool.h>\n\nint32_t foo(void);\n\nint32_t foo(void) {\nreturn 123;\n}\n\n",
        );
    }

    #[test]
    fn compile_function_with_return_identifier_reference() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .parameter(
                        "argc",
                        TypeExpr::named("i32"),
                        Type::SignedInteger(32),
                        false,
                        Span::new(MOCK_MODULE_ID, 0, 0),
                    )
                    .statement(Statement::from(
                        Return {
                            value: Some(Expression::new(
                                ExpressionKind::IdentifierReference("argc".into()),
                                Span::new(MOCK_MODULE_ID, 0, 0),
                            )),
                        },
                        Span::new(MOCK_MODULE_ID, 0, 0),
                    ))
                    .return_type(TypeExpr::named("i32"), Type::SignedInteger(32))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n#include <stdbool.h>\n\nint32_t foo(int32_t argc);\n\nint32_t foo(int32_t argc) {\nreturn argc;\n}\n\n",
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
                            TypeExpr::named("i32"),
                            Type::SignedInteger(32),
                            Expression::new(ExpressionKind::NumberLiteral(999.0), Span::new(MOCK_MODULE_ID, 0, 0)),
                        ),
                        Span::new(MOCK_MODULE_ID, 0, 0),
                    ))
                    .statement(Statement::from(
                        Return {
                            value: Some(Expression::new(
                                ExpressionKind::IdentifierReference("variable".into()),
                                Span::new(MOCK_MODULE_ID, 0, 0),
                            )),
                        },
                        Span::new(MOCK_MODULE_ID, 0, 0),
                    ))
                    .return_type(TypeExpr::named("void"), Type::Void)
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n#include <stdbool.h>\n\nvoid foo(void);\n\nvoid foo(void) {\nint32_t variable = 999;\nreturn variable;\n}\n\n",
        );
    }

    #[test]
    fn compile_function_with_variable_assignment() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .statement(Statement::from(
                        VariableAssignment::new(
                            Expression::new(
                                ExpressionKind::IdentifierReference("variable".into()),
                                Span::new(MOCK_MODULE_ID, 0, 0),
                            ),
                            Expression::new(ExpressionKind::NumberLiteral(456.0), Span::new(MOCK_MODULE_ID, 0, 0)),
                        ),
                        Span::new(MOCK_MODULE_ID, 0, 0),
                    ))
                    .return_type(TypeExpr::named("i32"), Type::SignedInteger(32))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n#include <stdbool.h>\n\nint32_t foo(void);\n\nint32_t foo(void) {\nvariable = 456;\n}\n\n",
        );
    }

    #[test]
    fn compile_function_with_variable_declaration_func_call_no_args() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .parameter(
                        "argc",
                        TypeExpr::named("i32"),
                        Type::SignedInteger(32),
                        false,
                        Span::new(MOCK_MODULE_ID, 0, 0),
                    )
                    .statement(Statement::from(
                        VariableDeclaration::new(
                            "variable",
                            TypeExpr::named("i32"),
                            Type::SignedInteger(32),
                            Expression::new(
                                FunctionCall::builder("my_func").build().into(),
                                Span::new(MOCK_MODULE_ID, 0, 0),
                            ),
                        ),
                        Span::new(MOCK_MODULE_ID, 0, 0),
                    ))
                    .return_type(TypeExpr::named("i32"), Type::SignedInteger(32))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n#include <stdbool.h>\n\nint32_t foo(int32_t argc);\n\nint32_t foo(int32_t argc) {\nint32_t variable = my_func();\n}\n\n",
        );
    }

    #[test]
    fn compile_function_with_variable_declaration_func_call_with_arg() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .parameter(
                        "argc",
                        TypeExpr::named("i32"),
                        Type::SignedInteger(32),
                        false,
                        Span::new(MOCK_MODULE_ID, 0, 0),
                    )
                    .statement(Statement::from(
                        VariableDeclaration::new(
                            "variable",
                            TypeExpr::named("i32"),
                            Type::SignedInteger(32),
                            Expression::new(
                                FunctionCall::builder("my_func")
                                    .argument(
                                        None,
                                        Expression::new(
                                            ExpressionKind::NumberLiteral(1.0),
                                            Span::new(MOCK_MODULE_ID, 0, 0),
                                        ),
                                        Span::new(MOCK_MODULE_ID, 0, 0),
                                    )
                                    .build()
                                    .into(),
                                Span::new(MOCK_MODULE_ID, 0, 0),
                            ),
                        ),
                        Span::new(MOCK_MODULE_ID, 0, 0),
                    ))
                    .return_type(TypeExpr::named("i32"), Type::SignedInteger(32))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n#include <stdbool.h>\n\nint32_t foo(int32_t argc);\n\nint32_t foo(int32_t argc) {\nint32_t variable = my_func(1);\n}\n\n",
        );
    }

    #[test]
    fn compile_function_with_variable_declaration_func_call_with_args() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .parameter(
                        "argc",
                        TypeExpr::named("i32"),
                        Type::SignedInteger(32),
                        false,
                        Span::new(MOCK_MODULE_ID, 0, 0),
                    )
                    .statement(Statement::from(
                        VariableDeclaration::new(
                            "variable",
                            TypeExpr::named("i32"),
                            Type::SignedInteger(32),
                            Expression::new(
                                FunctionCall::builder("my_func")
                                    .argument(
                                        None,
                                        Expression::new(
                                            ExpressionKind::NumberLiteral(1.0),
                                            Span::new(MOCK_MODULE_ID, 0, 0),
                                        ),
                                        Span::new(MOCK_MODULE_ID, 0, 0),
                                    )
                                    .argument(
                                        None,
                                        Expression::new(
                                            ExpressionKind::NumberLiteral(2.0),
                                            Span::new(MOCK_MODULE_ID, 0, 0),
                                        ),
                                        Span::new(MOCK_MODULE_ID, 0, 0),
                                    )
                                    .argument(
                                        None,
                                        Expression::new(
                                            ExpressionKind::NumberLiteral(3.0),
                                            Span::new(MOCK_MODULE_ID, 0, 0),
                                        ),
                                        Span::new(MOCK_MODULE_ID, 0, 0),
                                    )
                                    .build()
                                    .into(),
                                Span::new(MOCK_MODULE_ID, 0, 0),
                            ),
                        ),
                        Span::new(MOCK_MODULE_ID, 0, 0),
                    ))
                    .return_type(TypeExpr::named("i32"), Type::SignedInteger(32))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n#include <stdbool.h>\n\nint32_t foo(int32_t argc);\n\nint32_t foo(int32_t argc) {\nint32_t variable = my_func(1, 2, 3);\n}\n\n",
        );
    }

    #[test]
    fn compile_function_with_variable_declaration_nested_func_call() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .return_type(TypeExpr::named("void"), Type::Void)
                    .statement(Statement::from(
                        VariableDeclaration::new(
                            "variable",
                            TypeExpr::named("i32"),
                            Type::SignedInteger(32),
                            Expression::new(
                                FunctionCall::builder("foo")
                                    .argument(
                                        None,
                                        Expression::new(
                                            FunctionCall::builder("baz").build().into(),
                                            Span::new(MOCK_MODULE_ID, 0, 0),
                                        ),
                                        Span::new(MOCK_MODULE_ID, 0, 0),
                                    )
                                    .build()
                                    .into(),
                                Span::new(MOCK_MODULE_ID, 0, 0),
                            ),
                        ),
                        Span::new(MOCK_MODULE_ID, 0, 0),
                    ))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n#include <stdbool.h>\n\nvoid foo(void);\n\nvoid foo(void) {\nint32_t variable = foo(baz());\n}\n\n",
        );
    }

    #[test]
    fn compile_empty_function_i32_return_type() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .return_type(TypeExpr::named("i32"), Type::SignedInteger(32))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n#include <stdbool.h>\n\nint32_t foo(void);\n\nint32_t foo(void) {\n}\n\n",
        );
    }

    #[test]
    fn compile_empty_function_with_parameters() {
        assert_compiles(
            vec![
                FunctionDeclaration::builder("foo")
                    .parameter(
                        "a",
                        TypeExpr::Named("i32".into()),
                        Type::SignedInteger(32),
                        false,
                        Span::new(MOCK_MODULE_ID, 0, 0),
                    )
                    .parameter(
                        "b",
                        TypeExpr::Named("i32".into()),
                        Type::SignedInteger(32),
                        false,
                        Span::new(MOCK_MODULE_ID, 0, 0),
                    )
                    .return_type(TypeExpr::named("i32"), Type::SignedInteger(32))
                    .build()
                    .into(),
            ],
            "#include <stdint.h>\n#include <stdbool.h>\n\nint32_t foo(int32_t a, int32_t b);\n\nint32_t foo(int32_t a, int32_t b) {\n}\n\n",
        );
    }
}
