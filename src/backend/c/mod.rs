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
    ast::{
        statement::{
            StatementKind,
            function_declaration::DeclarationModifier,
        },
        type_expr::GenericTypeArgument,
    },
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
            DeclaredType,
            DeclaredTypeId,
            FunctionId,
            SpecializedStructure,
            SpecializedStructureId,
            StructureId,
            SyntheticType,
        },
        r#type::{
            StructureReference,
            Type,
        },
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

    /// The types declared by the user during compilation.
    declared_types: HashMap<DeclaredTypeId, DeclaredType>,

    /// The functions defined in the source code during compilation.
    functions: HashMap<FunctionId, CheckedFunction>,

    /// The structures defined in the source code during compilation.
    structures: HashMap<StructureId, DeclaredStructure>,

    /// The specialized structures defined in the source code during compilation.
    specialized_structures: HashMap<SpecializedStructureId, SpecializedStructure>,

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
        declared_types: HashMap<DeclaredTypeId, DeclaredType>,
        functions: HashMap<FunctionId, CheckedFunction>,
        structures: HashMap<StructureId, DeclaredStructure>,
        specialized_structures: HashMap<SpecializedStructureId, SpecializedStructure>,
        synthetic_types: HashSet<SyntheticType>,
    ) -> Self {
        Self {
            builtin_types,
            declared_types,
            functions,
            structures,
            specialized_structures,
            synthetic_types,
            writer: Writer::default(),
        }
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
            let declared_type = &self.declared_types[&structure.declared_type_id];
            if !declared_type.generic_type_parameters.is_empty() {
                debug!(
                    "Not generating definition for type '{}' as it is generic, a specialization should cover it",
                    declared_type.name
                );
                continue;
            }

            code.push_str("typedef struct {\n");

            for field in &structure.fields {
                code.push_str(&format!("    {} {};\n", self.compile_type(&field.r#type, field.span)?, field.name));
            }

            code.push_str(&format!("}} {};\n\n", self.declared_type_name(declared_type, &vec![])?));
        }

        for specialized_structure in self.specialized_structures.values() {
            let declared_type = &self.declared_types[&specialized_structure.generic_type_id];
            code.push_str("typedef struct {\n");

            for field in &specialized_structure.fields {
                code.push_str(&format!("    {} {};\n", self.compile_type(&field.r#type, field.span)?, field.name));
            }

            code.push_str(&format!(
                "}} {};\n\n",
                self.declared_type_name(declared_type, &specialized_structure.generic_type_arguments)?
            ));
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
                    let function_id = function_declaration
                        .function_id
                        .ok_or(CBackendErrorKind::MissingFunctionId.at(statement.span))?;

                    let name = self.function_name(&function_id)?;
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
            Type::Optional(inner) => format!("Optional_{}", self.identifier_friendly_name(inner, span)?),
            Type::Structure(_) => self.identifier_friendly_name(r#type, span)?,
            Type::GenericType(_) | Type::Unknown => return Err(CBackendErrorKind::UnknownType.at(span)),
        };

        Ok(value)
    }

    /// Generates a name for the provided [`DeclaredType`].
    fn declared_type_name(
        &self,
        declared_type: &DeclaredType,
        generic_type_arguments: &Vec<GenericTypeArgument>,
    ) -> Result<String, CBackendError> {
        let mut name = format!(
            "ptl_mod_{}_{}_type_{}",
            declared_type.module_id,
            declared_type.namespace.clone().unwrap_or_else(|| "root".to_string()),
            declared_type.name
        );

        for argument in generic_type_arguments {
            name.push('_');
            name.push_str(&self.identifier_friendly_name(&argument.r#type, argument.span)?);
        }

        Ok(name)
    }

    /// Generates a name for the provided [`FunctionId`].
    fn function_name(&self, function_id: &FunctionId) -> Result<String, CBackendError> {
        let checked_function = &self.functions[function_id];

        // If the function is external, then we must not mangle its name.
        if checked_function.modifiers.contains(&DeclarationModifier::Extern)
            || (checked_function.namespace.is_none() && checked_function.name == "main")
        {
            return Ok(checked_function.name.clone());
        }

        let mut name = format!(
            "ptl_mod_{}_{}_fn_{}",
            checked_function.module_id,
            checked_function.namespace.clone().unwrap_or_else(|| "root".to_string()),
            checked_function.name,
        );

        for parameter in &checked_function.parameters {
            name.push('_');
            name.push_str(&self.identifier_friendly_name(&parameter.r#type, parameter.span)?);
        }

        Ok(name)
    }

    /// Returns a name for the provided structure reference type.
    fn get_name_for_structure_reference(
        &self,
        structure_reference: &StructureReference,
    ) -> Result<String, CBackendError> {
        match structure_reference {
            StructureReference::Plain(plain_id) => {
                let structure = &self.structures[plain_id];
                let declared_type = &self.declared_types[&structure.declared_type_id];
                self.declared_type_name(declared_type, &vec![])
            }

            StructureReference::Specialized(specialized_id) => {
                let structure = &self.specialized_structures[specialized_id];
                let declared_type = &self.declared_types[&structure.generic_type_id];
                self.declared_type_name(declared_type, &structure.generic_type_arguments)
            }
        }
    }

    /// Returns a name for the provided type that is able to be used within an identifier.
    fn identifier_friendly_name(&self, r#type: &Type, span: Span) -> Result<String, CBackendError> {
        let name = match r#type {
            Type::Boolean => "bool".to_string(),
            Type::Optional(inner) => format!("{}opt", self.identifier_friendly_name(inner, span)?),
            Type::Reference(inner) => format!("{}ref", self.identifier_friendly_name(inner, span)?),
            Type::SignedInteger(size) => format!("i{size}"),
            Type::Structure(structure_reference) => self.get_name_for_structure_reference(structure_reference)?,
            Type::UnsignedInteger(size) => format!("u{size}"),
            Type::Void => format!("void"),
            Type::GenericType(_) | Type::Unknown => return Err(CBackendErrorKind::UnknownType.at(span)),
        };

        Ok(name)
    }
}
