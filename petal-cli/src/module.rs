use std::{env::current_dir, fs, path::PathBuf};

use enum_display::EnumDisplay;
use petal_ast::{
    ASTParser,
    statement::{Statement, StatementKind},
};
use petal_core::{
    error::{Error, ErrorKind, Result},
    source_span::SourceSpan,
    string_intern::StringReference,
};
use petal_lexer::Lexer;
use petal_typechecker::temp_resolved_module::ResolvedModule;

use crate::compiler_state::CompilerState;

trait ResolvedModuleExt {
    /// Creates a new [ResolvedModule] from an existing [Module].
    fn from_module(module: &Module, statements: Vec<Statement>) -> ResolvedModule;
}

/// A module being compiled by the compiler.
pub struct Module {
    /// The path of the source that is being read.
    pub source_path: PathBuf,

    /// The parent directory of the source path.
    parent_directory: PathBuf,

    /// The source code being parsed.
    pub source_contents: String,
}

impl Module {
    /// Creates a new [Module] from a path.
    pub fn new(source_path: PathBuf) -> core::result::Result<Self, std::io::Error> {
        let source_contents = fs::read_to_string(&source_path)?;

        let parent_directory = source_path
            .parent()
            .map(|it| it.to_path_buf())
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "parent directory not found",
            ))?;

        println!(
            "debug: created module from source path '{}' with parent '{}'",
            source_path.display(),
            parent_directory.display()
        );

        Ok(Module {
            source_path,
            parent_directory,
            source_contents,
        })
    }

    /// Resolves this module and any modules referenced by it.
    pub fn resolve(&self, compiler_state: &mut CompilerState) -> Result<Vec<ResolvedModule>> {
        let mut lexer = Lexer::new(compiler_state.string_intern_pool.as_mut(), &self.source_contents);
        let token_stream = lexer.get_stream()?;

        let mut ast_parser = ASTParser::new(token_stream, &mut compiler_state.type_pool);
        let statement_stream = ast_parser.parse()?;

        // There will always be at least 1 module.
        let mut resolved_modules: Vec<ResolvedModule> = Vec::with_capacity(1);

        for statement in &statement_stream.statements {
            match &statement.kind {
                StatementKind::ImportStatement(import) => {
                    let mut referenced_modules =
                        self.resolve_referenced_module(compiler_state, import.module_name, statement.span)?;

                    resolved_modules.append(&mut referenced_modules);
                }

                _ => {}
            }
        }

        // Now that we have resolved all children, we can insert this module.
        resolved_modules.push(ResolvedModule::from_module(self, statement_stream.statements));

        Ok(resolved_modules)
    }

    /// Attempts to resolve a module referenced by this module.
    fn resolve_referenced_module(
        &self,
        compiler_state: &mut CompilerState,
        module_name: StringReference,
        span: SourceSpan,
    ) -> Result<Vec<ResolvedModule>> {
        // A module name must exist in the string intern pool.
        let module_name = compiler_state
            .string_intern_pool
            .resolve_reference_or_err(&module_name, span)?
            .to_owned();

        if module_name == "stdlib" {
            return self.resolve_standard_library_module(compiler_state, span);
        }

        // A corresponding .petal file must exist in the parent directory for the module.
        let module_path = self.parent_directory.join(&module_name).with_added_extension("petal");

        println!(
            "debug: attempting to resolve module at path '{}'",
            module_path.display()
        );

        let module = Module::new(module_path).map_err(|_| ModuleError::module_not_found(&module_name, span))?;
        module.resolve(compiler_state)
    }

    /// Attempts to resolve the standard library module, as referenced by this module.
    fn resolve_standard_library_module(
        &self,
        compiler_state: &mut CompilerState,
        span: SourceSpan,
    ) -> Result<Vec<ResolvedModule>> {
        // FIXME: The standard library path should be somewhere global.
        let standard_library_module_path = current_dir()
            .map_err(|_| ModuleError::module_not_found("stdlib", span))?
            .join("stdlib")
            .join("main")
            .with_added_extension("petal");

        println!(
            "debug: attempting to resolve standard library module at path '{}'",
            standard_library_module_path.display()
        );

        let module =
            Module::new(standard_library_module_path).map_err(|_| ModuleError::module_not_found("stdlib", span))?;
        module.resolve(compiler_state)
    }
}

impl ResolvedModuleExt for ResolvedModule {
    fn from_module(module: &Module, statements: Vec<Statement>) -> Self {
        ResolvedModule::new(module.source_path.clone(), module.source_contents.clone(), statements)
    }
}

#[derive(Debug, Clone, PartialEq, EnumDisplay)]
pub enum ModuleError {
    #[display(
        "A module could not be found with the name '{0}', ensure a {0}.petal file exists as a sibling to the current module"
    )]
    ModuleNotFound(String),
}

impl ModuleError {
    /// Creates a new [Error] with the kind as a [ModuleError::ModuleNotFound] kind.
    pub fn module_not_found(name: &str, span: SourceSpan) -> Error {
        Error::new(ModuleError::ModuleNotFound(name.to_owned()), span)
    }
}

impl ErrorKind for ModuleError {}
