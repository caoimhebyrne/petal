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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
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
    pub fn create_module(&mut self, file_path: PathBuf) -> Result<(ModuleId, bool), ModuleError> {
        // If a module already exists with the provided file path, we must return the existing one.
        if let Some((module_id, _)) = self.modules.iter().find(|it| it.1.file_path == file_path) {
            return Ok((*module_id, true));
        }

        let id = ModuleId(self.modules.len());

        let module = Module::create(id, file_path)?;
        self.modules.insert(id, module);

        Ok((id, false))
    }

    /// Retrieves a [`Module`] from this [`ModuleRegistry`].
    ///
    /// This function will panic if a module does not exist with the provided ID. This is "safe" because the intended
    /// use-case for this structure is for it to only be initialized once. A [`ModuleId`] must not, and cannot, be
    /// created by anything else.
    pub fn get_module(&self, id: ModuleId) -> &Module {
        self.modules.get(&id).expect("get_module should never return None")
    }
}

/// A fake [`ModuleId`] not registered with any [`ModuleRegistry`].
///
/// This must exclusively be used by tests that require a [`ModuleId`], but do not interact with the
/// [`ModuleRegistry`].
#[cfg(test)]
pub const MOCK_MODULE_ID: ModuleId = ModuleId(0);
