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
