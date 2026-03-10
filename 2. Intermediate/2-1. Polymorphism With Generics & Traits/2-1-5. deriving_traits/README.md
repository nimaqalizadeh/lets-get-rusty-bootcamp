# Deriving Traits

## Attributes

Attributes (`#[...]`) are **metadata annotations** attached to code items. They don't add logic themselves — they instruct the compiler or tools to do something, such as auto-generate code, configure compilation, or suppress warnings.

There are two forms:
- `#[attr]` — outer attribute, applies to the **next** item
- `#![attr]` — inner attribute, applies to the **enclosing** scope (file, module, or crate)

## The `derive` Attribute

`#[derive(...)]` tells the compiler to **automatically implement** one or more traits for a type. Instead of writing the implementation by hand, the compiler generates it for you.

```rust
#[derive(Debug, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}
```

Here, the compiler auto-generates:
- `Debug` — allows formatting the struct with `{:?}`
- `PartialEq` — allows comparing two instances with `==`

### What the compiler actually generates for `Debug`

When you write `#[derive(Debug)]`, the compiler expands it into:

```rust
impl std::fmt::Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Point")
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}
```

Which produces output like:
```
Point { x: 3, y: 1 }
```

`Debug` lives in `std::fmt` and requires implementing a `fmt` method. When you call `println!("{:?}", p1)`, it internally calls this `fmt` method. The formatter uses a builder pattern (`debug_struct`, `.field(...)`, `.finish()`) which handles nested structs, proper spacing, and pretty-print mode (`{:#?}`).

> To see the exact code the compiler generates for any `derive`, you can use [cargo-expand](https://github.com/dtolnay/cargo-expand):
> ```bash
> cargo expand
> ```

### Implementing `Debug` manually

Instead of `#[derive(Debug)]`, you can implement the trait yourself. This gives you full control over the output format:

```rust
use std::fmt;

struct Point {
    x: i32,
    y: i32,
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Point {{ x: {}, y: {} }}", self.x, self.y)
    }
}

fn main() {
    let p = Point { x: 3, y: 1 };
    println!("{:?}", p); // Point { x: 3, y: 1 }
}
```

`println!("{:?}", p)` calls `p.fmt(f)` under the hood — so as long as your type implements `fmt::Debug`, it works.

**When to implement manually instead of using `#[derive(Debug)]`:**
- You want a **custom output format**
- Your struct contains a field that **doesn't implement `Debug`**
- You want to **hide sensitive fields** (e.g. passwords, tokens)

### Why are `Debug` and `PartialEq` traits?

Traits don't just share methods between your own types — they also **plug your type into existing Rust behavior**.

When you implement `Debug` on `Point`, you don't call `fmt` yourself. But `println!`, the standard library, and every debugging tool in Rust can use it — because they are written against the `Debug` trait:

```rust
// println! internally works like this:
fn print_debug<T: Debug>(value: &T) {
    value.fmt(...);
}
```

Same with `PartialEq` — the `==` operator is syntax sugar for the trait method:

```rust
p1 == p2   // is the same as:
p1.eq(&p2)
```

Traits serve two purposes:

| Purpose | Example |
|---|---|
| Share behavior **you define** across your types | A `Summary` trait with `.summarize()` on `Article`, `Tweet`, etc. |
| **Plug your type into** existing Rust behavior | `Debug` → `println!`, `PartialEq` → `==`, `Iterator` → `for` loops |

`Debug` and `PartialEq` are the second kind — they are **contracts** that let your type work with things the standard library already wrote. Implement the trait once, and the whole ecosystem can use it.

### Commonly derivable traits

| Trait | What it enables |
|---|---|
| `Debug` | `{:?}` formatting |
| `Clone` | `.clone()` to duplicate a value |
| `Copy` | Implicit copy instead of move |
| `PartialEq` | `==` and `!=` comparisons |
| `Eq` | Full equality (requires `PartialEq`) |
| `PartialOrd` | `<`, `>`, `<=`, `>=` comparisons |
| `Ord` | Total ordering (requires `Eq` + `PartialOrd`) |
| `Hash` | Allows use as a `HashMap` key |
| `Default` | `.default()` to create a zero/empty value |

## Other Built-in Attributes

Attributes are used for much more than deriving traits. Here is a brief overview of the other categories:

| Category | Example | Purpose |
|---|---|---|
| Conditional compilation | `#[cfg(target_os = "linux")]` | Include code only on certain platforms |
| Testing | `#[test]`, `#[should_panic]` | Mark and configure test functions |
| Diagnostics | `#[allow(unused)]`, `#[deprecated]` | Control lint warnings and deprecation notices |
| Code generation | `#[inline]`, `#[cold]` | Hint the compiler on optimization |
| ABI / FFI | `#[no_mangle]`, `#[repr(C)]` | Control symbol names and memory layout |
| Documentation | `#[doc = "..."]` | Attach documentation (same as `///`) |
| Runtime | `#[panic_handler]`, `#[global_allocator]` | Configure low-level runtime behavior |
| Type system | `#[non_exhaustive]` | Signal a type may grow in future versions |
| Features | `#![feature(...)]` | Enable unstable compiler features (nightly only) |

For the full list and details, see the official Rust reference:
[Built-in Attributes Index — The Rust Reference](https://doc.rust-lang.org/reference/attributes.html#built-in-attributes-index)
