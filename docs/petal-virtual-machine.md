# Petal Virtual Machine

The Petal Virtual Machine is responsible for executing your code once it has been through the following stages:

1. Lexing: Turning the source code characters into parseable tokens.
2. AST generation: Turning the tokens from the lexer into statements and expressions.
3. (TODO) Type resolving: Ensuring that all types referenced by statements and expressions exist.
4. (TODO) Type checking: Ensuring that all referenced types are valid and compatible with each other.

After all of the above steps have been performed successfully: the Petal VM will attempt to find the `main` function
of your program. If the `main` function exists, it will be called.

## The `main` function

The `main` function is a special function in the Petal language. It is considered to be the entrypoint of your
application code.

The virtual machine expects the function to have the following properties. If any of these properties are not true,
an error will be returned by the VM.

1. The function MUST have a return type of `i32`. It is not permitted to be any other type (e.g. `void`.).

2. The function MUST not have any parameters.
   
    This restriction will be removed in the future. An `Array<String>` containing the program arguments will be passed
    to the `main` function by the VM once the `Array` type has been implemented. 

### Example `main` function declaration

```go
func main() -> i32 {
    return 0;
}
```

## Scopes

Whenever a function is called by the VM, a new empty scope will be created. Once the function has finished executing,
the scope will be destroyed.

If this scope was defined using Petal syntax, it would look something like this:

```go
type Scope = struct {
    /**
     * The parent of this scope.
     */
    Scope*? parent;

    /**
     * The variables defined in this scope.
     */
    Array<Variable> variables;

    /**
     * The statements to execute at the end of this function scope.
     */
    Array<Expression> deferred;
};
```

### Variable Lookup

One of the main things that the scope is responsible for is tracking variables. 

When a variable is being read from or written to, the current scope will check if a variable exists with the provided
name. If one could not be found, its parent scope will be checked. This process repeats until a variable is found, or
until there is no parent scope to check (and in that case, the VM will return an error).

### Deferring

The `defer` keyword can be used within a function to defer execution until the function scope is destroyed. This
allows cleanup to occur without needing to copy and paste logic at each termination point within the function.

The `defer` keyword is similar to the `return` keyword, except for the fact that an expression must be provided. The
`Expression` will be captured and not evaluated until the function has finished executing.

For example:

```go
func get_return_value() -> i32 {
    print("get_return_value");
    return 0;
}

func main() -> i32 {
    defer print("This was deferred!");
    return get_return_value();
}
```

This program would result in a standard output containing the following:

```
get_return_value
This was deferred!
```

If a variable is referenced within a `defer`'s expression, it is not evaluated until the expression is executed. It
is for this reason that `defer` is not allowed within child scopes (`if`, `while`, etc.) yet.
