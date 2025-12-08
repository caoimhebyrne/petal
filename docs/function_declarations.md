## Function Declarations

When you declare a function in Petal, you control its name, return type, and parameter types. All of these pieces of
information influence how the function is called.

On top of the aforementioned properties, the following pieces of information are also important to a function's
declaration:

- The ID of the module that it was defined in.
- Whether the function is public or not.

For example, a function named `add` that takes two integers and returns an integer may be defined as:

```go
public func add(a: i32, b: i32) -> i32 {
    return a + b;
}
```

But, to the typechecker, this function looks more like:

```go
public func main_add_i32i32_i32(a: i32, b: i32) -> i32 {
    return a + b;
}
```

> [!NOTE]
> This name mangling does not occur on the `main` function, or functions that are marked as `extern`. 
>
> This ensures that:
> - the `main` function exists (as it is called as the entrypoint by libc).
> - functions from other libraries can be called through Petal code without their names being mangled.
>
> In the future, I may add an "external name" attribute that allows you to opt-in to Petal's function overloading
> behavior for external functions, but that is not available yet. It would look something like this:
> ```go
> @external_name("awesome_function")
> extern func awesome_function(cstr: &u8, value: i32) -> bool;
> ```

This presents a problem, though. When we call a function, we do so with the name that we define it with, and multiple
functions may have the same name...

### Resolving functions and their overloads

As mentioned in the previous section... When you call a function, you do so with the name that you declared it with:

**main.petal**:
```go
func main() -> i32 {
    return add(4, 5);
}
```

But, if the typechecker sees the function with its own verbose name (e.g. `main_add_i32i32_i32`), how do we know that
`add` is referring to `main_add_i32i32_i32` and not some variant with the same name (like `main_add_boolbool_i32`)?

Well, we can use the following rules to decide whether a given function is the subject of a function call:

1. Is this the only function with the provided "plain" name?
2. Is this the only function with the provided "plain" name, and parameter types?
3. Is this the only function with the proivded "plain" name, parameter types, and expected return type?
    - This does not apply in all cases, return type can not always be inferred.

If there are still multiple functions available after these three rules, then the function call is ambiguous, and we cannot resolve it any further.

Let's go through the following example:

**main.petal**:
```go
func add(a: i32, b: i32) -> i32 {
    return a + b;
}

func add(a: bool, b: bool) -> i32 {
    return 5;
}

func main() -> i32 {
    return add(4, 5);
}
```

In this example, we have two functions with the name `add`:

- One takes two `i32`s, and returns an `i32`.
- One takes two `bool`s, and returns an `i32`.

The `main` function calls a function called `add` and passes two integer literals.

The typechecker will see this function call, and attempt to find all functions with the provided "plain" name, in this
case, we have two matches.

The typechecker will then look at the parameter types. They do not have a *concrete* type yet, as we have no other
information. But, we do know that they are an integer literal -- we just do not know the size of that integer literal.

With this knowledge, we narrow down the search to a single candidate, which lets us move on to the final phase of
function resolution.

Once we reach a single candidate, the typechecker will ensure that the provided function is accessible from the current
module. If the function is not accessible from this module (i.e. it is marked externally and not `public`), then the
typechecker will throw an error.
