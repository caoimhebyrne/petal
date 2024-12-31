## Functions

A function can be defined using the `func` keyword.
```rs
func my_function() {

}
```
This function takes no paramaters, and has an inferred return type of `void`. To specify a return type, use `->` after
the parameter list's closing parenthesis.
```rs
func my_better_function() -> i32 {
    return 5;
}
```
Values can be returned from a function using the `return` keyword. If your function returns `void`, you do not have to
provide a value.

Parameters can be defined between the parenthesis after the function name.
```rs
func my_awesome_function(a: i32, b: i32) -> i32 {
    return a * b;
}
```
This function takes in two parameters, `a` and `b`, with both being the `i32` type.
