# Deref Coercion

Implicit deref coercion allows the Rust compiler to automatically coerce a reference of one type into a reference of another type. This mechanism is built around the `Deref` and `DerefMut` traits.

## The `Deref` Trait

```rust
pub trait Deref {
    type Target: ?Sized;

    fn deref(&self) -> &Self::Target;
}
```

**`type Target: ?Sized`** — an associated type that declares what type this `Deref` impl dereferences *to* (e.g., `str` for `String`, `T` for `Box<T>`). The `: ?Sized` bound means `Target` is allowed to be an unsized type like `str` or `[T]`, which don't have a known size at compile time. Without `?Sized`, only sized types would be permitted.

**`fn deref(&self) -> &Self::Target`** — returns a reference to the target type. Every time the dereference operator `*` is used on a type, the compiler calls this method under the hood. The number of calls needed is resolved at **compile time**, so there is no runtime cost.

## How Coercion Works

If `T` implements `Deref<Target = U>`, then:

> - **`T`** — the type you have (e.g., `Box<i32>`)
> - **`U`** — the type you get after dereferencing (e.g., `i32`)
> - **`v`** — a variable of type `T` (e.g., `let v = Box::new(5)`)

- `*v` (where `v: T`) is equivalent to `*Deref::deref(&v)`

```rust
let b = Box::new(5);
*b        // what you write
*b.deref() // what the compiler actually does
```

- `&T` is automatically coerced to `&U`

```rust
fn takes_str(s: &str) {}

let s = String::from("hello");
takes_str(&s); // &String passed, &str expected — compiler coerces automatically
```

- `T` implicitly gains all methods of `U` that take a `&self` receiver

```rust
// String derefs to str — so all &self methods of str are available on String
let s = String::from("hello");
s.len();            // str::len()      — number of bytes
s.contains("ell");  // str::contains() — check substring
s.to_uppercase();   // str::to_uppercase()
s.starts_with("he");// str::starts_with()

// Vec<T> derefs to [T] — so all &self methods of slices are available on Vec
let v = vec![3, 1, 2];
v.len();            // slice::len()
v.contains(&1);     // slice::contains()
v.first();          // slice::first() — returns Option<&T>

// Box<T> derefs to T — so all &self methods of T are available on Box
let b = Box::new(String::from("hello"));
b.len();            // calls str::len() — two hops: Box<String> → String → str
```

Only `&self` methods are gained this way. For `&mut self` methods (like `sort()` on a slice), `DerefMut` is required.

The compiler will silently insert as many `deref` calls as needed to satisfy the target type. For example:

- `String` → `&str` (because `String: Deref<Target = str>`)
- `Vec<T>` → `&[T]` (because `Vec<T>: Deref<Target = [T]>`)
- `Box<T>` → `&T` (because `Box<T>: Deref<Target = T>`)

This is why you can pass a `&String` where a `&str` is expected, or a `&Vec<T>` where a `&[T]` is expected — the compiler inserts the coercion automatically.

**Coercion direction:** coercion always goes from the smart/owning type toward the simpler/borrowed type — never the reverse.

- **Smart/owning type** — a type that owns the data and manages its memory (allocates/frees it). Examples: `String`, `Vec<T>`, `Box<T>`, `Rc<T>`.
- **Simple/borrowed type** — a type that just points to data owned by someone else, with no ownership or allocation. Examples: `&str`, `&[T]`, `&T`.

```
String  (owns heap data)  →  &str  (just a view into it)
Vec<T>  (owns heap data)  →  &[T]  (just a view into it)
Box<T>  (owns heap data)  →  &T    (just a view into it)
```

`&String` → `&str` is possible, but `&str` → `&String` is not, because `str` does not implement `Deref<Target = String>`. You also cannot add that impl yourself — the **orphan rule** forbids implementing a trait on a type when both the trait and the type are from an external crate (in this case, both `Deref` and `str` are from `std`). Going the other direction always requires an explicit conversion (e.g., `s.to_string()`).

The coercion chain is only as long as the unbroken sequence of `Deref` impls. If `T` does not implement `Deref`, the chain stops at `&T`:

```rust
struct Foo; // no Deref impl

let b = Box::new(Foo);
let _: &Foo = &b;  // works: Box<Foo> → &Foo
// can't go further — Foo has no Deref impl

// Contrast with Box<String> where String: Deref<Target = str>
let b = Box::new(String::from("hello"));
let _: &String = &b;  // Box<String> → &String
let _: &str    = &b;  // Box<String> → &String → &str (two hops)
```

## Implementing `Deref`

```rust
use std::ops::Deref;

struct MyBox<T>(T);

impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

let x = MyBox(5);
assert_eq!(5, *x); // compiler rewrites this as *(x.deref())
```

## `DerefMut` for Mutable Coercion

`DerefMut` works the same way but for mutable references (`&mut T` → `&mut U`). The coercion rules are:

