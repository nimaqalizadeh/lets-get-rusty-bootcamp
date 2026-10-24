# The Rust Programming Handbook

# Chapter 2

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

#\* One of the most common and idiomatic uses for tuples in Rust is to return multiple values from a function. This is often cleaner and more lightweight than defining a new struct just for a single function’s return type.

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

# Chapter 3

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

Note: Don't confuse the difference between capturing a variable and taking an argument in closures.
The way a closure captures its environment determines which of three special traits it imple-
ments: FnOnce, FnMut, or Fn.

Chapter 3, page 78

## Given this vector `let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];` sum the square of elements that are greater than 3 and even (a chain of using iter, filter, map and sum)

Chapter 3, page 79

# Chapter 4

Use Alice Ryhl insight:

**The chain:** borrowing creates a reference, and every reference carries a lifetime. So lifetimes only matter when references are involved.

1. **Borrowing → Reference:** A _reference_ is a compile-time-only pointer to a value — no runtime checks. Creating one is called _borrowing_.

2. **Reference → Lifetime:** Every reference has a **lifetime** — a window of time where the reference is allowed to be used (typically the body of a function or just a few lines). The compiler makes sure the reference is never used outside that window.

3. **The borrow checker** enforces two rules over these references:
   - (1) A reference cannot outlive the scope it was borrowed from — if borrowed from a local variable, its lifetime is bounded by that variable's scope.
   - (2) At any moment, you can have either **one mutable borrow** or **any number of immutable borrows** — never both at the same time.

## Write example for Ownership principals: Each value has a single owner and Only one owner at a time. When the owner goes out of scope, the value is dropped

Chapter 4, page 88

## Write example for types that manage resources on the heap (String, Vec, Box) to show that Rust’s default behavior upon assignment is to move ownership

Note: move = shallow copy of the pointer, length, and capacity

Chapter 4, page 89

## What is the exeption of moving in the case of assignment? (Copy trait)

Chapter 4, page 90

## What is the solution if you want copy the type instead of default move? (Explicit duplication: The Clone trait (Deep copy))

Note: Unlike Copy, which is an implicit, bit-for-bit copy, clone() is explicit

1. Move = Shallow Copy + Invalidation -> copies the pointer, the length, and the capacity + immediately invalidates the original variable

2. Copy = Shallow Copy (No Invalidation) -> Because they don't have any pointers to heap memory, doing a shallow copy is perfectly safe! Therefore, Rust does the exact same shallow copy as a move, but does not invalidate the original variable

3. Clone = Deep Copy (Usually) -> It does the shallow copy of the stack data, but then it also requests brand new memory on the heap and copies all the actual text or data over.

Exception: There are a few advanced exceptions, like `Rc` or `Arc`, where `.clone()` just increments a counter instead of doing a deep copy, but for standard data, Clone means deep copy

**stack-only is shallow, stack+heap is deep**

Chapter 4, page 92

## What is the difference between a reference and pointer in Rust?

Note: Reference is governed by Rust’s strict compile-time borrow checker. This means that while a reference allows you to access data without taking full responsibility for it, the compiler guarantees that the reference will always be valid and will not lead to dangerous situations such as data races

In the case of having a dangling pointer (allocate memory, create a pointer to it, and then free the memory), you can commit two major action with it:

1. **Double-free bugs** -> pass the dangling pointer to `free(dangling_ptr)`

2. **Use after free errors** -> read or write data to the target of pointer (`println!{*dangling_ptr}`)

Rust’s compiler prevents this entire class of bugs at compile time through its analysis of lifetimes.

**Data races** -> Multi-threading

Chapter 4, page 100-103

## Common patterns and idioms in Rust: Ownership,borrowing, and references. Write example for each case:

### 1. Borrowing for read-only access (write an struct with String fields and a function that just borrow the object. Then write another struct that just have primitive fields)

Note: There is a very common misconception that wether struct is on heap or stack?

The struct itself lives on the stack in both cases. The types of the fields inside the struct do not change where the struct is placed in memory. What changes is whether the struct points to the heap or stack.

