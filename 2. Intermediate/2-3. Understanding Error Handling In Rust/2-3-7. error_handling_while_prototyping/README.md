# Prototyping

I like to add `anyhow` pretty early during the prototyping phase, to get more fine-grained control over my error handling. This way, I can use `bail!` and `with_context` to quickly add more context to my errors without losing momentum. Later on, I can revisit each error case and see if I can handle it more gracefully.

The great thing about `anyhow` is that it's a solid choice for error handling in production code as well, so you don't have to rewrite your error handling logic later on.

## `anyhow` imports

```rust
use anyhow::{bail, Context, Result};
```

## When to use each one

| Situation | Use |
|---|---|
| Function can fail | `-> Result<T>` as the return type |
| An operation failed and you want to add a message | `.context("...")?` |
| You want to fail on your own terms | `bail!("...")` |

- **`Result`** goes on your **function signature**. Once you write `-> Result<T>`, the `?` operator works on any error inside that function. Use it on every function that can fail.
- **`Context` / `with_context`** chains onto an **existing `Result`** (or `Option`) right before `?`. Use it when an operation *can* fail and you want to explain *why* that failure matters. For example, `std::fs::read_to_string(path)?` gives a generic IO error, but `.context("Failed to load user config")?` tells you what you were actually trying to do.
- **`bail!`** is for when **there is no existing error** to attach context to. You're creating a brand new error and returning immediately. Typical spots: validation checks (`if age < 0 { bail!(...) }`), `let-else` blocks, or any place where you decide "this is wrong, I'm done."

## Details and examples

### `Result`

A type alias for `Result<T, anyhow::Error>`. It replaces `std::result::Result` so you can use `?` on any error type without writing manual `From` conversions — `anyhow` wraps them all for you.

```rust
// Without anyhow — you'd need to define your own error type or use Box<dyn Error>
fn load_config() -> std::result::Result<String, Box<dyn std::error::Error>> { ... }

// With anyhow — just use Result and ? works on any error type
fn load_config() -> Result<String> {
    let path = std::env::var("CONFIG_PATH")?;  // env::VarError -> anyhow::Error
    let content = std::fs::read_to_string(path)?;  // io::Error -> anyhow::Error
    Ok(content)
}
```

When using `Result` with just `?` (no `.context()`), you get the **original error message** from the underlying error type, nothing more. For example, if `CONFIG_PATH` is not set:

```
Error: environment variable not found
```

If the file doesn't exist:

```
Error: No such file or directory (os error 2)
```

You don't know *what* the code was trying to do, only *what went wrong* at a low level. That's exactly where `.context()` helps — it wraps the error with your own message so it's actually useful for debugging.

### `Context`

A trait that adds `.context()` and `.with_context()` to `Result` and `Option`. It lets you attach a human-readable message explaining *what you were trying to do* when the error occurred. This makes error messages much more useful for debugging.

```rust
let home = std::env::var("HOME")
    .context("Could not read HOME environment variable")?;

// If HOME is not set, instead of just:
//   "environment variable not found"
// you get:
//   "Could not read HOME environment variable"
//   Caused by: environment variable not found

// Use .with_context() when you need to include dynamic values:
let config = std::fs::read_to_string(&path)
    .with_context(|| format!("Failed to read config file at {}", path))?;
```

**`.context()` vs `.with_context()`:** `.context("static message")` takes a plain string — use it for simple messages. `.with_context(|| format!(...))` takes a closure — use it when you need dynamic values. The closure is only evaluated if there's an error, so you avoid the cost of formatting the string when things succeed.

### `bail!`

A macro that immediately returns an `Err(anyhow::Error)`. It's shorthand for `return Err(anyhow!(...))`, so these two are equivalent:

```rust
// The verbose way
fn process_age(age: i32) -> Result<()> {
    if age < 0 {
        return Err(anyhow::anyhow!("age cannot be negative, got {}", age));
    }
    Ok(())
}

// The shorthand — bail! does the same thing
fn process_age(age: i32) -> Result<()> {
    if age < 0 {
        bail!("age cannot be negative, got {}", age);
    }
    Ok(())
}
```

