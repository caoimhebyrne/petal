use std::{
    collections::HashMap,
    fmt::Display,
    path::PathBuf,
};

use crate::module::{
    Module,
    ModuleError,
};

/// Each module gets assigned a unique identifier at creation time. This identifier is carried throughout the module's
/// lifecycle, including when it gets promoted to a [`ParsedModule`] and/or a [`CheckedModule`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ModuleId(usize);

impl Display for ModuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Default)]
pub struct ModuleRegistry {
    /// The raw [`Module`]s owned by this registry.
    ///
    /// These modules contain the original file contents, and do not represent the state of the module through
    /// the compilation process.
    modules: HashMap<ModuleId, Module>,
}

impl ModuleRegistry {
    /// Creates a new [`Module`] within this [`ModuleRegistry`], assigning it a unique [`ModuleId`].
    ///
    /// # Errors
    ///
    /// Will return `Err` if `file_path` does not exist, or otherwise could not be read from.
    pub fn create_module(&mut self, file_path: PathBuf) -> Result<InsertModuleResult, ModuleError> {
        self.insert_module(file_path, Module::create)
    }

    /// Creates a new [`Module`] within this [`ModuleRegistry`] with the provided [`String`] being used as its
    /// contents. The [`Module`] will be assigned a unique [`ModuleId`].
    ///
    /// # Errors
    ///
    /// Should not return an error, but the function has a [`Result`] type due to its implementation.
    pub fn create_module_with_contents(
        &mut self,
        file_path: PathBuf,
        contents: String,
    ) -> Result<InsertModuleResult, ModuleError> {
        self.insert_module(file_path, |module_id, file_path| {
            Ok(Module::create_with_contents(module_id, file_path, contents))
        })
    }

    /// Retrieves a [`Module`] from this [`ModuleRegistry`].
    ///
    /// # Panics
    ///
    /// This function will panic if a module does not exist with the provided ID. This is "safe" because the intended
    /// use-case for this structure is for it to only be initialized once. A [`ModuleId`] must not, and cannot, be
    /// created by anything else.
    #[must_use]
    pub fn get_module(&self, id: ModuleId) -> &Module {
        self.modules.get(&id).expect("get_module should never return None")
    }

    /// Inserts a [`Module`] into this [`ModuleRegistry`] by evaluating [`supplier`] with a generated [`ModuleId`].
    fn insert_module<S>(&mut self, file_path: PathBuf, supplier: S) -> Result<InsertModuleResult, ModuleError>
    where
        S: FnOnce(ModuleId, PathBuf) -> Result<Module, ModuleError>,
    {
        // If a module already exists with the provided [`file_path`], then we can return it instead of allocating a
        // new one.
        if let Some(tuple) = self.modules.iter().find(|it| it.1.file_path == file_path) {
            return Ok(InsertModuleResult::Existing(*tuple.0));
        }

        let id = ModuleId(self.modules.len());
        self.modules.insert(id, supplier(id, file_path)?);

        Ok(InsertModuleResult::New(id))
    }
}

/// The result of inserting a module into a [`ModuleRegistry`].
pub enum InsertModuleResult {
    /// The module at the provided path already existed in this [`ModuleRegistry`].
    Existing(ModuleId),

    /// The module at the provided path has not been added to this [`ModuleRegistry`] until now.
    New(ModuleId),
}

/// A fake [`ModuleId`] not registered with any [`ModuleRegistry`].
///
/// This must exclusively be used by tests that require a [`ModuleId`], but do not interact with the
/// [`ModuleRegistry`].
#[cfg(test)]
pub const MOCK_MODULE_ID: ModuleId = ModuleId(0);
