# Multiple error handling

There are couple of ways to handle the situations where functions return differenct types of errors:

1. Instead of specifying a concrete error type we can specify a trait object. so instead of this:

```rust
fn parse_file(filename: &str) -> Result<i32, io::Error>> {
    let s = fs::read_to_string(filename)?;
    let i = s.parse()?;
    Ok(i)
}
```
We can say that Result accepts any type that implement the `Error` trait which is defined in the `error` module. The benefit of this aproach is that it makes the error handling simple; the downside is callers of our function won't know what concrete error types are being returned at compile time and therefore can't handle individual errors differently.

```rust
fn parse_file(filename: &str) -> Result<i32, Box<dyn error::Error>> {
    let s = fs::read_to_string(filename)?;
    let i = s.parse()?;
    Ok(i)
}
```

2. Create a custom error `enum`

Error handling in this way requires more code but the advantage is that caller of this function can distinguish between `File` error and `Parse` error. 

```rust
enum ParseFileError {
    File,               // unit variant — holds no data
    Parse(ParseIntError) // tuple variant — holds one ParseIntError value
}

fn parse_file(filename: &str) -> Result<i32, ParseFileError> {
    let s = fs::read_to_string(filename)
                    .map_err(|e| ParseFileError::File)?;
    let i = s.parse()
                    .map_err(|e| ParseFileError::Parse(e))?;
    Ok(i)
}
```

### How `map_err` works here

- **`map_err(|e| ParseFileError::File)?`** — the closure receives the original `io::Error` as `e`, but ignores it and returns the unit variant `ParseFileError::File`. The original error is **discarded** because the `File` variant holds no data. The `?` then propagates the `Err(ParseFileError::File)` up to the caller if the file read failed, or unwraps the `Ok` value and continues.

- **`map_err(|e| ParseFileError::Parse(e))?`** — the original `ParseIntError` is **wrapped and kept** inside the tuple variant. The `?` propagates the `Err(ParseFileError::Parse(e))` up to the caller if parsing failed, or unwraps the `Ok` value and continues. The caller can later extract it:

```rust
fn main() {
    // parse_file returns Result<i32, ParseFileError>
    // the ? inside parse_file propagates the error up to here
    match parse_file("numbers.txt") {
        Ok(n) => println!("parsed number: {}", n),

        // File variant holds no data, so there's nothing to extract —
        // we only know the file read failed, not why
        Err(ParseFileError::File) => {
            println!("failed to read the file");
        }

        // Parse variant wraps the original ParseIntError inside it —
        // e is the actual ParseIntError returned by s.parse(), so we can
        // display the specific reason why parsing failed
        Err(ParseFileError::Parse(e)) => {
            println!("failed to parse number: {}", e);
        }
    }
}
```

This is the standard Rust pattern for wrapping errors in a custom enum — unit variants for errors where the detail doesn't matter, tuple variants when you want to preserve the original error for inspection or display.

### What does "propagating to the caller" mean?

It means the function **stops immediately and returns the error** to whoever called it:

```rust
fn parse_file(filename: &str) -> Result<i32, ParseFileError> {
    let s = fs::read_to_string(filename)
                    .map_err(|e| ParseFileError::File)?;  // if this fails...
    // ...the lines below never run
    let i = s.parse()
                    .map_err(|e| ParseFileError::Parse(e))?;
    Ok(i)
}

fn main() {
    match parse_file("numbers.txt") {
        Ok(n) => println!("{}", n),
        Err(e) => println!("something went wrong"), // ...the error lands here
    }
}
```

If `fs::read_to_string` fails, `?` returns `Err(ParseFileError::File)` from `parse_file` immediately — `s.parse()` and `Ok(i)` are never reached. The error travels back to `main`, which called `parse_file`.

### What if there's no `?`?

Without `?`, `map_err` still converts the error type but the `Result` stays as a `Result` — you must handle it manually:

```rust
let s = fs::read_to_string(filename)
                .map_err(|e| ParseFileError::File); // s is Result<String, ParseFileError>, not String
```

If you try to use `s` as a string, the compiler will refuse because you're treating a `Result` as if it's already unwrapped. You'd have to write:

```rust
match s {
    Ok(content) => { /* use content */ }
    Err(e) => return Err(e), // manually do what ? did automatically
}
```

So `?` is shorthand for: if `Ok` → unwrap and continue, if `Err` → return the error immediately.

---

## The `#[must_use]` Rule in Rust

Rust enforces a general rule: **if a value signals success/failure or contains something meaningful, you must acknowledge it.** This is implemented via the `#[must_use]` attribute — if you ignore a `#[must_use]` value entirely, the compiler emits a warning.

### Types and variants that are `#[must_use]` by default

#### 1. `Result<T, E>`
```rust
parse_file("numbers.txt"); // warning: unused `Result` that must be used
```
The core rule:
- `Result<T, E>` has two possibilities: `Ok(value)` or `Err(error)`
- You **cannot access the inner value** without handling both cases
- Ignoring a `Result` entirely means silently swallowing potential errors

**Ways to handle it:**
| Mechanism | Behavior |
|-----------|----------|
| `match` | explicit handling of both `Ok` and `Err` |
| `if let Ok(v) = result` | handle only the success case |
| `?` | propagate `Err` to the caller, unwrap `Ok` |
| `.unwrap()` | unwrap `Ok`, **panic** on `Err` |
| `.expect("msg")` | same as unwrap but with a custom panic message |
| `let _ = result` | explicitly discard — tells compiler "I know and don't care" |

#### 2. `Option<T>`
```rust
some_vec.iter().find(|x| *x == &5); // warning: unused `Option` that must be used
```
The core rule:
- `Option<T>` has two possibilities: `Some(value)` or `None`
- You **cannot access the inner value** without handling both cases
- Ignoring an `Option` means you may be silently missing a `None` case

**Ways to handle it:**
| Mechanism | Behavior |
|-----------|----------|
| `match` | explicit handling of both `Some` and `None` |
| `if let Some(v) = option` | handle only the `Some` case |
| `?` | return `None` early to the caller (inside functions returning `Option`) |
| `.unwrap()` | unwrap `Some`, **panic** on `None` |
| `.unwrap_or(default)` | unwrap `Some`, return a default on `None` |
| `let _ = option` | explicitly discard |

#### 3. Futures / `async`
```rust
fetch_data(); // warning: unused future — the async operation never runs at all!
```
Futures in Rust are **lazy** — they do nothing until `.await`ed. Ignoring one means the operation never executes.

#### 4. Iterators
```rust
vec.iter().map(|x| x * 2); // warning: unused iterator
```
Iterators are also **lazy** — they produce no values until consumed by `.collect()`, `.for_each()`, etc.

#### 5. Custom types with `#[must_use]`
Any type or function can opt into this rule:
```rust
#[must_use]
fn calculate() -> i32 { 42 }

calculate(); // warning: unused return value of `calculate`
```

To use it, the return value must **go somewhere**:

```rust
let result = calculate();
println!("{}", result);

println!("{}", calculate());

do_something(calculate());

let _ = calculate();
```

### The general rule
> If you have a `Result` or `Option` and want to use the inner value, you must acknowledge **all** possibilities before the compiler lets you access it.

This is one of Rust's biggest strengths — it makes it **impossible to accidentally ignore an error or a missing value** at compile time, unlike languages where you can forget to check for `null` or an error code and cause a silent runtime bug.