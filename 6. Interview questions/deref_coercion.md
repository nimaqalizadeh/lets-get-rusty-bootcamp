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
