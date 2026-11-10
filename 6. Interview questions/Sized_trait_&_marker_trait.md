## Sized

There’s more to `&str` than meets the eye, even after having investigated deref coercion.
From our previous discussion on memory layouts, it would have been reasonable to expect `&str` to be represented as a single `usize` on the stack, a pointer. That’s not the case though. `&str` stores some metadata next to the pointer: **the length of the slice it points to**. Going back to the example from a previous section:

```rust
let mut s = String::with_capacity(5);
s.push_str("Hello");
// Create a string slice reference from the `String`, 
// skipping the first byte.
let slice: &str = &s[1..];
```

In memory, we get:
```bash
                    s                              slice
      +---------+--------+----------+      +---------+--------+
Stack | pointer | length | capacity |      | pointer | length |
      |    |    |   5    |    5     |      |    |    |   4    |
      +----|----+--------+----------+      +----|----+--------+
           |        s                           |  
           |                                    |
           v                                    | 
         +---+---+---+---+---+                  |
Heap:    | H | e | l | l | o |                  |
         +---+---+---+---+---+                  |
               ^                                |
               |                                |
               +--------------------------------+
```

What’s going on?
### Dynamically sized types

`str` is a dynamically sized type (DST).
A DST is a type whose size is not known at compile time. **Whenever you have a reference to a DST, like `&str`, it has to include additional information about the data it points to**. It is a **fat pointer**.
In the case of `&str`, it stores the length of the slice it points to. We’ll see more examples of DSTs in the rest of the course.

## The Sized trait

Rust’s std library defines a trait called `Sized`.

```rust
pub trait Sized {
    // This is an empty trait, no methods to implement.
}
```

A type is `Sized` if its size is known at compile time. In other words, **it’s not a DST**.

## Marker traits

Sized is your first example of a marker trait.
A marker trait is a trait that doesn’t require any methods to be implemented. It doesn’t define any behavior. It only serves to mark a type as having certain properties. **The mark is then leveraged by the compiler to enable certain behaviors or optimizations**.

## Auto traits

In particular, `Sized` is also an auto trait.
You don’t need to implement it explicitly; the compiler implements it automatically for you based on the type’s definition.

### Examples

All the types we’ve seen so far are `Sized`: `u32`, `String`, `bool`, etc.

`str`, as we just saw, is not `Sized`.
`&str` is `Sized` though! We know its size at compile time: two usizes, one for the pointer and one for the length.
## Common confusion: "Hello" is hardcoded, so why is `str` a DST?

### Q1: A string literal like `"Hello"` is stored in the binary and has a `'static` lifetime. Doesn't that make it a fixed size?

No. These are two separate ideas:

- **`'static` is about lifetime (how long the data lives).** `"Hello"` is stored in the binary's read-only data section, so it lives for the whole program. That's why its type is `&'static str`.
- **`Sized` / DST is about the type (whether the type fixes the size).** `'static` says nothing about how many bytes the data takes.

So `"Hello"` is an unsized `str` that lives forever, reached through a sized fat pointer `&'static str`.

### Q2: The compiler can see `"Hello"` is 5 bytes. Why is its size "unknown at compile time"?

The compiler **does** know the length of that literal. "Dynamically sized" describes the **type** `str`, not the value `"Hello"`.

Every string literal has the same type, whatever its length:

```rust
let a = "Hi";      // &'static str
let b = "Hello";   // &'static str (same type, different length)
```

If code only knows "this is a `str`", it can't tell how big it is. That's what DST means: **the size is not part of the type**.

Compare with arrays, where the length *is* part of the type:

- `i32` → always 4 bytes
- `[u8; 5]` → always 5 bytes (`[u8; 5]` and `[u8; 6]` are different types)
- `str` → could be 2 bytes, 5 bytes, or 10 MB read from a file, all the same type

### Q3: So where does the known length go?

The compiler writes it into the **fat pointer** at compile time:

```
b = [ address of "Hello" in the binary | len = 5 ]
```

The length is stored in the `&str` value, not in the type. That's why you can never have a bare `str` on the stack. It always sits behind a pointer: `&str`, `Box<str>`, or inside a `String`.

```rust
use std::mem::size_of;

size_of::<&str>();      // 16 on 64-bit: pointer + length
size_of::<&[u8; 5]>();  // 8: just a pointer, the length is in the type
// size_of::<str>();    // compile error: `str` doesn't have a size known at compile time
```

### Q4: Why didn't Rust make string literals a sized type like `&[u8; 5]`?

It could have. Byte-string literals actually are: `b"Hello"` has type `&[u8; 5]`. But if `"Hi"` and `"Hello"` were different types, this function couldn't accept both without generics:

```rust
fn greet(s: &str) { /* ... */ }  // accepts a string of any length
```

Making all strings one unsized type `str`, with the length carried in the reference, lets `&str` work the same for literals, slices of a `String` (`&s[1..]`), file contents, and so on.

### One-line summary

> `'static` means *lives forever*. DST means *the type doesn't fix the size*. The compiler knows the size of the **value** `"Hello"`, but the **type** `str` doesn't record it. So the length goes into the fat pointer `&str`, which is itself `Sized` (two `usize`s).

## Wrap-up: what gets mixed up when you first learn `str`

When you first meet `str`, it's easy to mix up these ideas:

- **"Hardcoded means fixed size."** `"Hello"` is in the binary and the compiler can see it's 5 bytes, so it seems like `str` should be `Sized`.
- **"`'static` means known size."** `'static` is about *how long* the data lives, not *how big* it is.
- **"`&str` always points to the binary."** It can point anywhere: the binary, the heap, the stack.
- **"Every `&str` is `'static`."** Only references to data that lives for the whole program are `'static`.
- **"There is no `str` type, only `&str`."** `str` exists. You just can't hold it by value.

### The actual picture

**1. Size comes from the type.**

- `u32` → always 4 bytes.
- `[u8; 5]` → always 5 bytes (the length is part of the type).
- `str` → the type alone doesn't tell you how many bytes. That's why it's a DST: the size isn't known at compile time *from the type*.

The same applies to slices: `[u8]` (no length) is a DST, just like `str`. Arrays with a fixed length are `Sized`, slices are not.

**2. Where the data lives doesn't matter.**

A `&str` is always the same fat pointer, `[ ptr | len ]`, wherever the bytes are:

```rust
let a: &str = "Hello";      // points into the binary (read-only data)
let s = String::from("Hi");
let b: &str = &s;           // points into the heap
```

**3. Lifetime depends on what it points to.**

A `&str` borrowed from a `String` is tied to that `String`'s lifetime, so it's **not** `'static`:

```rust
let b: &str;
{
    let s = String::from("Hi");
    b = &s;   // ❌ error: `s` does not live long enough
}
```

Only data that lives for the whole program, such as string literals, gives you a `&'static str`.

**4. `str` exists, but always behind a pointer.**

You can't hold a `str` by value because its size is unknown, so it always goes behind a pointer: `&str`, `Box<str>`, `Rc<str>`, or inside a `String`.

The reason for this design: **one type for all strings, of any length, stored anywhere.** That's what lets a single function accept all of them:

```rust
fn greet(s: &str) { println!("Hello, {s}"); }

greet("literal");               // from the binary
greet(&String::from("heap"));   // from the heap
greet(&"sliced"[1..]);          // a slice of a literal
```
