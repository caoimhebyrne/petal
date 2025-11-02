/// The options that are passed to a codegen driver to change its execution behaviour.
#[derive(Debug, Clone)]
pub struct DriverOptions {
    /// The name of the module that is being compiled.
    pub module_name: String,

    /// Whether the driver should output an intermediate representation of its code before compiling the object file.
    pub dump_bytecode: bool,
}
