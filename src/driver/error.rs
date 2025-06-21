use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum DriverError {
    // The driver failed at the compilation stage.
    CompilationFailure,

    // The driver failed at the linking stage.
    LinkingFailure,

    // The driver failed to read/write a file.
    IOError { file_name: String },
}

impl Display for DriverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriverError::CompilationFailure => {
                write!(f, "Failed to compile, read the logs above for more information.")
            }

            DriverError::LinkingFailure => write!(f, "Failed to link, read the logs above for more information."),

            DriverError::IOError { file_name } => write!(f, "Failed to read from/write to file: {}", file_name),
        }
    }
}
