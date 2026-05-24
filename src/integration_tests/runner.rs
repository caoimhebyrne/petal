use std::{
    env::current_dir,
    fs,
    path::PathBuf,
};

use crate::{
    integration_tests::TestResult,
    module::ParsedModule,
    module_registry::{
        InsertModuleResult,
        ModuleId,
        ModuleRegistry,
    },
    typed_ast::resolver::TypeResolver,
};

/// Responsible for orchestrating the execution of a single test case.
pub struct TestCaseRunner {
    /// The [`ModuleRegistry`] used by this [`TestCaseRunner`].
    module_registry: ModuleRegistry,

    /// The [`ParsedModule`]s within this [`TestCaseRunner`].
    parsed_modules: Vec<ParsedModule>,
}

impl TestCaseRunner {
    /// Creates a new [`TestCaseRunner`], adding the prelude module on creation (assuming that it is located in the
    /// current working directory).
    pub fn with_prelude() -> TestResult<Self> {
        let mut this = Self { module_registry: ModuleRegistry::default(), parsed_modules: Vec::new() };

        let prelude_module_path = current_dir().expect("current_dir").join("prelude");
        debug!("Using prelude module path: '{}'", prelude_module_path.display());

        // todo: `ModuleResolver` should be able to import directories.
        for entry in fs::read_dir(prelude_module_path).expect("read_dir") {
            let entry = entry.expect("entry");
            this.add_module_from_path(entry.path())?;
        }

        Ok(this)
    }

    /// Adds a new module to this [`TestCaseRunner`], reading its contents from the file pointed to by the provided
    /// [`PathBuf`].
    pub fn add_module_from_path(&mut self, file_path: PathBuf) -> TestResult<()> {
        match self.module_registry.create_module(file_path)? {
            InsertModuleResult::Existing(_) => Ok(()),
            InsertModuleResult::New(module_id) => self.parse_module(module_id),
        }
    }

    /// Adds a module to this [`TestCaseRunner`], using the provided [`value`] as its file contents. The path
    /// associated with the module will just be the current working directory.
    ///
    /// # Panics
    ///
    /// This function will panic if another module already exists with its path being the current working directory,
    /// meaning that you cannot call [`add_module_from_str`] more than once on a given [`TestCaseRunner`].
    pub fn add_module_from_str(&mut self, value: &str) -> TestResult<()> {
        // The path that we assign to the module is _technically_ incorrect, but since we're providing the contents, it
        // doesn't actually need to exist.
        let file_path = current_dir().expect("current_dir");

        let module_id = match self.module_registry.create_module_with_contents(file_path.clone(), value.to_string())? {
            InsertModuleResult::Existing(value) => panic!(
                "`create_module_with_contents` returned `InsertModuleResult::Existing({value})` for file path '{}'",
                file_path.display()
            ),

            InsertModuleResult::New(value) => value,
        };

        debug!("Module ID {} assigned to path '{}'", module_id, file_path.display());

        self.parse_module(module_id)
    }

    /// Attempts to compile the [`ParsedModule`]s within this [`TestCaseRunner`].
    pub fn compile(self) -> TestResult<()> {
        TypeResolver::default().resolve(self.parsed_modules)?;
        Ok(())
    }

    /// Parses the [`Module`] referenced by the provided [`ModuleId`].
    fn parse_module(&mut self, module_id: ModuleId) -> TestResult<()> {
        let parsed_module = self.module_registry.get_module(module_id).parse()?;
        self.parsed_modules.push(parsed_module);

        Ok(())
    }
}
