use std::{
    fmt::{self},
    fs, io,
};

use crate::{
    ast::ASTParser,
    core::{error::Error, span::Span},
    lexer::Lexer,
};

/// A module being compiled by the Petal compiler.
pub struct Module {
    /// The path of the file that this module is being created from.
    pub file_path: String,

    /// The contents of the file that this module is being created from.
    pub file_contents: String,
}

/// An error that occurs while creating a Petal module.
pub enum ModuleError {
    /// The file could not be read.
    IOError(io::Error),
}

impl Module {
    /// Creates a new [`Module`] from a file path.
    pub fn create(file_path: String) -> Result<Module, ModuleError> {
        let file_contents = fs::read_to_string(&file_path).map_err(ModuleError::IOError)?;
        Ok(Module { file_path, file_contents })
    }

    /// Attempts to parse AST nodes from this module.
    pub fn parse(&self) -> Result<(), Box<dyn Error>> {
        let mut lexer = Lexer::new(&self.file_contents);

        let tokens = lexer.parse()?;
        let _ = ASTParser::new_and_parse(tokens)?;

        Ok(())
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
            ModuleError::IOError(error) => write!(f, "failed to read module file contents: {}", error),
        }
    }
}
