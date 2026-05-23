## Write cargo main features (at least 9 command)

Note:
(cargo --list)
(cargo help)
Chapter 1, pages 15-19

## What is shadowing? in which situations shadowing is useful?

Chapter 2, page 29-30
Note:
Shadowing is useful in situations where you want to perform a transformation on a value and
maintain immutability. Each shadowed variable is a new variable, allowing you to apply trans-
formations step by step without modifying the original value.

```rust
fn main() {
    let text = "  hello, world  ";
    let text = text.trim();           // &str — whitespace removed
    let text = text.to_uppercase();   // String — converted to uppercase
    let text = text.replace(",", ""); // String — punctuation removed

    println!("{text}"); // "HELLO WORLD"
}
```

## Write two examples for the case of using tuple and struct according to this explanation:

# One of the most common and idiomatic uses for tuples in Rust is to return multiple values from a function. This is often cleaner and more lightweight than defining a new struct just for a single function’s return type.

Chapter 2, page 33

## Write example for each of these types, use destructuring to access the elements. Try to print their elements with for-loop. Try to get slice from each of them.

Note:
tuple, array, slice, String (use chars() to iterate in a for loop), &str (immutable reference to a string slice. `str` is an string slice (sequence of UTF-8) resides somewhere in memory),

Chapter 2, pages 32-35

## Write example for these three types of structs (Classic, tuple, unit. Unit structs are useful for creating types that don’t need to store data but still need to implement certain traits.)

Chapter 2, pages 36-37

## Write examples for methods, associated functions and associated methods

Chapter 2, pages 39-40

## Write a method for an enum. Try to make constructor for each of the enum variants (try with implementing From trait so can use both `from` and `into` functions)

Chapter 2, page 42

## Write three example of fucntions and pass parameters with value, with reference, with mut reference

Chapter 2, pages 45-46

## Write a function that takes ownership and a function that returns ownership. Also use `clone` in argument to have a deep copy

Chapter 2, pages 47

## Write examples for each of these control flows: loop (use break, continue and label), if-else, for, while, match, if-let, while-let, let-else (can use Fibonnaci example)

Chapter 2, pages 48-52

## Write example for these type of matching (literals, variables, enums, match guards, match ranges) with option and result)

Chapter 2, pages 52-56

## use `uwrape` and `expect` for both Result and Option.

Note:
`.unwrap()` takes no arguments,
`panic!("My message")` lets you write a custom message
Rust has a dedicated method that gives you the best of both worlds: `.expect()`

Chapter 2, pages 57-59

## write an error for the Result, Err arm and try to destructure the error

Chapter 2, pages 58-59

## use unwrap_or for an Option

Note: for both `Option` and `Result` we can use:
`unwrap_or()` -> you set a default value
`unwrap_or_default() -> Rust sets a deafault value
`unwrap_or_else(||...)` -> You can run a block of code or call a function to generate the default value only if a failure actually happens
Chapter 2, pages 60

## destructure and Err with match and if let, panic (and write message -> to do it need to write) in the case of error and print the destructured error

Chapter 2, pages 61

## function can return no value, and can have no parameter; write example for each cases

Chapter 3, page 68-69

## Write closures that capture the variables from the scope environment in three variants: 1. immutable, 2.mutable reference (try to modify the environment variable value) 3. taking the ownership

Note:
If you are taking ownership of the value and want to modify it, the `mut` goes on the variable: `|mut x: String|`

If you are taking a reference to a value and want to modify it, the `mut` is part of the type: `|x: &mut String|`

Two scenarios for moving:

1. Using move keyword

```rust
fn main() {
    let name = String::from("Nima");

    let n = move || name;
    n();
    println!("{name}");
}
```

2. Moving happens by value passing (Implicit Move in Closures)

```rust
fn main() {
    let name = String::from("Nima");

    let n = |x: String| x;
    n(name);
    //borrowed of move value
    // println!("{name}");
}
```

Chapter 3, page 76-78

## Write example for these closure traits: `Fn`, `FnOnce`, `FnMut`

Note: Don't confuse the
