use std::{
    self,
    collections::{
        HashMap,
        HashSet,
    },
    io::Write,
    path::PathBuf,
    process::{
        Command,
        Stdio,
    },
};

use crate::{
    ast::statement::StatementKind,
    backend::c::{
        error::{
            CBackendError,
            CBackendErrorKind,
        },
        writer::Writer,
    },
    core::span::Span,
    module::CheckedModule,
    typechecker::{
        BuiltinTypes,
        context::{
            CheckedFunction,
            DeclaredStructure,
            FunctionId,
            StructureId,
            SyntheticType,
        },
        r#type::Type,
    },
};

pub mod error;
pub mod expression;
pub mod statement;
mod writer;

/// The C codegen backend.
pub struct CBackend {
    /// The built-in types that have been recognized during compilation.
    builtin_types: BuiltinTypes,

    /// The functions defined in the source code during compilation.
    functions: HashMap<FunctionId, CheckedFunction>,

    /// The structures defined in the source code during compilation.
    structures: HashMap<StructureId, DeclaredStructure>,

    /// The types that have been synthesised during compilation.
    ///
    /// This could include: optional type implementations and generic type implementations.
    synthetic_types: HashSet<SyntheticType>,

    /// The writer to use.
    writer: Writer,
}

impl CBackend {
    /// Creates a new [`CBackend`].
    pub fn new(
        builtin_types: BuiltinTypes,
        functions: HashMap<FunctionId, CheckedFunction>,
        structures: HashMap<StructureId, DeclaredStructure>,
        synthetic_types: HashSet<SyntheticType>,
    ) -> Self {
        Self { builtin_types, functions, structures, synthetic_types, writer: Writer::default() }
    }

    /// Compiles a [`CheckedModule`] to C code.
    pub fn emit_code(mut self, modules: &Vec<CheckedModule>) -> Result<String, CBackendError> {
        let mut code = String::new();

        code.push_str("#include <stdint.h>\n#include <stdbool.h>\n#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n#include <unistd.h>\n\n");

        code.push_str(
            r#"_Noreturn static void __ptl_internal_fn_panic(const char* msg) {
    fprintf(stderr, "PANIC: %s\n", msg);
    exit(255);
}

"#,
        );

        debug!("Attempting to generate C code with {} structure(s)", self.structures.len());

        for structure in self.structures.values() {
            code.push_str("typedef struct {\n");

            for field in &structure.fields {
                code.push_str(&format!("    {} {};\n", self.compile_type(&field.r#type, field.span)?, field.name));
            }

            code.push_str(&format!("}} {};\n", structure.name));
        }

        if !self.structures.is_empty() {
            code.push('\n');
        }

        for synthetic_type in &self.synthetic_types {
            match synthetic_type {
                SyntheticType::Optional { inner_type } => {
                    let inner_type_str = self.compile_type(&inner_type, Span::new(modules[0].id, 0, 0))?;

                    code.push_str(&format!(
                        "typedef struct {{ bool has_value; {} value; }} Optional_{};\n\n",
                        inner_type_str, inner_type
                    ));
                }
            }
        }

        for module in modules {
            for statement in &module.ast {
                if let StatementKind::FunctionDeclaration(function_declaration) = &statement.kind {
                    let name = function_declaration.name.clone();
                    let return_type = self.compile_type(&function_declaration.return_type, statement.span)?;

                    let parameters: String = if function_declaration.parameters.is_empty() {
                        "void".into()
                    } else {
                        function_declaration
                            .parameters
                            .iter()
                            .map(|it| self.compile_function_parameter(it))
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
                self.compile_statement(statement)?;
            }
        }

        code.push_str(&self.writer.code);
        Ok(code)
    }

    /// Compiles C code into a binary.
    pub fn emit_binary(code: &str, executable_file_path: &PathBuf) -> Result<(), CBackendError> {
        let mut child = Command::new("cc")
            // Tell the compiler that the stdin contains C code.
            .args(["-x", "c"])
            .arg("-o")
            .arg(executable_file_path)
            // Tell the compiler to read from stdin.
            .arg("-")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| CBackendErrorKind::CompilerInvocationFailed(e.to_string()).without_span())?;

        child
            .stdin
            .as_mut()
            .ok_or(
                CBackendErrorKind::CompilerInvocationFailed("Failed to open stdin to compiler process".into())
                    .without_span(),
            )?
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
    fn compile_type(&self, r#type: &Type, span: Span) -> Result<String, CBackendError> {
        let value = match r#type {
            Type::SignedInteger(size) => format!("int{}_t", size),
            Type::UnsignedInteger(size) => format!("uint{}_t", size),
            Type::Boolean => "bool".into(),
            Type::Void => "void".into(),
            Type::Reference(referenced) => format!("{}*", self.compile_type(referenced, span)?),
            Type::Structure(structure_id) => {
                let structure = self
                    .structures
                    .get(structure_id)
                    .ok_or(CBackendErrorKind::MissingStructure(*structure_id).at(span))?;

                structure.name.clone()
            }
            Type::Optional(inner) => format!("Optional_{}", inner),
            Type::Unknown => return Err(CBackendErrorKind::UnknownType.at(span)),
        };

        Ok(value)
    }
}
