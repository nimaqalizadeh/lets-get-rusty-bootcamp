# Option and Result
## Why `Result<Option<String>, io::Error>`?

This return type reflects two independent failure modes:

**`Result` — operation can fail**
- Reading the file might fail (file not found, permission denied, etc.)
- This is an `io::Error` — something went wrong externally
- `Err(io::Error)` signals: "I couldn't even try"

**`Option` — operation can succeed but yield nothing**
- The file was read successfully, but it might be empty (no lines)
- Before `.next()`: we have a `Result<String, io::Error>` (the file contents)
- After `.next()`: we have an `Option<&str>` — `None` if the file is empty, `Some(line)` otherwise
- `None` signals: "I succeeded, but there's nothing here"

So the three possible outcomes are:

| Return value | Meaning |
|---|---|
| `Err(e)` | File couldn't be read |
| `Ok(None)` | File was read, but it's empty |
| `Ok(Some(line))` | File was read and has a first line |

**Why not just `Option<String>`?**

That's what `read_first_line2` does — it calls `.ok()` to convert the `Result` into an `Option`, which collapses the two failure cases: you can no longer distinguish "file not found" from "file is empty". Both become `None`. You lose the error information.

`Result<Option<String>, io::Error>` preserves that distinction, which matters if the caller needs to handle errors (e.g., log them, retry, surface to the user).

If you want handle all three possibilites:
**Handling the error (e.g., logging):**
```rust
match read_first_line("file.txt") {
    Err(e) => eprintln!("Error reading file: {}", e),
    Ok(None) => println!("File is empty"),
    Ok(Some(line)) => println!("First line: {}", line),
}
```

Or with `if let` if you only care about the error:
```rust
if let Err(e) = read_first_line("file.txt") {
    eprintln!("Error reading file: {}", e);
}
```

If you used `Option<String>` instead, you'd only get `Some(line)` or `None` — you'd never know *why* it failed, so you couldn't log a meaningful error message.

## Combinators: `.ok()` and `.and_then()`
Methods like `ok`, `and_then`, etc are called combinators. Combinators are functions which perform operations on a value and even change the values. In combinators you can chain function calls.

**`.ok()`** — converts `Result` into `Option`, discarding the error:
- `Ok(value)` → `Some(value)`
- `Err(e)` → `None` (error is dropped)

So `fs::read_to_string(filename)` returns `Result<String, io::Error>`, and `.ok()` turns it into `Option<String>`.

**`.and_then()`** — chains an operation that itself returns an `Option`, flattening the result:
- If the value is `Some(s)`, it runs the closure and returns its `Option` result
- If the value is `None`, it short-circuits and returns `None` immediately

> **Flattening** means removing one layer of nesting: `Option<Option<T>>` → `Option<T>`.
> Think of it as taking the value out of the inner box so you only have one box left:
> - `Some(Some(10))` → `Some(10)`
> - `Some(None)` → `None`
> - `None` → `None`

The closure returns `Option<String>` because `s.lines().next()` returns `Option<&str>` and `.map(|s| s.to_owned())` converts it to `Option<String>`.

Without `.and_then()` you'd get `Option<Option<String>>` because the closure returns `Option<String>` wrapped in another `Option`. `.and_then()` flattens that into just `Option<String>`.

`Option<Option<String>>` has three possible values:
- `None` — outer Option is empty
- `Some(None)` — outer Option has a value, but inner is empty
- `Some(Some(String))` — both layers have a value

`None` and `Some(None)` both mean "no string" — they're semantically the same, so the extra layer adds no information. `.and_then()` collapses them into a flat `Option<String>`:
- `None` → `None`
- `Some(None)` → `None`
- `Some(Some(s))` → `Some(s)`

It's equivalent to `map` + `flatten`:
```
.and_then(f)  ==  .map(f).flatten()
```

So the full chain in `read_first_line2`:
```rust
fs::read_to_string(filename)   // Result<String, io::Error>
    .ok()                      // Option<String>
    .and_then(|s| {            // runs only if Some, flattens Option<Option<_>>
        s.lines().next()       // Option<&str>
            .map(|s| s.to_owned()) // Option<String>
    })                         // Option<String>
```

## Converting `Option` → `Result`

