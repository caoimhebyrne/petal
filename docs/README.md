# Welcome to the Petal Language Documentation!

> [!WARNING]
> Petal is still an experimental language and is under constant development, this documentation may be out of date.
> I recommend taking a look at the [language examples](../examples) if you are interested in up-to-date information.\

## Table of Contents

1. [Functions](#functions)
2. [Statements](#statements)

## Functions

### Declaration

A function is defined in Petal using the `func` keyword. The general form of a function is something like this:
```swift
func my_function(parameter: parameter_type) -> return_type {
    ...
}
```

In the above example, the parameters between the parenthesis and the `-> return_type` are optional, a function that
has no parameters which returns void looks like this:
```swift
func my_function() {

}
```

### Function Body

Between the opening and closing braces of a function declaration is the function's body. This is a list of statements
that are seperated by semicolons.

## Statements

There are many statement types in the Petal language.

### Function Declarations

[See 'Functions'](#functions).

### Variable Declarations

A variable declaration allows a value of any type to be referenced by its name within a scope. The general form of
a variable declaration looks like this:
```swift
value_type name = <initial_value>;
```

For example, an `i32` with an initial value of `5` would be declared as:
```swift
i32 my_variable = 5;
```

### Function Calls

A function can be called by its name, followed by a set of parenthesis surrounding the arguments to be passed to the
function.

#### A function with no parameters:
```swift
my_function();
```

#### A function with three parameters, all of i32:
```swift
my_function(5, 1, 2);
```
