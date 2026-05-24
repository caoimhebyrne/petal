// The tests within this module ensure that the compiler does not regress in terms of _compilation ability_ and error
// reporting.
//
// This does not check for correctness in terms of generated code or behavior (yet). That will be added once another
// IR layer is added.

use pretty_assertions::assert_eq;

use crate::{
    core::error::Error,
    integration_tests::runner::TestCaseRunner,
};

mod runner;

// The result of a test case.
type TestResult<T> = Result<T, Box<dyn Error>>;

fn init_env_logger() {
    // Using `try_init` allows us to ignore an error which can occur when trying to initialize the logger more than
    // once during test execution.
    let _ = env_logger::builder().is_test(true).filter_level(log::LevelFilter::Trace).try_init();
}

/// Asserts that the provided code snippet successfully passes type-checking.
fn assert_successful_type_check(snippet: &str) -> TestResult<()> {
    init_env_logger();

    let mut runner = TestCaseRunner::with_prelude()?;
    runner.add_module_from_str(snippet)?;
    runner.compile()
}

/// Asserts that the provided code snippet returns an error which matches the provided string.
fn assert_failing_type_check(snippet: &str, expected_error: &str) -> TestResult<()> {
    init_env_logger();

    let mut runner = TestCaseRunner::with_prelude()?;
    runner.add_module_from_str(snippet)?;

    let Err(error) = runner.compile() else {
        panic!("Compilation was successful, but expected an error: '{expected_error}'");
    };

    assert_eq!(error.to_string(), expected_error);
    Ok(())
}

mod function_declaration {
    use super::*;

    #[test]
    fn empty_body_with_no_parameters() -> TestResult<()> {
        assert_successful_type_check("func foo() {}")
    }

    #[test]
    fn empty_body_with_parameters() -> TestResult<()> {
        assert_successful_type_check("func foo(a: i32, b: i32) {}")
    }

    #[test]
    fn empty_body_with_named_parameters() -> TestResult<()> {
        assert_successful_type_check("func foo(~a: i32, ~b: i32) {}")
    }

    #[test]
    fn empty_body_with_mixed_parameters() -> TestResult<()> {
        assert_successful_type_check("func foo(a: i32, ~b: i32) {}")
    }

    #[test]
    fn empty_body_with_return_type() -> TestResult<()> {
        assert_successful_type_check("func foo() -> i32 {}")
    }

    #[test]
    fn fails_with_invalid_return_type() -> TestResult<()> {
        assert_failing_type_check(
            "func foo() -> this_does_not_exist {}",
            "Cannot find type named 'this_does_not_exist'",
        )
    }
}

mod function_call {
    use super::*;

    #[test]
    fn with_no_arguments() -> TestResult<()> {
        assert_successful_type_check(
            r"
            func foo() {}

            func bar() {
                foo();
            }
            ",
        )
    }

    #[test]
    fn with_positional_arguments() -> TestResult<()> {
        assert_successful_type_check(
            r"
            func foo(a: i32, b: i32) {}

            func bar() {
                foo(5, 10);
            }
            ",
        )
    }

    #[test]
    fn with_named_arguments() -> TestResult<()> {
        assert_successful_type_check(
            r"
            func foo(~a: i32, ~b: i32) {}

            func bar() {
                foo(a: 5, b: 10);
            }
            ",
        )
    }

    #[test]
    fn with_out_of_order_named_arguments() -> TestResult<()> {
        assert_successful_type_check(
            r#"
            func foo(~a: i32, ~b: str) {}

            func bar() {
                foo(b: "", a: 2);
            }
            "#,
        )
    }

    #[test]
    fn with_mixed_arguments() -> TestResult<()> {
        assert_successful_type_check(
            r"
            func foo(a: i32, ~b: i32) {}

            func bar() {
                foo(5, b: 10);
            }
            ",
        )
    }

    #[test]
    fn fails_with_type_mismatch() -> TestResult<()> {
        assert_failing_type_check(
            r"
            func foo(a: bool) {}

            func bar() {
                foo(123);
            }
            ",
            "Expected a value of type 'bool', but received a value of type 'u8'",
        )
    }

