# Generics

Generics allow you to define structs, enums, functions, methods, traits, trait
implementations, and type aliases with generic types that are substituted for
concrete types at compile time.

## Conventions for naming generics

- Using uppercase letters and start from `T` (T for Type), then `U`, `V`, `W`, ...
- Using `T0`, `T1`, `T2`, ...
- Using CamelCase names ending in `Type` like `PayloadType`

## Generics in `impl` blocks

Use `<T>` both after `impl` and the struct name:

```rust
impl<T> BrowserCommand<T> { ... }
```

- `impl<T>` — **declares** `T` as a generic placeholder ("any type")
- `BrowserCommand<T>` — **uses** that `T` to specify which version of the struct is being implemented

**Why can't Rust just look at `BrowserCommand<T>` and infer `T` is generic?** Because of ambiguity — Rust parses `impl<...>` before looking up what `BrowserCommand` is. At that point it needs to know: is `T` a new type variable or a concrete type that exists somewhere? Both are valid:

```rust
struct T;                      // a real concrete type named T
struct BrowserCommand<T> { payload: T }

impl BrowserCommand<T> { ... }   // which T? the struct? or a generic?
```

The explicit `impl<T>` removes the ambiguity — it declares your intent upfront.

Without `impl<T>`, Rust treats `T` as a concrete named type and errors with `cannot find type 'T' in this scope`. Rust would look for a definition like:

```rust
struct T;
enum T { ... }
type T = String;
```

Since none of those exist, it errors: `cannot find type 'T' in this scope`.

To implement methods only for a specific concrete type, omit `impl<T>` and use the type directly:

```rust
impl BrowserCommand<String> { ... }  // only available when T = String
```

## Generics in free functions

Same pattern as `impl<T>` — declare `T` after the function name, then use it in parameters or return type:

```rust
fn serialize_payload<T>(payload: &T) -> String {
    // works for any type T
}
```

Rust infers `T` from the argument at the call site:

```rust
serialize_payload(p1);  // T = String
serialize_payload(p2);  // T = i32
```

## Generics in the standard library

Rust's standard library uses generics extensively. Two common examples:

```rust
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

- `Option<T>` — represents a value that may or may not exist (`Some(T)` or `None`)
- `Result<T, E>` — represents either success (`Ok(T)`) or failure (`Err(E)`), with two type parameters

## Generics under the hood:

**Monomorphization** (from Greek: _mono_ = one, _morph_ = form) is the process the Rust compiler uses to handle generics. It takes your generic, polymorphic code (code that can take many forms) and turns it into specific, monomorphic code (code with exactly one form) during compilation.

Instead of figuring out types at runtime, the Rust compiler looks at every specific type you pass into a generic function or struct, and **generates a brand-new, dedicated copy of that code for each type**.

Here is how it works under the hood.

### 1. What You Write (The Generic Code)

Let's say you write a simple generic function that returns the value passed to it, and you call it with an integer and a float:

```rust
// 1. The generic function definition
fn print_and_return<T: std::fmt::Debug>(value: T) -> T {
    // T: std::fmt::Debug is a trait bound — it constrains T to only accept
    // types that implement the Debug trait. This is required because {:?}
    // (the debug formatter) calls Debug's fmt method. Without this bound,
    // Rust wouldn't know that T supports {:?} and would refuse to compile.
    println!("{:?}", value);
    value
}

fn main() {
    // 2. Calling it with different types
    let a = print_and_return(5);       // called with i32
    let b = print_and_return(3.14);    // called with f64
}
```

### 2. What the Compiler Generates (The Monomorphized Code)

When you run `cargo build`, the compiler notices you used `print_and_return` with an `i32` and an `f64`. It completely removes the generic `<T>` version and replaces it with exact, type-specific functions.

Behind the scenes, the compiled code looks something like this:

```rust
// The compiler auto-generates a version specifically for i32
fn print_and_return_i32(value: i32) -> i32 {
    println!("{:?}", value);
    value
}

