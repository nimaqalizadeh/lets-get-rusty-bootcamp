# Orphan rule

In order to implement a trait on a given type either the trait or the type must be defined on the current crate, without this rule two crates could implement the same trait on a same type and rust would not know which implementation to use

There is a way to get around this rule by creating a wrapper type.

## Example

`Point` is defined in the library crate (`src/lib.rs`), which already implements `PartialEq` for it:

```rust
// src/lib.rs
pub struct Point { pub x: i32, pub y: i32 }

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
```

From `main.rs` (the binary crate), both `Point` and `PartialEq` are foreign. If `main.rs` could also implement `PartialEq` for `Point`, there would be two conflicting implementations and the compiler wouldn't know which to use. The orphan rule blocks this entirely:

```rust
// FAILS in main.rs — neither Point nor PartialEq belongs to this crate
impl PartialEq for Point { ... }
```

The fix is to wrap `Point` in a new type defined in `main.rs`. `PointWrapper` is a distinct type owned by the binary crate, so its `PartialEq` impl is the only one for that type — no conflict:

```rust
struct PointWrapper(Point); // owned by the binary crate

impl PartialEq for PointWrapper { // allowed — PointWrapper is ours
    fn eq(&self, other: &Self) -> bool {
        self.0.x == other.0.x && self.0.y == other.0.y
    }
}
```

`self.0` accesses the inner `Point` of the tuple struct.

## Note on visibility

By default, everything in Rust is private, with two exceptions:

- Associated items in a `pub` trait are public by default
- Enum variants in a `pub` enum are public by default

Although the `impl PartialEq for Point` block in `src/lib.rs` has no `pub` keyword, it is still accessible from outside the crate. This is because `PartialEq` is a `pub` trait from `std`, so its associated items (like `eq`) are public by default. And since `Point` itself is `pub`, anyone who imports `Point` can use the `PartialEq` impl.

Reference: [Rust Reference — Visibility and Privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html)

video: lets get rusty/045.Orphan rule
