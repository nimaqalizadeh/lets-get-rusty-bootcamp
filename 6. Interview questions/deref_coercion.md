## Deref
The `Deref` trait is the mechanism behind the language feature known as deref coercion.
The trait is defined in the standard library, in the `std::ops` module:

```rust
// I've slightly simplified the definition for now.
// We'll see the full definition later on.
pub trait Deref {
    type Target;
    
    fn deref(&self) -> &Self::Target;
}
```

`type Target` is an **associated type**.
**It’s a placeholder for a concrete type that must be specified when the trait is implemented**.

## Deref coercion

By implementing `Deref<Target = U>` for a type `T` you’re telling the compiler that `&T` and `&U` are somewhat interchangeable.

In particular, you get the following behavior:

- References to `T` are implicitly converted into references to U (i.e. `&T` becomes `&U`)
- You can call on `&T` all the methods defined on `U` that take &self as input.

There is one more thing around the **dereference operator**, *, but we don’t need it yet (see std’s docs if you’re curious).
String implements Deref

String implements Deref with Target = str:

```rust
impl Deref for String {
    type Target = str;
    
    fn deref(&self) -> &str {
        // [...]
    }
}
```

Thanks to this implementation and deref coercion, a `&String` is automatically converted into a `&str` when needed.
Don’t abuse deref coercion

Deref coercion is a powerful feature, but it can lead to confusion.
Automatically converting types can make the code harder to read and understand. If a method with the same name is defined on both T and U, which one will be called?


## Deep Dive


### `deref` is what runs when you write `*`

`*` is the **dereference operator**. The `Deref` trait lets your own type decide what `*` does.

```rust
use std::ops::Deref;

struct MyBox<T>(T);

impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

let x = MyBox(5);
assert_eq!(*x, 5);
```

`MyBox` is not a reference, so `*x` only compiles because `MyBox` implements `Deref`.
The compiler rewrites it for you:

```rust
*x                    // what you write
*(x.deref())          // what the compiler runs
```

So `*x` is two steps:

1. Call `x.deref()`, which gives a `&T`.
2. Apply the plain, built-in `*` on that `&T` to get to the `T`.

### Why does `deref` return a reference and not the value?

If `deref` returned `T` directly, it would have to **move** the value out of `self`.
After one `*x`, the inner value would be gone.

Returning `&T` only borrows it. That is why the compiler still needs the extra `*` in step 2.

### `*` on a plain reference

For `&T` and `&mut T`, `*` is built into the language. No method is called.

```rust
let r = &5;
assert_eq!(*r, 5);   // built-in dereference
```

| You write `*x` and `x` is...          | What happens                    |
|---------------------------------------|---------------------------------|
| a reference (`&T`, `&mut T`)          | Built-in dereference            |
| a type that implements `Deref`        | `*(x.deref())`                  |
| anything else                         | Compile error                   |

### Writing through `*` needs `DerefMut`

`Deref` only gives a shared reference, so it is read-only.
To assign through `*`, the type must also implement `DerefMut`:

```rust
use std::ops::DerefMut;

impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

let mut y = MyBox(5);
*y = 10;              // runs *(y.deref_mut()) = 10
```

`DerefMut` has no `type Target` of its own. It reuses the one from `Deref`.

### How this connects to deref coercion

Deref coercion is the compiler calling `deref` for you, **without you writing `*`**.
It does this when a reference has the wrong type but `deref` can fix it, and it repeats as many times as needed.

```rust
fn greet(name: &str) {
    println!("Hello, {name}");
}

let name = MyBox(String::from("Rust"));

greet(&name);                    // with deref coercion
greet(name.deref().deref());     // what the compiler does
```

`&MyBox<String>` becomes `&String` (first `deref`), which becomes `&str` (second `deref`).

### You cannot move out through `*`

`*x` goes through a shared reference, so you can read or copy the value, but not take ownership of it:

```rust
let name = MyBox(String::from("Rust"));
let s: String = *name;   // error[E0507]: cannot move out of dereference of `MyBox<String>`
let s: &String = &*name; // fine: borrow it instead
```

It worked with `MyBox(5)` above because `i32` is `Copy`.
`Box<T>` is the one exception: it is special in the compiler, and `*my_box` can move the value out.
