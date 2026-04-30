use std::{
    fmt::{
        self,
    },
    fs,
    io,
};

use crate::{
    ast::{
        ASTParser,
        statement::Statement,
    },
    core::{
        error::Error,
        span::Span,
    },
    lexer::Lexer,
    module_registry::ModuleId,
};

/// A module being compiled by the Petal compiler.
pub struct Module {
    /// The unique identifier for this [`Module`].
    pub id: ModuleId,

    /// The path of the file that this module is being created from.
    pub file_path: String,

    /// The contents of the file that this module is being created from.
    pub file_contents: String,
}

/// A module that has been parsed into an AST.
pub struct ParsedModule {
    /// The unique identifier for this [`ParsedModule`]. This is inherited from its parent [`Module`].
    pub id: ModuleId,

    /// The top-level statements within this module.
    pub ast: Vec<Statement>,
}

impl ParsedModule {
    /// Creates a new [ParsedModule].
    pub fn new(id: ModuleId, ast: Vec<Statement>) -> Self {
        Self { id, ast }
    }
}

/// A module that has been verified by the Typechecker.
#[derive(Debug)]
pub struct CheckedModule {
    /// The unique identifier for this [`CheckedModule`]. This is inherited from its parent [`ParsedModule`].
    pub id: ModuleId,

    /// The top-level statements within this module.
    pub ast: Vec<Statement>,
}

impl CheckedModule {
    /// Creates a new [CheckedModule].
    pub fn new(id: ModuleId, ast: Vec<Statement>) -> Self {
        Self { id, ast }
    }
}

/// An error that occurs while creating a Petal module.
pub enum ModuleError {
    /// The file could not be read.
    IOError { path: String, error: io::Error },
}

impl Module {
    /// Creates a new [`Module`] from a file path.
    pub fn create(id: ModuleId, file_path: String) -> Result<Module, ModuleError> {
        let file_contents =
            fs::read_to_string(&file_path).map_err(|error| ModuleError::IOError { path: file_path.clone(), error })?;

        Ok(Module { id, file_path, file_contents })
    }

    /// Attempts to parse AST nodes from this module.
    pub fn parse(&self) -> Result<ParsedModule, Box<dyn Error>> {
        let mut lexer = Lexer::new(self.id, &self.file_contents);

        let tokens = lexer.parse()?;
        let ast = ASTParser::new_and_parse(self.id, tokens)?;

        Ok(ParsedModule::new(self.id, ast))
    }
}

impl Error for ModuleError {
    fn span(&self) -> Option<Span> {
        None
    }
}

impl fmt::Display for ModuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModuleError::IOError { path, error } => write!(f, "Could not read from '{path}': {error}"),
        }
    }
}
