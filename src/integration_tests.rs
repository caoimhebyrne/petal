use crate::{
    ast::ASTParser,
    core::error::Error,
    lexer::Lexer,
    module::ParsedModule,
    module_registry::create_mock_module_id,
};

fn compile(src: &str) -> Result<(), Box<dyn Error>> {
    let mut parsed_modules: Vec<ParsedModule> = vec![];

    // Prelude module
    {
        let module_id = create_mock_module_id(0);
        let mut lexer = Lexer::new(module_id, include_str!("../prelude/str.petal"));
        let tokens = lexer.parse()?;

        let statements = ASTParser::new_and_parse(module_id, tokens)?;
        parsed_modules.push(ParsedModule::new(module_id, statements));
    }

    // Source code
    {
        let module_id = create_mock_module_id(1);
        let mut lexer = Lexer::new(module_id, src);
        let tokens = lexer.parse()?;

        let statements = ASTParser::new_and_parse(module_id, tokens)?;
        parsed_modules.push(ParsedModule::new(module_id, statements));
    }

    Ok(())
}

fn compile_expecting_success(src: &str) {
    if let Err(error) = compile(src) {
        panic!("Unexpected error during compilation: '{}'", error);
    }
}

fn compile_expecting_error(src: &str, contains_message: &str) {
    let error = match compile(src) {
        Ok(_) => panic!("Expected compilation to fail, but compilation succeded?"),
        Err(error) => error,
    };

    let error_string = error.to_string();
    if !error_string.contains(contains_message) {
        panic!(
            "Expected compilation error to contain '{contains_message}', but error did not contain it: '{error_string}'"
        )
    }
}

mod functions {
    use super::*;

    #[test]
    fn compiles_void() {
        compile_expecting_success(
            r#"
            func foo() {}
            "#,
        );
    }

    #[test]
    fn compiles_explicit_return_type() {
        compile_expecting_success(
            r#"
            func foo() -> i32 {
                return 0;
            }
            "#,
        );
    }

    #[test]
    fn compiles_parameter() {
        compile_expecting_success(
            r#"
            func foo(a: i32) -> i32 {
                return a;
            }
            "#,
        );
    }

    #[test]
    fn compiles_parameters() {
        compile_expecting_success(
            r#"
            func foo(a: i32, b: i32) -> i32 {
                return a + b;
            }
            "#,
        );
    }

    #[test]
    fn compiles_named_parameters() {
        compile_expecting_success(
            r#"
            func foo(~a: i32, ~b: i32) -> i32 {
                return a + b;
            }

            func bar() {
                foo(a: 3, b: 2);
            }
            "#,
        );
    }

    #[test]
    fn compiles_member_function() {
        compile_expecting_success(
            r#"
            type Foo = struct {
                value: i32,
            };

            func Foo.bar(this: &Foo) {
            }

            func baz() {
                foo: Foo = { .value = 5 };
                foo.bar();
            }
        "#,
        );
    }

    #[test]
    fn compiles_member_function_with_this_type() {
        compile_expecting_success(
            r#"
            type Foo = struct {
                value: i32,
            };

            func Foo.bar(this: &This) {
            }

            func baz() {
                foo: Foo = { .value = 5 };
                foo.bar();
            }
        "#,
        );
    }

    #[test]
    fn doesnt_compile_with_named_parameters_and_positional_args() {
        compile_expecting_error(
            r#"
            func foo(~a: i32, ~b: i32) -> i32 {
                return a + b;
            }

            func bar() {
                foo(3, 2);
            }
            "#,
            "No value was provided for parameter named 'a' in call to function 'foo'",
        );
    }
}

mod variable_declaration {
    use super::*;

    #[test]
    fn compiles_i32() {
        compile_expecting_success(
            r#"
            func main() {
                foo: i32 = 0;
            }
            "#,
        );
    }

    #[test]
    fn compiles_reference() {
        compile_expecting_success(
            r#"
            func main() {
                foo: i32 = 10;
                bar: &i32 = &foo;
            }
            "#,
        );
    }

    #[test]
    fn compiles_optional() {
        compile_expecting_success(
            r#"
            func main() {
                foo: ?i32;
            }
            "#,
        );
    }

    #[test]
    fn compiles_optional_with_default_value() {
        compile_expecting_success(
            r#"
            func main() {
                foo: ?i32 = 0;
            }
            "#,
        );
    }

