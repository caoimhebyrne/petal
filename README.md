## `mlang`

This is a compiler/runtime for my language, which doesn't have a name yet. The plan is for this compiler to provide
enough basic features to build a self-hosted compiler.

```rust
// This is a variable declaration. Since it is in the top level of the file,
// it can be accessed by any scope within this file.
i32 my_variable = 0;

// This is the main function, it is executed whenever a file is interpreted
// in this language.
// The main function must be public, this allows it to be visible from other
// files / modules.
public func main() {
    i32 another_variable = my_variable * 2;
    print(another_variable);
}
```

### Building
1. Clone this git repository.
2. Build the compiler using the [Makefile](./Makefile).
   ```sh
   $ make build
   ```

### License

This project is licensed under the MIT license. See the [LICENSE](./LICENSE) file for more information.
