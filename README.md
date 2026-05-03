![Logo](./assets/petal-logo.svg)

# `petal` 🌸

Petal is a hobby programming language with a ~~compiler~~ transpiler (to C) written in Rust. A LLVM (or maybe
Cranelift) compiler backend will be added in the future.

## Roadmap

- [x] Basic function declarations (parameters, return types)
- [x] Signed and unsigned integer types
- [x] Variable declarations
- [x] Function calls (arguments)
- [x] Basic type-checking
- [x] Variable assignment
- [x] Named function call arguments (`myFunc(value: i32)`)
  - [x] Make positional parameters the default, with `~` signifying a named parameter.
- [x] Booleans
- [x] Control flow (if, while)
- [x] Equality operands (`==`, `!=`)
- [x] Smart type inference for integers
  - For example, `i64 a = 5;` should compile, but it does not as `5` is always treated as an `i32`. It should be
    inferred to be an `i64` (if the literal fits in the width).
- [x] Multiple module support
  - [x] Private module members by default (expose via `public` keyword)
  - [x] Name mangling
- [x] References
- [x] User defined types (`type CString = &u8`)
- [x] Structures
  - [x] Member access
  - [x] Member function declarations
  - [x] Member function calls
- [ ] Optionals (`type Optional<T> = { is_present: bool, value: T }`)
  - [ ] Short-hand via type modifiers, e.g. `?i32`.
  - [ ] Smart casting
- [ ] Basic generics
- [ ] Arrays
- ...
- [ ] Website/playground REPL
- [ ] LLVM or Cranelift backend

## Contributions

Anyone is open to making a pull request! Before you do, please ensure that the feature you are adding is documented
on a GitHub issue, and has been approved by @caoimhebyrne.

I request that any code submissions are not entirely AI written. If you are going to use AI tools on this project, then:

- Ensure that you understand the code that it is producing.
- Guide the AI using your existing knowledge and ideas.

## License

This project is licensed under the [Apache License 2.0](./LICENSE) license.
