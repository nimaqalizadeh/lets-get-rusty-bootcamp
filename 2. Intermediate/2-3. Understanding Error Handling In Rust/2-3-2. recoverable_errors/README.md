# Recoverable Errors

## Ways to Match / Handle a `Result<T, E>`

### 1. `match` expression — full control

```rust
let file = match File::open("example.txt") {
    Ok(file) => file,
    Err(error) => panic!("Failed to open file: {:?}", error),
};
```

Both arms must return the same type — but `panic!` is an exception. It has the
type `!` (the **never type**), which means "this code never returns at all". The
compiler treats `!` as compatible with any type, so the `Err` arm satisfies the
`File` type requirement even though it never actually produces a `File`.

**What if you don't want to panic?** Use one of these instead:

**1. Return a default value of the same type**
```rust
// parse_input() returns Result<String, _>
let text = match parse_input() {
    Ok(val) => val,                     // val is a String
    Err(_)  => String::from("default"), // also a String — both arms match
};
```

**2. Propagate the error up to the caller with `?`**
```rust
fn run() -> Result<File, io::Error> {
    let file = File::open("example.txt")?; // caller decides what to do
    Ok(file)
}
```

**3. Use `unwrap_or` / `unwrap_or_else` for a fallback**
```rust
let file = File::open("example.txt").unwrap_or_else(|e| {
    println!("Using default because: {e}");
    File::open("default.txt").unwrap() // open fallback file — if this also fails, panic
                                       // (acceptable: if even the fallback is missing,
                                       //  something is seriously wrong)
});
```

> `panic!` is for situations that **should never happen** (programmer bugs).
> For expected failures like missing files, bad input, or network errors —
> use `Result` and handle it gracefully.

---

### 2. `if let` — only care about one variant

```rust
// Only handle Ok
if let Ok(file) = File::open("example.txt") {
    println!("Opened: {:?}", file);
}

// Only handle Err
if let Err(e) = File::open("example.txt") {
    println!("Error: {:?}", e);
}
```

Cleaner than `match` when you only care about one arm.

---

### 3. `while let` — loop while Ok (or while Err)

```rust
while let Ok(line) = reader.read_line(&mut buf) {
    // process line...
}
```

Stops the loop as soon as the pattern no longer matches. When `read_line` returns `Err`, the pattern doesn't match and the loop stops silently — **the error is dropped and ignored**. You never know if the loop stopped because the file ended normally or because an actual IO error occurred.

The safer approach is a regular `loop` with a full `match`:

```rust
loop {
    match reader.read_line(&mut buf) {
        Ok(0)  => break,             // end of file
        Ok(_)  => { /* process */ }  // got a line
        Err(e) => {                  // actual error
            println!("Error: {e}");
            break;
        }
    }
}
```

> Use `while let` only when you genuinely don't care about errors — like iterating over an `Option` or a known-safe iterator.

---

### 4. `unwrap()` — panic on `Err`, return inner value on `Ok`

```rust
let file = File::open("example.txt").unwrap();
```

Use only in tests or throwaway code. Panics with a generic message on `Err`.

---

### 5. `expect(msg)` — like `unwrap` with a custom panic message

```rust
let file = File::open("example.txt").expect("Failed to open file!");
```

Preferred over `unwrap` when you want a meaningful panic message.

---

### 6. `unwrap_or(default)` — fallback value on `Err`

Both `unwrap_or` and `unwrap_or_else` do the same thing:
- If `Ok(v)` → return `v`
- If `Err` → return a fallback value instead of panicking

They are a safe alternative to `unwrap()` — instead of crashing on error, you get a default value.

`unwrap_or` takes a **fixed fallback value** directly:

```rust
let value: Result<i32, &str> = Err("oops");
let n = value.unwrap_or(0);
// value is Err → n gets the fallback: 0

let value: Result<i32, &str> = Ok(42);
let n = value.unwrap_or(0);
// value is Ok → n gets the inner value: 42
```

---

### 7. `unwrap_or_else(|e| ...)` — compute fallback lazily via closure

`unwrap_or_else` takes a **closure** instead of a fixed value. The closure only runs **if there is an error** — making it lazy.

```rust
let value: Result<i32, &str> = Err("oops");
let n = value.unwrap_or_else(|e| {
    println!("Got error: {e}"); // only runs on Err
    -1                          // this becomes the fallback value
});
// n == -1
```

> **Why lazy?** When you write `unwrap_or(heavy_computation())`, Rust evaluates
> **all arguments before calling the function**. So `heavy_computation()` runs first
> and its result is computed — then `unwrap_or` is called with that result.
> This happens regardless of whether the value is `Ok` or `Err`. It's the same as:
>
> ```rust
> let fallback = heavy_computation(); // runs no matter what
> value.unwrap_or(fallback);          // now decides Ok or Err
> ```
>
> `unwrap_or_else` fixes this — the closure is not pre-evaluated as an argument.
> It only executes **inside** `unwrap_or_else` when there actually is an error.

- Use `unwrap_or` for cheap fallbacks like `0`, `""`, `false`
- Use `unwrap_or_else` for expensive fallbacks like DB calls, file reads, or logging

---

### 8. `map(|v| ...)` — transform `Ok` value, pass `Err` through

**"Transform"** = change the value inside `Ok` to something else.
**"Pass through"** = don't touch it, return it as-is.

`get_name()` returns `Result<String, io::Error>`. Sometimes you want to transform the success value — for example convert it to uppercase or parse it — **without touching the `Err`**. `map` lets you do that.

