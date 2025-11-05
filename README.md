# `petal` 🌸

Petal is a hobby programming language with a compiler written in Rust.

## Usage

1. The compiler itself can be built by cloning this repository and running `cargo build --release`.
2. A petal file can be compiled using the created `petal` binary.
   ```shell
   $ ./target/release/petal ./examples/00_init.petal
   ```

## Examples

Some examples of Petal can be found in the [examples](./examples) directory.

## Tests

A Python test runner exists which ensures that certain Petal files execute and produce the correct output. Some of the
test cases also ensure that the compiler provides accurate errors when the code is incorrect (e.g. mismatched types,
etc.).

This test suite (and its runner) is run on each commit push, can be found in the [tests](./tests) directory.

If you wish to run the test suite manually, you can do so by running the following command:

```shell
$ cargo build && python3 ./tests/test-runner.py
```

Ensure that you have a recent-ish version of Python 3 installed (the one that comes wth macOS is too old for example).

## License

This project is licensed under the [MIT license](./LICENSE).
