# Custom error handling
## Why Error Handling Is Hard

**Complicated by nature**

- Defining
- Propagating
- Handling or Discarding
- Reporting
  - End-Users & Developers

Errors have three different audiences:
 1. Machines (or programs): they need errors for controlling the flow
 2. End-users: they want to see friendly messages
 3. Developers: they want to have details to be able to debug


In Rust we have two types of errors:
 1. Non-recoverable errors which thrown with `panic` macro
 2. Recoverable errors

## Idiomatic Errors in Rust

An idiomatic Rust error should implement the `Error` trait, which requires:

- `Display` — provides user-facing error messages
- `Debug` — provides developer-facing details for debugging

### The `source()` method

The `Error` trait provides a `source()` method for **error chaining**:

```rust
pub trait Error: Debug + Display {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None  // default implementation
    }
}
```
- Semantically mark types as errors
- Standardizes:
  - Checking the source of an error
  - User facing reporting
  - Developer facing reporting

The `Error` trait is defined in the standard library. Error handling infrastructure and third parties are built on top of this trait, so it is important the our types build this trait to work perfect with error handling ecosystem. 

> **Note:** `&(dyn Error + 'static)` is a reference to a trait object (`dyn Error`) with a `'static` lifetime bound. `'static` here means the underlying error type owns all its data (no borrowed references), so it can live as long as needed without dangling references.

- Returns `Some(&error)` if this error wraps a lower-level error
- Returns `None` if this is a root cause error (the default)

## How structue custom error types:
Custom error types can be structs or enums. 

```rust
struct ServerError {
    status_code: u8,
    body: String,
    soruce: Box<dyn Error>
}
```
Using struct for custom error types is appropriate when we want to attach more context to our error and code does not need to distinguish between different error types. 

If our code does need to distinguish  between error types then an `enum` is more appropriate:

```rust
enum APIError {
    UserInputError(String),
    InternalError(Box<dyn Error>)
}
```
So here the code can distinguish between user input errors and internal errors. This is usefule when we want to change our code behaviour. 

We can mix enums and structs:

```rust
enum APIErrorType {
    UserInputError,
    InternalError
}

struct APIError {
    msg: String,
    source: Option<Box<dyn Error>>,
    err_type: APIErrorType
}
```

### Best practices

- Define custom error enums to represent different failure modes
- Use `thiserror` crate to reduce boilerplate for library errors
- Use `anyhow` crate for application-level error handling
- Propagate errors with the `?` operator instead of `unwrap()`
- Implement `From` trait to enable automatic conversion between error type

video/ lets get rusty: 058. Overview of error handling