| From | To | Requires |
|------|-----|----------|
| `&T` | `&U` | `T: Deref<Target = U>` |
| `&mut T` | `&mut U` | `T: DerefMut<Target = U>` |
| `&mut T` | `&U` | `T: Deref<Target = U>` |

Note: `&T` cannot coerce to `&mut U` — mutability can only be relaxed, not introduced.

To use `DerefMut`, implement both `Deref` and `DerefMut` on your type. `DerefMut` reuses the `Target` type from `Deref` — you must implement `Deref` first:

```rust
use std::ops::{Deref, DerefMut};

struct Wrapper(Vec<i32>);

impl Deref for Wrapper {
    type Target = Vec<i32>;
    fn deref(&self) -> &Vec<i32> {
        &self.0
    }
}

impl DerefMut for Wrapper {
    // no Target here — reuses the one defined in Deref
    fn deref_mut(&mut self) -> &mut Vec<i32> {
        &mut self.0
    }
}

let mut w = Wrapper(vec![3, 1, 2]);

w.first();  // &self method  — works via Deref
w.sort();   // &mut self method — works via DerefMut
w.push(4);  // &mut self method — works via DerefMut
```

## When to Implement `Deref`

**Implement it when** your type transparently wraps another type and should behave like it:

```rust
// Good: MyString is just a thin wrapper around String — users expect it to behave like one
struct MyString(String);

impl Deref for MyString {
    type Target = String;
    fn deref(&self) -> &String { &self.0 }
}

let s = MyString(String::from("hello"));
println!("{}", s.len()); // deref coercion gives access to String methods
```

**Avoid implementing it when** dereferencing could fail — coercions are implicit, so failures are confusing:

```rust
// Bad: deref that can panic is surprising when inserted silently by the compiler
struct MaybeBox<T>(Option<T>);

impl<T> Deref for MaybeBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.as_ref().unwrap() // panics if None — very hard to debug at a call site
    }
}
```

**Avoid implementing it when** your type has methods that collide with the target type — it becomes unclear which method gets called:

```rust
// Confusing: both Wrapper and Vec<T> have a .len() — which one does s.len() call?
struct Wrapper(Vec<i32>);

impl Wrapper {
    fn len(&self) -> usize { 42 } // always returns 42
}

impl Deref for Wrapper {
    type Target = Vec<i32>;
    fn deref(&self) -> &Vec<i32> { &self.0 }
}

let w = Wrapper(vec![1, 2, 3]);
w.len(); // calls Wrapper::len() → 42, not Vec::len() → 3
         // but this is non-obvious to callers
```

## Common Standard Library Types That Implement `Deref`

| Type | Target |
|------|--------|
| `Box<T>` | `T` |
| `Vec<T>` | `[T]` |
| `String` | `str` |
| `Rc<T>` | `T` |
| `Arc<T>` | `T` |
| `Ref<'_, T>` | `T` |
| `MutexGuard<'_, T>` | `T` |

### `Box<T>` → `T`

```rust
let b = Box::new(5);
let x: &i32 = &b; // &Box<i32> coerces to &i32
println!("{}", b.pow(2)); // calls i32::pow directly on the Box
```

### `Vec<T>` → `[T]`

```rust
fn sum(slice: &[i32]) -> i32 { slice.iter().sum() }

let v = vec![1, 2, 3];
sum(&v); // &Vec<i32> coerces to &[i32]
```

### `String` → `str`

```rust
fn greet(name: &str) { println!("Hello, {name}"); }

let s = String::from("Alice");
greet(&s); // &String coerces to &str
```

### `Rc<T>` → `T`

```rust
use std::rc::Rc;

let r = Rc::new(String::from("hello"));
println!("{}", r.len()); // coerces Rc<String> → String, then String → str to call len()
```

### `Arc<T>` → `T`

```rust
use std::sync::Arc;
use std::thread;

let a = Arc::new(vec![1, 2, 3]);
let a2 = Arc::clone(&a);

thread::spawn(move || {
    println!("{}", a2.len()); // Arc<Vec<i32>> coerces to &Vec<i32> to call len()
}).join().unwrap();
```

### `Ref<'_, T>` → `T`

```rust
use std::cell::RefCell;

let c = RefCell::new(vec![1, 2, 3]);
let r = c.borrow(); // returns Ref<'_, Vec<i32>>
println!("{}", r.len()); // Ref<Vec<i32>> coerces to &Vec<i32> to call len()
```

### `MutexGuard<'_, T>` → `T`

```rust
use std::sync::Mutex;

let m = Mutex::new(vec![1, 2, 3]);
let guard = m.lock().unwrap(); // returns MutexGuard<'_, Vec<i32>>
println!("{}", guard.len()); // MutexGuard<Vec<i32>> coerces to &Vec<i32> to call len()
```