// The compiler auto-generates a version specifically for f64
fn print_and_return_f64(value: f64) -> f64 {
    println!("{:?}", value);
    value
}

fn main() {
    // Your calls are replaced with the specific functions
    let a = print_and_return_i32(5);
    let b = print_and_return_f64(3.14);
}
```

---

### The Trade-offs of Monomorphization

Rust relies heavily on this concept because it aligns with Rust's philosophy of **zero-cost abstractions**. However, it comes with a specific set of trade-offs.

| **Feature**               | **The Impact**                                                                                                                                                             |
| ------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Execution Speed (Pro)** | Extremely fast. Because the exact type is known, there is no runtime overhead (like vtable lookups). It uses **static dispatch**.                                          |
| **Optimization (Pro)**    | The compiler can heavily optimize the code. If it knows a function is only doing `i32` math, it can apply CPU-specific integer optimizations or inline the code perfectly. |
| **Compile Times (Con)**   | Slower compilation. The compiler has to generate, check, and optimize multiple versions of the same function.                                                              |
| **Binary Size (Con)**     | Larger executable files (Code Bloat). If you use a generic function with 20 different types, the compiler writes 20 different functions into your final binary.            |

Because of monomorphization, using generics in Rust costs you absolutely nothing in terms of runtime performance compared to writing duplicate code by hand. You only pay the cost during compile time and in the size of the final binary file.

### What gets monomorphized

Monomorphization applies to **all** generic constructs, not just functions:

- **structs** — `BrowserCommand<String>` and `BrowserCommand<i32>` become two separate structs in the binary
- **enums** — `Option<String>` and `Option<i32>` become two separate enums
- **free functions** — `serialize_payload::<String>` and `serialize_payload::<i32>` become two separate functions
- **impl blocks** — the methods inside `impl<T> BrowserCommand<T>` are duplicated for each concrete `T` used
- **closures** — each closure with generic bounds gets its own monomorphized version

The rule: **anywhere `T` appears and gets substituted with a concrete type, the compiler generates a dedicated copy**.

## Generics in structs

Declare `<T>` after the struct name; `T` can then be used as a field type:

```rust
struct Wrapper<T> {
    value: T,
}

let w1 = Wrapper { value: 42 };        // T = i32
let w2 = Wrapper { value: "hello" };   // T = &str
```

Multiple type parameters are separated by commas:

```rust
struct Pair<T, U> {
    first: T,
    second: U,
}
```

## Generics in enums

Same syntax as structs — declare `<T>` after the enum name and use it in
variants:

```rust
enum MaybeTwo<A, B> {
    First(A),
    Second(B),
    Neither,
}
```

The standard library's `Option` and `Result` are the most common examples (see
the [Generics in the standard library](#generics-in-the-standard-library)
section above).

## Generics in traits

A trait can be generic over a type parameter, meaning each implementation can
choose what that type is:

```rust
trait Converter<T> {
    fn convert(&self) -> T;
}

struct Celsius(f64);

impl Converter<f64> for Celsius {
    fn convert(&self) -> f64 {
        self.0
    }
}

impl Converter<String> for Celsius {
    fn convert(&self) -> String {
        format!("{}°C", self.0)
    }
}
```

The same struct can implement the same trait multiple times, once per concrete `T`.

## Generics in trait implementations

When implementing a generic trait for a generic struct, declare all type
parameters after `impl`:

```rust
trait Summary<T> {
    fn summarize(&self) -> T;
}

struct Article<T> {
    content: T,
}

impl<T: Clone> Summary<T> for Article<T> {
    fn summarize(&self) -> T {
        self.content.clone()
    }
}
```

## Generics in type aliases

Type aliases can fix one or more parameters of a generic type to create a
shorter, more specific name:

```rust
type StringResult<E> = Result<String, E>;
type IoResult<T>     = Result<T, std::io::Error>;

