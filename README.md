## `petal` 🌸

This is the compiler for my language, Petal. The plan for this compiler is for it to provide enough features to build
a self-hosted compiler. LLVM is used for code-generation.

```go
// This is the main function, it is called similar to how it would be called in C.
// Petal programs are linked with libc, meaning `main` is called from the `__start` 
// symbol.
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
2. Build the compiler using `cargo build`

## TODO
- [ ] Type aliases (`type <name> = <definition>`, e.g: `type string = &i8`)
- [ ] Ditch LLVM and make my own IR?
- [ ] Improve control flow
  - [ ] Add `else` blocks
  - [ ] While loops
- [ ] Importing other modules
- [ ] Structures
- [ ] Standard library

### License

This project is licensed under the MIT license. See the [LICENSE](./LICENSE) file for more information.