**`.ok_or(err)`** — converts `Option` into `Result`, providing an error value for the `None` case:
- `Some(value)` → `Ok(value)`
- `None` → `Err(err)`

```rust
let x: Option<i32> = Some(5);
let y: Result<i32, &str> = x.ok_or("no value");  // Ok(5)

let x: Option<i32> = None;
let y: Result<i32, &str> = x.ok_or("no value");  // Err("no value")
```

**`.ok_or_else(|| err)`** — same but takes a closure, so the error is only computed if needed (lazy):
```rust
None.ok_or_else(|| format!("missing value at line {}", line_number))
// error string is only constructed if the Option is None
```

Use `.ok_or_else()` over `.ok_or()` when building the error value is expensive or has side effects.

**The difference as `match` expressions:**

`.ok_or(expensive_fn())` — `expensive_fn()` is evaluated **before** the match, always:
```rust
let err = expensive_fn();  // called regardless of Some or None
match option {
    Some(v) => Ok(v),
    None => Err(err),
}
```

`.ok_or_else(|| expensive_fn())` — `expensive_fn()` is only called inside the `None` arm:
```rust
match option {
    Some(v) => Ok(v),
    None => Err(expensive_fn()),  // only called when None
}
```

## `Option<Result<T, E>>`

The inverse nesting — "maybe an operation that can fail":
- `None` — nothing to operate on (no input)
- `Some(Ok(value))` — had input, operation succeeded
- `Some(Err(e))` — had input, operation failed

A practical example: parsing an optional config value:
```rust
let input: Option<&str> = Some("42");
let parsed: Option<Result<i32, _>> = input.map(|s| s.parse::<i32>());
```

Here `.map()` is correct (not `.and_then()`) because the closure returns `Result`, not `Option` — you want to keep that distinction rather than flatten it away.

You can convert it to `Result<Option<T>, E>` using `.transpose()`, which swaps the two wrappers:
```rust
let result: Result<Option<i32>, _> = parsed.transpose();
```

## Combinator Summary

| From | To | Combinator | Notes |
|---|---|---|---|
| `Result<T, E>` | `Option<T>` | `.ok()` | Discards the error |
| `Option<T>` | `Result<T, E>` | `.ok_or(err)` | Provides a fixed error value |
| `Option<T>` | `Result<T, E>` | `.ok_or_else(\|\| err)` | Lazily computes the error |
| `Option<Option<T>>` | `Option<T>` | `.flatten()` | Removes one nesting layer |
| `Option<T>` | `Option<U>` | `.and_then(\|v\| Option<U>)` | Map + flatten in one step |
| `Option<Result<T,E>>` | `Result<Option<T>, E>` | [`.transpose()`](https://doc.rust-lang.org/std/option/enum.Option.html#method.transpose) | Separate `impl` on `Option<Result<T,E>>` |
| `Result<Option<T>, E>` | `Option<Result<T, E>>` | [`.transpose()`](https://doc.rust-lang.org/std/result/enum.Result.html#method.transpose) | Separate `impl` on `Result<Option<T>,E>` |

## Under the Hood: Combinators are `match`

**Combinators are just `match` expressions.** They exist purely to make chaining readable — instead of nesting `match` inside `match`, you write a single expression. The compiler generates the same code either way:

| Combinator | Called on | Returns | Equivalent `match` |
|---|---|---|---|
| `result.ok()` | `Result` | `Option` | `match result { Ok(v) => Some(v), Err(_) => None }` |
| `option.and_then(\|v\| f(v))` | `Option` | `Option` | `match option { Some(v) => f(v), None => None }` |
| `option.ok_or(err)` | `Option` | `Result` | `match option { Some(v) => Ok(v), None => Err(err) }` |
| `option.ok_or_else(\|\| err)` | `Option` | `Result` | `match option { Some(v) => Ok(v), None => Err(err()) }` — closure `err()` only called in the `None` arm |
| `option_result.transpose()` | `Option<Result<T,E>>` | `Result<Option<T>,E>` | `match self { Some(Ok(x)) => Ok(Some(x)), Some(Err(e)) => Err(e), None => Ok(None) }` |
| `result_option.transpose()` | `Result<Option<T>,E>` | `Option<Result<T,E>>` | `match self { Ok(Some(x)) => Some(Ok(x)), Ok(None) => None, Err(e) => Some(Err(e)) }` |