fn read_file(path: &str) -> IoResult<String> { ... }
```

This is common in the standard library — `std::io::Result<T>` is just
`Result<T, std::io::Error>`.

## Generics with `where` clauses

When bounds become long or complex, move them to a `where` clause after the
signature for readability:

```rust
// Inline bounds — hard to read
fn print_pair<T: std::fmt::Display + Clone, U: std::fmt::Debug>(a: T, b: U) {
    println!("{} {:?}", a.clone(), b);
}

// Equivalent with where — much cleaner
fn print_pair<T, U>(a: T, b: U)
where
    T: std::fmt::Display + Clone,
    U: std::fmt::Debug,
{
    println!("{} {:?}", a.clone(), b);
}
```

`where` clauses also allow bounds on types that are not themselves a parameter
— for example, bounding an associated type:

```rust
fn foo<I>(iter: I)
where
    I: Iterator,
    I::Item: std::fmt::Display,
{
    for item in iter {
        println!("{}", item);
    }
}
```

## Trait bounds

A **trait bound** constrains a generic type parameter to only accept types that
implement a specific trait. Without a bound, Rust knows nothing about `T` and
won't let you call any methods on it.

```rust
// No bound — T can be anything, but you cannot use Display-specific features
fn wrap<T>(value: T) -> Vec<T> {
    vec![value]                        // fine — no methods called on T
}

// With a bound — T must implement Display, so {} works
fn print<T: std::fmt::Display>(value: T) {
    println!("{}", value);             // ok — Display is guaranteed
}

// Without a bound, trying to use Display causes a compile error:
fn print_broken<T>(value: T) {
    println!("{}", value);             // error: `T` doesn't implement `Display`
}
```

### Multiple bounds

Use `+` to require more than one trait:

```rust
fn print_and_clone<T: std::fmt::Display + Clone>(value: T) -> T {
    println!("{}", value);
    value.clone()
}
```

### Bounds on generic structs and impl blocks

Bounds can be placed on `impl` blocks to restrict which types get certain
methods:

```rust
use std::fmt::Display;

struct Wrapper<T> {
    value: T,
}

// This method is only available when T implements Display
impl<T: Display> Wrapper<T> {
    fn show(&self) {
        println!("{}", self.value);
    }
}
```

### Trait bounds vs. generic traits

These are two different things that are easy to confuse:

| Concept | What it means | Example |
|---|---|---|
| **Trait bound** | A constraint on a type parameter | `fn foo<T: Clone>(x: T)` |
| **Generic trait** | The trait itself has a type parameter | `trait Converter<T> { ... }` |

They can appear together — a bound can reference a generic trait:

```rust
// Generic trait
trait Converter<T> {
    fn convert(&self) -> T;
}

struct Celsius(f64);

impl Converter<f64> for Celsius {
    fn convert(&self) -> f64 { self.0 }
}

impl Converter<String> for Celsius {
    fn convert(&self) -> String { format!("{}°C", self.0) }
}

// C: Converter<T> is a trait bound that uses the generic trait Converter<T>
// T: Clone         is a plain trait bound
fn convert_all<C, T>(items: &[C]) -> Vec<T>
where
    C: Converter<T>,   // bound using a generic trait
    T: Clone,          // plain bound
{
    items.iter().map(|c| c.convert()).collect()
}

fn main() {
    let readings = vec![Celsius(0.0), Celsius(100.0), Celsius(37.0)];

    let temps: Vec<f64>   = convert_all(&readings); // C = Celsius, T = f64
    let labels: Vec<String> = convert_all(&readings); // C = Celsius, T = String

    println!("{:?}", temps);   // [0.0, 100.0, 37.0]
    println!("{:?}", labels);  // ["0°C", "100°C", "37°C"]
}
```

video: lets get rusty/039.Generics