> Think of a box that is either green (Ok) or red (Err).
> `map` only opens the **green box**, swaps what's inside, and reseals it.
> The red box is never touched.

```
Result<String, io::Error>
         ↓  map(|s| s.to_uppercase())
Result<String, io::Error>  ← Ok value changed, Err untouched
```

```rust
let result = get_name().map(|s| s.to_uppercase());

// If Ok("alice")  → returned as Ok("ALICE")
// If Err(e)       → returned unchanged as Err(e)
```

---

### 9. `map_err(|e| ...)` — transform `Err` value, pass `Ok` through

The vice versa of `map` — only opens the **red box (Err)**, transforms what's inside, and leaves the **green box (Ok) untouched**.

`File::open` returns `Result<File, io::Error>`. Sometimes you want the error to be a `String` instead — for example to display it or match your function's return type. `map_err` lets you convert the error type **without touching the `Ok` value**.

```
Result<File, io::Error>
         ↓  map_err(|e| format!("IO error: {e}"))
Result<File, String>     ← error type changed, File untouched
```

```rust
let result: Result<File, String> = File::open("example.txt")
    .map_err(|e| format!("IO error: {e}"));

// If Ok(file)  → returned unchanged as Ok(file)
// If Err(e)    → e is converted to a String, returned as Err("IO error: ...")
```

---

### 10. `and_then(|v| ...)` — chain fallible operations (flatMap)

The problem it solves: when you use `map` and the closure itself returns a `Result`,
you end up with a nested `Result<Result<T, E>, E>` — which is awkward to work with.

**With `map` — gets nested:**

```rust
let result = File::open("example.txt")
    .map(|file| read_to_string(file)); // read_to_string also returns Result

// result type: Result<Result<String, io::Error>, io::Error>  ← nested, ugly
```

**With `and_then` — stays flat:**

```rust
let result = File::open("example.txt")
    .and_then(|file| read_to_string(file)); // flattens the nested Result

// result type: Result<String, io::Error>  ← clean
```

Think of it as **"if the first step succeeded, try the next step"**:

```
File::open succeeds  →  run read_to_string  →  return its Result
File::open fails     →  skip read_to_string →  return the Err immediately
```

**Rule of thumb:**
- Use `map` when your closure returns a **plain value** (`String`, `i32`, ...)
- Use `and_then` when your closure returns a **`Result`** (another fallible operation)

---

### 11. `?` operator — pass the error to the caller

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

---

## Python vs Rust Error Handling

### Python — exceptions bubble up automatically

```python
def open_and_read():
    file = open("example.txt")  # raises FileNotFoundError automatically
    return file.read()

def main():
    try:
        content = open_and_read()  # exception bubbles up here
    except FileNotFoundError as e:
        print(f"Error: {e}")       # main handles it
```

Python exceptions travel up **silently and automatically** — middle functions don't need to do anything.

**Two failure modes:**

1. **No try/except** — exception bubbles all the way up and crashes the program at runtime
2. **With try/except** — you might forget to handle a specific case, program still crashes at runtime

Both failures are discovered **at runtime** — possibly in production, by your users.

---

### Rust — errors are explicit values

```rust
fn open_and_read() -> Result<String, io::Error> {
    let file    = File::open("example.txt")?; // must explicitly pass it up with ?
    let content = read_to_string(file)?;
    Ok(content)
}

fn main() {
    match open_and_read() {
        Ok(content) => println!("{content}"),
        Err(e)      => println!("Error: {e}"),
    }
}
```

Errors travel up **only if you explicitly write `?`**. The compiler forces you to deal with every `Result`.

**Rust eliminates both Python failure modes:**

- You **cannot ignore** a `Result` — compiler warns you
- You **cannot forget** a case in `match` — compiler enforces exhaustive matching:

```rust
match result {
    Ok(val) => val,
    // forgot Err arm → compile ERROR, won't build
}
```

```
error[E0004]: non-exhaustive patterns: `Err(_)` not covered
```

---

### Side by side

| | Python | Rust |
|---|---|---|
| How errors travel up | Automatically (invisible) | Explicitly with `?` |
| Can you ignore an error? | Yes, accidentally | No — compiler forces you |
| Forget to handle an error | Crashes at **runtime** | Won't **compile** |
| Miss a case in handler | Crashes at **runtime** | Won't **compile** |
| When you find out | In production | While writing code |

> Rust moves the problem from **"your users discover it"** to **"you discover it before shipping"**.

---

## Quick Comparison

| Method          | Panics on Err? | Returns value? | Good for                        |
|-----------------|---------------|----------------|---------------------------------|
| `match`         | No            | Yes            | Full control, complex logic     |
| `if let`        | No            | No             | One-arm handling                |
| `while let`     | No            | No             | Loops over results              |
| `unwrap()`      | Yes           | Yes            | Tests / quick prototyping       |
| `expect()`      | Yes           | Yes            | Tests with clear panic message  |
| `unwrap_or()`   | No            | Yes            | Simple fallback value           |
| `unwrap_or_else`| No            | Yes            | Computed/logged fallback        |
| `map()`         | No            | Yes (Result)   | Transform success value         |
| `map_err()`     | No            | Yes (Result)   | Transform error value           |
| `and_then()`    | No            | Yes (Result)   | Chain fallible operations       |
| `?`             | No            | Yes            | Propagate errors up the call stack |
