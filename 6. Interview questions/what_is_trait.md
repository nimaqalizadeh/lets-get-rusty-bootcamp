## Invoking a trait method

To invoke a trait method, we use the . operator, just like we do with regular methods:

```rust
let x = WrappingU32 { inner: 5 };
assert!(!x.is_zero());
```

### To invoke a trait method, two things must be true:

- The type must implement the trait.
- The trait must be in scope.

To satisfy the latter, you may have to add a use statement for the trait:

```rust
use crate::MaybeZero;
```

### This is not necessary if:

- The trait is defined in the same module where the invocation occurs.
- The trait is defined in the standard library’s prelude. The prelude is a set of traits and types that are automatically imported into every Rust program. It’s as if use std::prelude::*; was added at the beginning of every Rust module.

You can find the list of traits and types in the prelude in the Rust documentation.
https://doc.rust-lang.org/std/prelude/index.html


When a type is defined in another crate (e.g. u32, from Rust’s standard library), you can’t directly define new methods for it. If you try:

```rust
impl u32 {
    fn is_even(&self) -> bool {
        self % 2 == 0
    }
}
```
the compiler will complain:
```bash
error[E0390]: cannot define inherent `impl` for primitive types
  |
1 | impl u32 {
  | ^^^^^^^^
  |
  = help: consider using an extension trait instead
```


## Extension trait

An extension trait is a trait whose primary purpose is to attach new methods to foreign types, such as u32. That’s exactly the pattern you deployed in the previous exercise, by defining the IsEven trait and then implementing it for i32 and u32. You are then free to call is_even on those types as long as IsEven is in scope.

```rust
// Bring the trait in scope
use my_library::IsEven;

fn main() {
    // Invoke its method on a type that implements it
    if 4.is_even() {
        // [...]
    }
}
```


## Orphan rule

Things get more nuanced when multiple crates are involved. In particular, at least one of the following must be true:

- The trait is defined in the current crate
- The implementor type is defined in the current crate

This is known as Rust’s orphan rule. Its goal is to make the method resolution process unambiguous.

Imagine the following situation:

- Crate A defines the IsEven trait
- Crate B implements IsEven for u32
- Crate C provides a (different) implementation of the IsEven trait for u32
- Crate D depends on both B and C and calls `1.is_even()`

Which implementation should be used? The one defined in B? Or the one defined in C?
There’s no good answer, therefore the orphan rule was defined to prevent this scenario. Thanks to the orphan rule, neither crate B nor crate C would compile.

https://doc.rust-lang.org/reference/items/implementations.html#trait-implementation-coherence

### more explanation

Rust’s **orphan rule** dictates that you can only implement a trait for a type if you "own" at least one of them. Specifically, either:

* The **trait** must be defined in your current crate.
* The **type** must be defined in your current crate.

If both the trait and the type are "foreign" (defined in external crates or the standard library), the compiler will reject the implementation.

### Why Your Example Fails

In your code snippet, you attempt to write:

```rust
impl PartialEq for u32 { ... }

```

* `PartialEq` is a foreign trait (defined in `std`).
* `u32` is a foreign type (defined in `std`).

Because you don't own either the trait or the type, this is an "orphan" implementation, and the Rust compiler throws an error.

### The Problem It Solves (Coherence)

The orphan rule exists to ensure **method resolution is always unambiguous**.

Using the scenario from your text:

1. **Crate A** defines the `IsEven` trait.
2. **Crate B** implements `IsEven` for `u32`.
3. **Crate C** implements `IsEven` for `u32` completely differently.
4. **Crate D** imports both Crate B and Crate C, and runs `5.is_even()`.

Without the orphan rule, Crate D's compiler wouldn't know whether to use Crate B's version or Crate C's version. Because of the orphan rule, crates B and C are fundamentally forbidden from implementing Crate A's trait on a standard library type. Only Crate A (the trait owner) can implement it for `u32`.

### The Standard Workaround: The Newtype Pattern

If you absolutely need to implement a foreign trait on a foreign type, Rust developers use the **Newtype pattern**. You wrap the foreign type in a tuple struct that you define locally:

```rust
// You own this struct, so it is local to your crate
struct MyU32(u32);

// Now this is completely valid, because the type `MyU32` is local
impl PartialEq for MyU32 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

```

