use std::{
    collections::HashMap,
    io::Write,
    path::PathBuf,
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
    typechecker::{
        context::{
            CheckedFunction,
            DeclaredStructure,
            FunctionId,
            StructureId,
        },
        r#type::Type,
    },
};

pub mod error;
pub mod expression;
pub mod statement;

/// The C codegen backend.
pub struct CBackend {
    /// The structure types that were resolved by the type checker for these modules.
    structures: HashMap<StructureId, DeclaredStructure>,

    /// The functions that were resolved by the type checker for these modules.
    functions: HashMap<FunctionId, CheckedFunction>,
}

impl CBackend {
    /// Creates a new [`CBackend`].
    pub fn new(
        structures: HashMap<StructureId, DeclaredStructure>,
        functions: HashMap<FunctionId, CheckedFunction>,
    ) -> Self {
        Self { structures, functions }
    }

    /// Compiles a [`CheckedModule`] to C code.
    pub fn emit_code(&self, modules: &Vec<CheckedModule>) -> Result<String, CBackendError> {
        let mut code = String::new();

        code.push_str("#include <stdint.h>\n#include <stdbool.h>\n\n");

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

        // FIXME: It would be nice to introduce passes like the typechecker, but that's not so easy here.
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
                code.push_str(&self.compile_statement(statement)?);
            }
        }

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
    fn compile_type(&self, r#type: &Type, span: Span) -> Result<String, CBackendError> {
        let value = match r#type {
            Type::SignedInteger(size) => format!("int{}_t", size),
            Type::UnsignedInteger(size) => format!("uint{}_t", size),
            Type::Boolean => "bool".into(),
            Type::Void => "void".into(),
            Type::Reference(referenced) => format!("{}*", self.compile_type(referenced, span)?),
            Type::Structure(structure_id) => {
                let structure = self.structures.get(structure_id).unwrap();
                structure.name.clone()
            }
            Type::Unknown => return Err(CBackendErrorKind::UnknownType.at(span)),
        };

        Ok(value)
    }
}