Useful in guard clauses and `let-else` blocks where you want to exit early with a descriptive error.

```rust
// Use bail! in a let-else block
let Ok(home) = std::env::var("HOME") else {
    bail!("Could not read HOME environment variable");
};

// Use bail! for validation
fn process_age(age: i32) -> Result<()> {
    if age < 0 {
        bail!("age cannot be negative, got {}", age);
    }
    // ...
    Ok(())
}
```

### `let else` vs `if let`

They handle opposite cases:
- **`if let`** — runs code when the pattern **matches** (the happy path is inside the block)
- **`let else`** — runs code when the pattern **doesn't match** (the happy path continues after the block)

**`if let`** — do something with the matched value inside the block:

```rust
// if let
if let Ok(home) = std::env::var("HOME") {
    println!("Home is: {}", home);
    // `home` only exists inside this block
}
// `home` is NOT available here

// equivalent match expression
match std::env::var("HOME") {
    Ok(home) => {
        println!("Home is: {}", home);
    }
    Err(_) => {} // do nothing
}
```

**`let else`** — handle the failure, then continue with the matched value:

```rust
// let else
let Ok(home) = std::env::var("HOME") else {
    bail!("HOME not set"); // must diverge: return, bail!, break, continue, or panic!
};
// `home` IS available here
println!("Home is: {}", home);

// equivalent match expression
let home = match std::env::var("HOME") {
    Ok(home) => home,
    Err(_) => bail!("HOME not set"),
};
println!("Home is: {}", home);
```

Use `if let` when you want to optionally do something. Use `let else` when you need the value for the rest of the function and want to bail early if it's missing.

## Use `println!` and `dbg!` for debugging

Printing values is pretty handy while prototyping. It's one less context switch compared to starting a debugger.

Most people use `println!` for that, but `dbg!` has a few advantages:

- It prints the **file name and line number** where the macro is called. This helps you quickly find the source of the output.
- It outputs the **expression as well as its value**.
- It's **less syntax-heavy** than `println!`; e.g. `dbg!(x)` vs `println!("{x:?}")`.

Where `dbg!` really shines is in recursive functions or when you want to see the intermediate values during an iteration:

```rust
fn factorial(n: u32) -> u32 {
    // `dbg!` returns the argument,
    // so you can use it in the middle of an expression
    if dbg!(n <= 1) {
        dbg!(1)
    } else {
        dbg!(n * factorial(n - 1))
    }
}

dbg!(factorial(4));
```

The output is nice and tidy:

```
[src/main.rs:2:8] n <= 1 = false
[src/main.rs:2:8] n <= 1 = false
[src/main.rs:2:8] n <= 1 = false
[src/main.rs:2:8] n <= 1 = true
[src/main.rs:3:9] 1 = 1
[src/main.rs:7:9] n * factorial(n - 1) = 2
[src/main.rs:7:9] n * factorial(n - 1) = 6
[src/main.rs:7:9] n * factorial(n - 1) = 24
[src/main.rs:9:1] factorial(4) = 24
```

Since `dbg!` returns the value it wraps, you can insert it into method chains without changing behavior:

```rust
let names = vec!["Alice", "Bob", "Charlie", "Dave"];

let result: Vec<&str> = names
    .iter()
    .filter(|name| dbg!(name.len() > 3))
    .copied()
    .collect();

dbg!(&result);
```

Output:

```
[src/main.rs:4:25] name.len() > 3 = true
[src/main.rs:4:25] name.len() > 3 = false
[src/main.rs:4:25] name.len() > 3 = true
[src/main.rs:4:25] name.len() > 3 = true
[src/main.rs:8:5] &result = ["Alice", "Charlie", "Dave"]
```

You can see exactly which names passed the filter and why — without restructuring your code.

