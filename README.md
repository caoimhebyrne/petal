# `petal` 🌸

Petal is a hobby programming language with a compiler written in Rust.

## Quick Start

Provided that you have a [Petal compiler installed](./docs/installation.md), you can write and compile a Hello World
program in Petal very easily!

1. Create a file called `hello.petal`.

   ```go
   import stdlib;

   func main() -> i32 {
       println("Hello, world!");
       return 0;
   }
   ```
2. Compile a binary using the Petal compiler.

   ```
   $ petal -o ./hello ./hello.petal
   ```

3. Run the output binary!

   ```
   $ ./hello
   Hello, world!
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
