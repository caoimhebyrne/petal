## `petal` 🌸

This is a compiler/runtime for my language, called Petal. The plan is for this compiler to provide
enough basic features to build a self-hosted compiler.

This compiler uses LLVM to generate binaries.

```go
// This is the main function, it is called similar to how it would be called in C.
// Petal is linked with libc, meaning `main` is called from the `__start` symbol.
//
// This function expects an `i32` to be returned.
func main() -> i32 {
    // This is a variable declaration. Since it is in a function block, it can only
    // be accessed from within this function block.
    i32 my_variable = 100;

    // Values can be returned either by referencing a variable name, or writing the value inline.
    return my_variable;
}
```

### Building
1. Clone this git repository.
2. Build the compiler using the [Makefile](./Makefile).
   ```sh
   $ make build
   ```
3. If you are planning to use `clangd` within your code editor, install [Bear](https://github.com/rizsotto/Bear) and run
   the `setup-clangd` target.
   ```sh
   $ make setup-clangd
   ```

### Installation

You can install the Petal compiler for your user through the `install` target.
```sh
$ make install
```

### License

This project is licensed under the MIT license. See the [LICENSE](./LICENSE) file for more information.
