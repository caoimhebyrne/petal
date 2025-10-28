use std::{fs, io, path::PathBuf};

use crate::string_intern::{StringInternPool, StringInternPoolImpl};

/// A module is a single compilation unit, i.e. a single file being processed by the compiler.
pub struct Module {
    /// The path to the file from the current working directory of the process.
    pub input: PathBuf,

    /// The contents of the file to parse.
    pub contents: String,

    /// The string intern pool implementation used by this module.
    pub string_intern_pool: Box<dyn StringInternPool>,
}

impl Module {
    /// Creates a new [Module] instance with the provided file path.
    pub fn new(input: PathBuf) -> Result<Self, io::Error> {
        let contents = fs::read_to_string(&input)?;

        Ok(Module {
            input,
            contents,
            string_intern_pool: Box::new(StringInternPoolImpl::new()),
        })
    }

    /// Returns the name of the module.
    pub fn name(&self) -> String {
        self.input
            .with_extension("")
            .file_name()
            .map(|it| it.to_str())
            .flatten()
            .unwrap_or("unnamed_moule")
            .to_string()
    }
}