### 2. Mutable borrowing for modification

### 3. Returning references with borrowing

Note: Returning references from functions while maintaining ownership of the caller is a powerful pattern. This allows you to give controlled access to parts of your data without transferring
ownership or needing to copy data.

The Aha moment:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
// Lifetime annotation '<'a>' ensures that the returned reference
// lives at least as long as the shortest of the input references.
if x.len() > y.len() {
x
} else {
y
    }
}
```

The `longest` function says, I give two reference from to `str` in a scope and return a reference to one of them. The compiler concern is that the returened reference should be valid at that scope.

## Now try to write a code that uses above `longest` function and make compliler shout at you :)

Chapter 4, pages 106-108

## Write example for these pitfalls:

### 1. Forgetting ownership has moved

### 2. Multiple mutable references, or both mutable and immutable reference (try to change their order and see the result)

### 3. Dangling references

Note: The fundamental (avoid dangling references) rule is that you cannot return a reference to a value that was created inside the function. The safest and most common approach is to return an owned value (such as `String`, `Vec<T>`, etc.)

### 4. Unnecessary clones

Note: you should only call .clone() when you have a clear and deliberate need for a separate, independent copy of the data and are willing to accept the potential performance cost

Chapter 4, pages 109-111

# Chapter 5

## Write example for these cases:

### 1. Field initialization shorthand syntax for structs

```rust
User {
email, // Shorthand for email: email
username, // Shorthand for username: username
}
```

### 2. Reading field values

### 3. Modifying struct fields

Note: Immutability in Rust applies to the entire binding

### 4. Updating struct instances (use .. to indicate that the remaining fields should be copied from another instance. if a field’s type does not implement `Copy` (such as `String`), ownership of that field’s data is moved)

Note: The reason types like `String` don't have copy trait and should transfer ownership:
Types that manage resources, such as `String`, which owns data on the heap, do not implement `Copy`. A simple bitwise copy of String would result in two variables pointing to and believing they own the same heap memory, which would lead to a “double-free” error when both are dropped. For these non-Copy fields, the struct update syntax performs a move, transferring ownership.

Chapter 5, pages 117-125

## Methods for structs. Write example for each case

• &self: Borrows the instance immutably (read-only access)
• &mut self: Borrows the instance mutably (read-write access)
• self: Takes ownership of the instance (consumes it)

Note: Rust handles the borrowing (&self or &mut self) automatically through a feature called automatic referencing and dereferencing (instead of `Rectangle::area(&r)` use `r.area()`):

```rust
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}
fn main() {
    let r = Rectangle {
        width: 2,
        height: 5,
    };
    r.area();
    Rectangle::area(&r);
}
```

Chapter 5, pages 126

## What is an associated function?

Note: associated functions are functions that are associated with the struct type itself but do not take self as their first parameter.

Chapter 5, page 127

## Use cases for unit-like structs:

Note: Unit-like structs are typically used in situations where the existence of a type is more important than the data it holds. Some common use cases include the following:

### Marker traits: Sometimes you want to mark types with a specific trait to indicate that they conform to a particular behavior. Unit-like structs can be used to implement such traits without storing any data.

### Type-level programming: In more advanced Rust programming, unit-like structs can be used in type-level computations and generic programming scenarios where the type itself carries meaning.

Chapter 5, page 130

## Define a tuple structs (create instance, access to fields, use it in a function)

Chapter 5, pages 130-131

## Ownership of struct fields (write examples with moving, borrowing, mutable borrowing)

Note: Ownership is a central concept in Rust that ensures memory safety without needing a garbage collector

Chapter 5, pages 132-134

## Customizing `Debug` trait for struct: `impl fmt::Debug for Struct`

```rust
use std::fmt;

struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}