> **Note:** Remove `dbg!` calls from your final code — they also execute in release mode.

## The `todo!` macro

One of the cornerstones of prototyping is that you don't have to have all the answers right away. The `todo!` macro expresses exactly that idea. You can scaffold out functions or a module and fill in the blanks later.

```rust
// We don't know yet how to process the data
// but we're pretty certain that we need a function
// that takes a Vec<i32> and returns an i32
fn process_data(data: Vec<i32>) -> i32 {
    todo!()
}

// There exists a function that loads the data and returns a Vec<i32>
// How exactly it does that is not important right now
fn load_data() -> Vec<i32> {
    todo!()
}

fn main() {
    // Given that we have a function to load the data
    let data = load_data();
    // ... and a function to process it
    let result = process_data(data);
    // ... we can print the result
    println!("Result: {}", result);
}
```

We did not do much here, but we have a clear idea of what the program should do. Now we can go and iterate on the design. For example, should `process_data` take a reference to the data? Should we create a struct to hold the data and the processing logic? How about using an iterator instead of a vector? Should we introduce a trait to support algorithms for processing the data?

These are all helpful questions that we can answer without having to worry about the details of the implementation. And yet our code is typesafe and compiles, and it is ready for refactoring.

### Where else you can use `todo!()`

`todo!()` returns `!` (the never type), which coerces to any type. This means you can use it in **any expression position**, not just function bodies:

```rust
// Variable bindings — useful when you know the type but not the value yet
let config: Config = todo!();

// Match arms — handle the happy path first, deal with edge cases later
match status {
    Status::Ok => process(),
    Status::Error => todo!("handle error case"),
}

// Closure bodies — define the signature now, implement later
let transform = |input: Vec<u8>| -> String { todo!() };

// Trait default method implementations
trait Processor {
    fn validate(&self) -> bool {
        todo!("add validation logic")
    }
}

// If/else branches
let value = if condition {
    compute_value()
} else {
    todo!("handle the else case")
};
```

### Where `todo!()` does NOT work

`todo!()` is an expression, not a type. It cannot be used in places that expect type-level constructs:

```rust
// Struct fields — these need types, not expressions
struct Foo {
    x: i32,
    y: ???, // can't use todo!() here
}

// Enum variants — same reason
enum Bar {
    A(i32),
    B(???), // can't use todo!() here
}

// Generic parameters — type-level, not value-level
fn process<T: ???>() {} // can't use todo!() here
```

For these cases, you can use placeholder types instead (e.g. `()` or a temporary type alias) and refine them later.

## `unreachable!` for unreachable branches

On a related note, you can use the `unreachable!` macro to mark branches of your code that should never be reached.

```rust
fn main() {
    let age: u8 = 170;

    match age {
        0..150 => println!("Normal human age"),
        150.. => unreachable!("Witchcraft!"),
    }
}
```

This is a great way to document your assumptions about the code. The result is the same as if you had used `todo!`, but it's more explicit about the fact that this branch should never be reached:

```
thread 'main' panicked at src/main.rs:6:18:
internal error: entered unreachable code: Witchcraft!
```

Note that we added a message to the `unreachable!` macro to make it clear what the assumption is.

Since `unreachable!` panics at runtime, you should replace it with proper error handling during refactoring:

```rust
fn check_age(age: u8) -> Result<()> {
    match age {
        0..150 => println!("Normal human age"),
        150.. => bail!("Age {} is out of expected range", age),
    }
    Ok(())
}
```

This way, the caller can decide how to handle the unexpected case instead of the program crashing.

## `assert!` vs `debug_assert!` — where each one belongs

Both macros check that a condition is `true` and panic if it's not. The difference is **when they run**:

| Macro | `cargo test` | `cargo run` | `cargo run --release` |
|---|---|---|---|
| `assert!` | runs | runs | runs |
| `debug_assert!` | runs | runs | **compiled away** |

### Use `assert!` in tests

