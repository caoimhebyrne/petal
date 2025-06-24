use crate::{core::location::Location, typechecker::r#type::Type};

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParameter {
    // The name of the parameter.
    pub name: String,

    // The expected value type of the parameter.
    pub expected_type: Type,

    // The location of the parameter.
    pub location: Location,
}

impl FunctionParameter {
    pub fn new(name: String, expected_type: Type, location: Location) -> Self {
        Self {
            name,
            expected_type,
            location,
        }
    }
}