impl fmt::Debug for Rectangle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Rectangle {{ w: {}, h: {} }} ", self.width, self.height)
    }
}
fn main() {
    let r = Rectangle {
        width: 2,
        height: 5,
    };
    r.area();
    Rectangle::area(&r);
    println!("{r:?}");
}
```

Chapter 5, page 136

## Define an `enum` which the fields are unit struct, tuple struct, struct

Chapter 5, page 140

## Write example for the Rust idiomatic way to handle operations that might return a value or nothing at all (`Option<T>` enum)

Chapter 5, page 144

## Write example for an struct with enum fields. Define associate function for enum (suppose you have a network device that has an ID and name, but can be in various operational states - Online, Offline, Connecting (with number of attempts, Maintenance))

Chapter 5, page 145

## Define a tuple with type annotation and try to access its elements by 1. Destructuring 2. Direct Indexing

Chapter 5, page 149

## Write example for these tuples usecases (a good example is working with 2D or 3D spaces and distance (one time implement with struct and one time implement with tuples)):

• Returning multiple values from a function without the need to define a struct (use both struct and tuple to see the difference)
• Grouping related values for iteration or other operations where the context makes their meaning clear
• Passing multiple values as a single parameter to a function

Note: You can't iterate over a single tuple (its elements may have different types). The bullet means iterating over a **collection of tuples**, where destructuring restores the names of each position:

```rust
let students = [("Alice", 92), ("Bob", 78)];
for (name, score) in &students {
    println!("{name}: {score}");
}
```

Same idea applies to `zip`, `enumerate`, and `HashMap` iteration — the tuple positions are obvious from context, so a struct isn't needed:

```rust
for (i, v) in vec.iter().enumerate() { /* (index, value) */ }
for (k, v) in map.iter()             { /* (key, value)   */ }
```

Chapter 5, page 150-155

## What is a crate?

# Chapter 6

## Why error handling matters?

• Robustness: Programs must handle unexpected inputs, system issues (such as missing
files or denied permissions), and other anomalies without crashing. Good error handling makes software more resilient.

• User experience: Clearly reporting errors helps users (and fellow developers!) understand what went wrong and potentially how to fix it.

• Maintainability: A consistent approach to error handling makes code easier to read, understand, and modify over time.

Chapter 6, page 169

## Rust errors primarily fall into two categories:

- Recoverable errors -> Explicit handling of these, usually via the Result<T, E> type

- Unrecoverable errors (panic) -> `panic!` macro by default, unwinds the stack (cleaning up memory as it goes) and quits the current thread

Chapter 6, page 171

## What does Rust do to manage error categories?

Has defined enums in the standard library:

- Result<T, E>: Used for operations that might succeed (returning a value of type T in the Ok variant) or fail (returning an error of type E in the Err variant)

- Option<T>: Used when a value might be present (the Some(T) variant) or absent (the
  None variant

Chapter 6, page 171

## Write an example for division by 0 function and handle the return value by Result

Chapter 6, page 172

## When to use Result vs Option?

Use Result<T, E> when an operation can fail for a specific reason you want to communicate to the caller via the error type, E. It’s for recoverable errors.

• Use Option<T> when the absence of a value is a possible and normal outcome (e.g., search failed or optional data not provided), and the reason for absence isn’t important or is
implicit.

Chapter 6, page 175

## Why do we use `unwrap` or `expect` instead of `match`ing on `Result` or `Option`

`unwrap()` and `expect()` are **shortcuts** for a `match` that panics on the failure arm. Instead of writing the `match` + `panic!` by hand, you call these methods directly:

```rust
// These two are equivalent:
let v = result.unwrap();

let v = match result {
    Ok(v)  => v,
    Err(e) => panic!("called `unwrap()` on an `Err` value: {e:?}"),
};
```

The difference between them:

- `unwrap()` — panics with a **generic** message (`"called `unwrap()`on an`Err` value: ..."`) for example:

```bash
thread 'main' panicked at src/main.rs:3:17:
called `Result::unwrap()` on an `Err` value: "file not found"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

- `expect("msg")` — panics with **your custom message**, so failures are easier to diagnose. Prefer `expect` in real code; reserve `unwrap` for prototypes and tests. For example:

```bash
thread 'main' panicked at src/main.rs:3:17:
config file should exist at startup: "file not found"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Caution: unwrap and expect turn a potentially recoverable error (Err or None) into an unrecoverable one (panic)