Tests are where you **verify correctness** — every check must run, no exceptions. Since `cargo test --release` would silently skip `debug_assert!`, always use `assert!` (and `assert_eq!`, `assert_ne!`) in tests:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_sort() {
        let mut data = vec![3, 1, 2];
        data.sort();
        // Always use assert! in tests — these checks ARE the test
        assert_eq!(data, vec![1, 2, 3]);
    }
}
```

### Use `debug_assert!` in production code for expensive dev-time checks

`debug_assert!` is for invariant checks in your **production code** that are helpful during development but **too slow to run in release builds**. The check itself might take longer than the actual work (e.g., an O(n) validation inside an O(log n) function), so you want it active while developing to catch bugs, but compiled away in production where performance matters.

You place `debug_assert!` inside the function body in two common patterns:

**Preconditions — verify assumptions about inputs** (at the top of the function):

```rust
fn binary_search(sorted_list: &[i32], target: i32) -> Option<usize> {
    // "I assume the caller gave me a sorted list"
    // O(n) check — catches bugs during development,
    // compiled away in release so it doesn't slow down production
    debug_assert!(
        sorted_list.windows(2).all(|w| w[0] <= w[1]),
        "list must be sorted!"
    );

    // actual O(log n) search logic
    sorted_list.binary_search(&target).ok()
}
```

**Postconditions — verify the result of your logic** (at the end, before returning):

```rust
fn sort_data(data: &mut Vec<i32>) {
    // ... sorting logic ...

    // "I assume my sorting logic worked correctly"
    debug_assert!(
        data.windows(2).all(|w| w[0] <= w[1]),
        "data should be sorted after sort_data"
    );
}

fn transform_matrix(matrix: &mut Vec<Vec<f64>>) {
    // ... perform transformation ...

    // Expensive O(n²) postcondition check — only during development
    debug_assert!(
        matrix.iter().enumerate().all(|(i, row)| {
            row.iter().enumerate().all(|(j, &val)| (val - matrix[j][i]).abs() < 1e-10)
        }),
        "matrix must remain symmetric after transformation"
    );
}
```

In both cases, you're saying: "I'm confident this is true, but let me double-check during development." If the assertion fails during `cargo run`, you know you have a bug. Once you're confident the code is correct, these checks disappear in release builds automatically.

### Why not just write tests instead of `debug_assert!`?

You should write tests **and** use `debug_assert!` — they catch different kinds of bugs.

**Tests** verify your code works with **specific inputs you choose**:

```rust
#[test]
fn test_binary_search() {
    assert_eq!(binary_search(&[1, 3, 5, 7], 5), Some(2));
    assert_eq!(binary_search(&[1, 3, 5, 7], 4), None);
}
```

But tests can't cover every possible input your function will receive in production. What if some caller, somewhere in your codebase, accidentally passes an unsorted list? Your tests won't catch that — they only test the cases you thought of.

`debug_assert!` catches bugs **at the call site, with real data, during development**:

```rust
fn binary_search(sorted_list: &[i32], target: i32) -> Option<usize> {
    // Catches ANY caller passing unsorted data — during development
    debug_assert!(sorted_list.windows(2).all(|w| w[0] <= w[1]));
    sorted_list.binary_search(&target).ok()
}
```

| | Tests (`assert!`) | `debug_assert!` in function body |
|---|---|---|
| **Checks** | Specific inputs you wrote | Every input during development |
| **Catches** | "Does my logic work?" | "Is someone calling this wrong?" |
| **When** | `cargo test` | Every `cargo run` in debug mode |

Tests verify the function **internally works correctly**. `debug_assert!` guards against the function being **called incorrectly** by the rest of your codebase. Use both.

### Other use cases for `debug_assert!`

Preconditions and postconditions are the most common use cases, but `debug_assert!` works anywhere you have an assumption that would be expensive or awkward to enforce through the type system.

**Loop invariants** — verify something stays true on every iteration:

```rust
fn running_average(data: &[f64]) -> Vec<f64> {
    let mut sum = 0.0;
    let mut averages = Vec::new();

    for (i, &val) in data.iter().enumerate() {
        sum += val;
        let avg = sum / (i + 1) as f64;
        averages.push(avg);

        // Invariant: average should always be between min and max of seen values
        debug_assert!(
            avg >= data[..=i].iter().copied().fold(f64::INFINITY, f64::min),
            "average dropped below minimum"
        );
    }
    averages
}
```

**State transitions** — verify an object is in the expected state before performing an operation:

```rust
impl Connection {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        debug_assert!(self.is_connected(), "must be connected before sending");
        // ... send logic ...
    }
}
```

**Intermediate computations** — check values mid-algorithm:

```rust
fn normalize(weights: &mut [f64]) {
    let total: f64 = weights.iter().sum();
    for w in weights.iter_mut() {
        *w /= total;
    }

    // Mid-algorithm sanity check: all weights should now sum to ~1.0
    debug_assert!(
        (weights.iter().sum::<f64>() - 1.0).abs() < 1e-9,
        "weights should sum to 1.0 after normalization"
    );
}
```

**Concurrency assumptions** — verify thread-safety expectations:

```rust
fn update_cache(&self) {
    debug_assert!(
        std::thread::current().name() == Some("main"),
        "cache updates must happen on the main thread"
    );
    // ... update logic ...
}
```

**Rule of thumb:** start with `assert!`. Only switch to `debug_assert!` when profiling shows the assertion is a measurable bottleneck **and** the code is safe without the check.

### `debug_assert!` is NOT an alternative to `unreachable!`

They serve different purposes:

| Macro | Purpose | Release behavior |
|---|---|---|
| `debug_assert!(condition)` | Validates a condition you **expect to be true** | Compiled away |
| `unreachable!()` | Marks code that should **never be reached** | **Always panics** |

`unreachable!()` is a hard guarantee — if execution reaches that line, it's always a bug, in every build. `debug_assert!` is a soft development-time check that you're confident enough to skip in release.

```rust
fn classify(value: u8) -> &'static str {
    match value {
        0..=49 => "low",
        50..=100 => "high",
        // unreachable! — always panics, this is a hard invariant
        101.. => unreachable!("value is clamped to 0..=100 before this call"),
    }
}