    #[test]
    fn doesnt_compile_without_initial_value() {
        compile_expecting_error(
            r#"
            func main() {
                foo: i32;
            }
            "#,
            "A variable declaration for a non-optional type must have an initial value",
        );
    }

    #[test]
    fn doesnt_compile_with_type_mismatch() {
        compile_expecting_error(
            r#"
            func main() {
                foo: i8 = 65535;
            }
            "#,
            "Unable to assign value of type 'i16' to variable of type 'i8'",
        );
    }
}

mod variable_assignment {
    use super::*;

    #[test]
    fn compiles_i32() {
        compile_expecting_success(
            r#"
            func main() {
                foo: i32 = 5;
                foo = 2;
            }
            "#,
        );
    }

    #[test]
    fn compiles_reference() {
        compile_expecting_success(
            r#"
            func main() {
                foo: i32 = 5;
                bar: i32 = 10;

                baz: &i32 = &foo;
                baz = &bar;
            }
            "#,
        );
    }

    #[test]
    fn compiles_optional() {
        compile_expecting_success(
            r#"
            func main() {
                foo: ?i32;
                foo = 10;
            }
            "#,
        );
    }

    #[test]
    fn doesnt_compile_with_type_mismatch() {
        compile_expecting_error(
            r#"
            func main() {
                foo: i8 = 0;
                foo = 65535;
            }
            "#,
            "Unable to assign value of type 'i16' to variable of type 'i8'",
        );
    }
}

mod structures {
    use super::*;

    #[test]
    fn compiles_empty_struct() {
        compile_expecting_success(
            r#"
            type Foo = struct {};

            func main() {
                foo: Foo = {};
            }
            "#,
        );
    }

    #[test]
    fn compiles_struct_with_member() {
        compile_expecting_success(
            r#"
            type Foo = struct {
                bar: i32,
            };

            func main() {
                foo: Foo = { .bar = 2 };
            }
            "#,
        );
    }

    // https://github.com/caoimhebyrne/petal/issues/1
    #[test]
    fn compiles_struct_with_uninitalized_optional_member() {
        compile_expecting_success(
            r#"
            type Foo = struct {
                bar: i32,
                baz: ?i32,
            };

            func main() {
                foo: Foo = { .bar = 4 };
            }
            "#,
        );
    }

    #[test]
    fn compiles_struct_with_initalized_optional_member() {
        compile_expecting_success(
            r#"
            type Foo = struct {
                bar: i32,
                baz: ?i32,
            };

            func main() {
                foo: Foo = { .bar = 4, .baz = 2 };
            }
            "#,
        );
    }

    #[test]
    fn compiles_struct_with_nested_struct() {
        compile_expecting_success(
            r#"
            type Foo = struct {
                bar: i32,
            };

            type Baz = struct {
                foo: Foo,
            };

            func main() {
                baz: Baz = { .foo = { .bar = 2 } };
            }
            "#,
        );
    }

    // https://github.com/caoimhebyrne/petal/issues/2
    #[test]
    fn compiles_struct_initialization_with_out_of_order_fields() {
        compile_expecting_success(
            r#"
            type Foo = struct {
                bar: bool,
                baz: u8,
            };

            func main() {
                foo: Foo = { .baz = 45, .bar = true };
            }
            "#,
        );
    }

    #[test]
    fn doesnt_compile_with_incomplete_initialization() {
        compile_expecting_error(
            r#"
            type Foo = struct {
                bar: i32,
            };

            func main() {
                foo: Foo = {};
            }
            "#,
            "The structure type 'Foo' has non-optional field 'bar' that was not provided in the initializer",
        );
    }

    #[test]
    fn doesnt_compile_with_initialization_type_mismatch() {
        compile_expecting_error(
            r#"
            type Foo = struct {};

            type Bar = struct {
                foo: Foo,
            };

            func main() {
                bar: Bar = { .foo = 0 };
            }
            "#,
            "Expected type '<structure 1>', but got 'u8'",
        );
    }

    #[test]
    fn doesnt_compile_with_field_mismatch() {
        compile_expecting_error(
            r#"
            type Foo = struct {
                bar: i32,
            };

            func main() {
                foo: Foo = { .bar = 1, .baz = 2 };
            }
            "#,
            "Structure initialization had 2 field(s), but structure declaration has 1 field(s)",
        );
    }
}
