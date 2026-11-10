Operator overloading is the ability to define custom behavior for operators like +, -, *, /, ==, !=, etc.


## In Rust, operators are traits.

For each operator, there is a corresponding trait that defines the behavior of that operator. By implementing that trait for your type, you unlock the usage of the corresponding operators.

For example, the `PartialEq` trait defines the behavior of the == and != operators:

```rust
// The `PartialEq` trait definition, from Rust's standard library
// (It is *slightly* simplified, for now)
pub trait PartialEq {
    // Required method
    //
    // `Self` is a Rust keyword that stands for 
    // "the type that is implementing the trait"
    fn eq(&self, other: &Self) -> bool;

    // Provided method
    fn ne(&self, other: &Self) -> bool { ... }
}
```
When you write x == y the compiler will look for an implementation of the PartialEq trait for the types of x and y and replace x == y with x.eq(y). It’s syntactic sugar!


Default implementations

The comment on PartialEq::ne states that “ne is a provided method”.
It means that PartialEq provides a default implementation for ne in the trait definition—the { ... } elided block in the definition snippet.
If we expand the elided block, it looks like this:

pub trait PartialEq {
    fn eq(&self, other: &Self) -> bool;

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

It’s what you expect: ne is the negation of eq.
Since a default implementation is provided, you can skip implementing ne when you implement `PartialEq` for your type. It’s enough to implement eq:

```rust
struct WrappingU8 {
    inner: u8,
}

impl PartialEq for WrappingU8 {
    fn eq(&self, other: &WrappingU8) -> bool {
        self.inner == other.inner
    }
    
    // No `ne` implementation here
}
```
You are not forced to use the default implementation though


## Destructuring syntax

Furthermore, the implementation is brittle: if the struct definition changes (e.g. a new field is added), you have to remember to update the PartialEq implementation.

You can mitigate the risk by destructuring the struct into its fields:

```rust
impl PartialEq for Ticket {
    fn eq(&self, other: &Self) -> bool {
        let Ticket {
            title,
            description,
            status,
        } = self;
        // [...]
    }
}
``` 
If the definition of Ticket changes, the compiler will error out, complaining that your destructuring is no longer exhaustive.
You can also rename struct fields, to avoid variable shadowing:

```rust
impl PartialEq for Ticket {
    fn eq(&self, other: &Self) -> bool {
        let Ticket {
            title,
            description,
            status,
        } = self;
        let Ticket {
            title: other_title,
            description: other_description,
            status: other_status,
        } = other;
        // [...]
    }
}
```
Destructuring is a useful pattern to have in your toolkit, but there’s an even more convenient way to do this: derive macros.

### Destructuring clarification
The Destructuring Safety Net
By destructuring the struct into its fields `(let Ticket { title, description, status } = self;)`, you force the compiler to use exhaustive pattern matching. If you add a price field later, this destructuring line will immediately throw a compiler error because it doesn't account for the new price field. The compiler forces you to fix the implementation before the code can run, eliminating the silent bug.

Avoiding Variable Shadowing
To compare two tickets, you must destructure both self and other. If you destructure them using the exact same variable names, the variables from other will "shadow" (overwrite access to) the variables from self.


## Derive macros

A derive macro is a particular flavour of Rust macro. It is specified as an attribute on top of a struct.

```rust
#[derive(PartialEq)]
struct Ticket {
    title: String,
    description: String,
    status: String
}
```

Derive macros are used to automate the implementation of common (and “obvious”) traits for custom types. In the example above, the PartialEq trait is automatically implemented for Ticket. If you expand the macro, you’ll see that the generated code is functionally equivalent to the one you wrote manually, although a bit more cumbersome to read:

```rust
#[automatically_derived]
impl ::core::cmp::PartialEq for Ticket {
    #[inline]
    fn eq(&self, other: &Ticket) -> bool {
        self.title == other.title 
            && self.description == other.description
            && self.status == other.status
    }
}
```
The compiler will nudge you to derive traits when possible.