fn process(sorted_data: &[i32]) {
    // debug_assert! — soft check, compiled away in release
    debug_assert!(
        sorted_data.windows(2).all(|w| w[0] <= w[1]),
        "data must be sorted"
    );
    // ... process sorted data ...
}
```

## `anyhow` vs `thiserror` — when to use which

`anyhow` (including `bail!`, `context`, and `Result`) is production-ready and widely used. You don't need to replace it with raw `Result` types before shipping. The real question is whether your crate is an **application** or a **library**:

- **`anyhow`** — best for **applications** (binaries, CLI tools, web servers). Errors are reported to the user as messages. Callers don't need to programmatically distinguish between error kinds.
- **`thiserror`** — best for **libraries**. It lets you define custom error types so that callers can match on specific variants:

```rust
// With thiserror — callers can match on specific error kinds
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("age {0} is out of expected range")]
    InvalidAge(u8),
    #[error("failed to load config")]
    ConfigError(#[from] std::io::Error),
}

fn check_age(age: u8) -> Result<(), AppError> {
    match age {
        0..150 => println!("Normal human age"),
        150.. => return Err(AppError::InvalidAge(age)),
    }
    Ok(())
}

// Now callers can handle specific cases
match check_age(170) {
    Err(AppError::InvalidAge(age)) => eprintln!("Bad age: {age}"),
    Err(AppError::ConfigError(e)) => eprintln!("Config problem: {e}"),
    Ok(()) => {}
}
```

A common pattern is to use **both**: `thiserror` to define your error types and `anyhow` in your application code that consumes them. Start with `anyhow` during prototyping, then introduce `thiserror` for parts of your codebase that need structured error handling.

https://corrode.dev/blog/prototyping/
