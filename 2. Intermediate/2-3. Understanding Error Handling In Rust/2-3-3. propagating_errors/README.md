# Propagating
## `?` operator — pass the error to the caller

`?` means: **"I'm not handling this error here — give it to whoever called me."**

It is shorthand for this `match`:

```rust
match result {
    Ok(val) => val,           // unwrap and continue
    Err(e)  => return Err(e), // stop this function, give error to caller
}
```

> `return Err(e)` does **not** mean this function is handling the error.
> It means this function is **refusing to handle it** — it exits immediately
> and hands the error upward to whoever called it.
>
> ```
> File::open fails
>   → returns Err to open_and_read
>       → open_and_read does `return Err(e)`  ← refusing, passing it up
>           → Err travels up to main()
>               → main() finally handles it
> ```

**Example — without `?`:**

```rust
fn open_and_read() -> Result<String, io::Error> {
    let file = match File::open("example.txt") {
        Ok(f)  => f,
        Err(e) => return Err(e), // stop, give error to caller
    };
    let content = match read_to_string(file) {
        Ok(s)  => s,
        Err(e) => return Err(e), // stop, give error to caller
    };
    Ok(content)
}
```

**Same thing — with `?`:**

```rust
fn open_and_read() -> Result<String, io::Error> {
    let file    = File::open("example.txt")?; // Err → return to caller immediately
    let content = read_to_string(file)?;      // Err → return to caller immediately
    Ok(content)
}
```

**Who should handle the error?**

`?` moves the error *up the call stack* until it reaches a function that can decide what to do with it:

```
main()               ← must handle it here (no one above)
  └─ open_and_read() ← uses ? to pass it up
       └─ File::open ← uses ? to pass it up
```

- Middle functions use `?` — they don't have enough context to react.
- The outermost caller (usually `main`) uses `match` or `unwrap_or_else` — it knows whether to log, retry, show a message, or exit.

**From the Rust Reference:**

The official name is the **"try propagation expression"**. It is based on the `Try` trait and works on more than just `Result`:

| Type | On success | On failure |
|---|---|---|
| `Result<T, E>` | evaluates to `T` | returns `Err(From::from(e))` |
| `Option<T>` | evaluates to `T` | returns `None` |
| `ControlFlow<B, C>` | evaluates to `C` | returns `Break(b)` |

> Note: when propagating a `Result`, `?` calls `From::from(e)` on the error —
> meaning it **automatically converts** the error type if a conversion exists.
> This is why you can use `?` across functions with different (but compatible) error types.

**Example — `?` converting error types automatically:**

```rust
use std::fs::File;
use std::io;
use std::num::ParseIntError;

// Our custom error type that can represent both kinds of errors
enum AppError {
    Io(io::Error),
    Parse(ParseIntError),
}

// Tell Rust: io::Error can be converted into AppError
impl From<io::Error> for AppError {
    fn from(e: io::Error) -> AppError {
        AppError::Io(e)
    }
}

// Tell Rust: ParseIntError can be converted into AppError
impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> AppError {
        AppError::Parse(e)
    }
}

fn run() -> Result<i32, AppError> {
    let _file = File::open("example.txt")?; // io::Error → automatically converted to AppError::Io
    let n: i32 = "42".parse()?;             // ParseIntError → automatically converted to AppError::Parse
    Ok(n)
}
```

Without `From::from`, these two `?` calls would fail to compile because `io::Error`
and `ParseIntError` are different types from `AppError`. The automatic conversion
is what makes `?` so powerful when combining multiple fallible operations.

Reference: [operator-expr.html#the-try-propagation-expression](https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#the-try-propagation-expression)

> Only works inside functions that return `Result`, `Option`, or another `Try`-compatible type.