    #[test]
    fn fails_with_too_many_arguments() -> TestResult<()> {
        assert_failing_type_check(
            r"
            func foo(a: i32) {}

            func bar() {
                foo(123, 456);
            }
            ",
            "Expected 1 argument in function call, but got 2 arguments",
        )
    }

    #[test]
    fn fails_with_positional_argument_for_named_parameter() -> TestResult<()> {
        assert_failing_type_check(
            r"
            func foo(~a: i32) {}

            func bar() {
                foo(123);
            }
            ",
            "A named argument must be provided for parameter 'a'",
        )
    }

    #[test]
    fn fails_with_named_argument_for_positional_parameter() -> TestResult<()> {
        assert_failing_type_check(
            r"
            func foo(a: i32) {}

            func bar() {
                foo(a: 123);
            }
            ",
            "A positional argument must be provided for parameter 'a'",
        )
    }

    #[test]
    fn fails_with_type_mismatch_for_named_argument() -> TestResult<()> {
        assert_failing_type_check(
            r"
            func foo(a: i32, ~b: bool) {}

            func bar() {
                foo(123, b: 456);
            }
            ",
            "Expected a value of type 'bool', but received a value of type 'u8'",
        )
    }
}

mod variable_assignment {
    use super::*;

    #[test]
    fn with_i32_type_and_value() -> TestResult<()> {
        assert_successful_type_check(
            r"
            func foo() {
                bar: i32 = 0;
                bar = 2;
            }
            ",
        )
    }

    #[test]
    fn with_dereference_i32_type() -> TestResult<()> {
        assert_successful_type_check(
            r"
            func foo(bar: &i32) {
                @bar = 5;
            }
            ",
        )
    }

    // todo: struct tests?
    #[test]
    fn with_struct_field_assignment() -> TestResult<()> {
        assert_failing_type_check(
            r#"
            type Foo = struct { value: str };

            func bar() {
                foo: Foo = { .value = "" };
                foo.value = 4;
            }
            "#,
            "Expected a value of type 'CompileTimeStr', but received a value of type 'u8'",
        )
    }

    #[test]
    fn fails_with_dereference_of_non_reference_value() -> TestResult<()> {
        assert_failing_type_check(
            r"
            func foo(bar: i32) {
                @bar = 5;
            }
            ",
            "You cannot dereference this expression type, it must be a reference type",
        )
    }

    #[test]
    fn fails_with_type_mismatch() -> TestResult<()> {
        assert_failing_type_check(
            r#"
            func foo() {
                bar: str = "";
                bar = 3;
            }
            "#,
            "Expected a value of type 'CompileTimeStr', but received a value of type 'u8'",
        )
    }

    #[test]
    fn fails_with_invalid_variable_name() -> TestResult<()> {
        assert_failing_type_check(
            r"
            func foo() {
                bar = 3;
            }
            ",
            "Could not resolve a value for identifier 'bar'",
        )
    }
}

mod variable_declaration {
    use super::*;

    #[test]
    fn with_i32_value() -> TestResult<()> {
        assert_successful_type_check(
            r"
            func foo() {
                value: i32 = 0;
            }
            ",
        )
    }

    #[test]
    fn fails_with_type_mismatch() -> TestResult<()> {
        assert_failing_type_check(
            r"
            func foo() {
                value: bool = 2;
            }
            ",
            "Expected a value of type 'bool', but received a value of type 'u8'",
        )
    }
}

mod r#return {
    use super::*;

    #[test]
    fn without_value_in_void_func() -> TestResult<()> {
        assert_successful_type_check(
            r"
            func foo() {
                return;
            }
            ",
        )
    }

    #[test]
    fn with_value_in_i32_func() -> TestResult<()> {
        assert_successful_type_check(
            r"
            func foo() -> i32 {
                return 2;
            }
            ",
        )
    }

    #[test]
    fn fails_with_value_in_void_fn() -> TestResult<()> {
        assert_failing_type_check(
            r"
            func foo() {
                return 2;
            }
            ",
            "Expected a value of type 'void', but received a value of type 'u8'",
        )
    }

    #[test]
    fn fails_with_value_type_mismatch() -> TestResult<()> {
        assert_failing_type_check(
            r"
            func foo() -> bool {
                return 2;
            }
            ",
            "Expected a value of type 'bool', but received a value of type 'u8'",
        )
    }
}
