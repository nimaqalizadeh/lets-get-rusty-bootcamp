# Understanding OOP principles

Rust implements OOP concepts differently

• Data encapsulation: Rust uses structs and enums to define custom data structures. Encapsulation (controlling access) is achieved through Rust’s module system and privacy rules (pub, private by default).

• Behavior encapsulation: Methods are associated with structs/enums using impl blocks, bundling data and behavior.

• Abstraction and polymorphism (via traits): Instead of classes and inheritance, Rust uses traits. Traits define shared interfaces (a set of method signatures) that different types can implement. This allows for polymorphism (treating different types implementing the same trait uniformly) through generics (static dispatch) and trait objects (dynamic dispatch).

• No class inheritance: Rust deliberately omits implementation inheritance.

see `math_utils.rs`

# Shared behavior with traits

how do we define shared behavior across different types? In traditional OOP, this is often done via inheritance or interfaces.

A trait defines a collection of method signatures (and sometimes associated types or constants)

see `trait_definition.rs`

# Traits for polymorphism: impl Trait

One of the main benefits of traits is enabling polymorphism. We can write functions that accept any type implementing a specific trait. The simplest way is to use impl Trait syntax in the parameter type. This uses static dispatch (monomorphization) – the compiler generates specialized code for each concrete type used.

see `impl_traits.rs`

# Traits for polymorphism: trait bounds

An alternative, more verbose syntax for the same static dispatch is using trait bounds with generic type parameters. This is necessary in more complex scenarios, such as when multiple parameters need the same generic type or when implementing traits for generic types.

A common pitfall: impl Trait versus “Any” Trait It’s important to clarify a common point of confusion about impl Trait. When you see fn notify(`item: &impl Summarizable`), it means the function can be called with a reference to any single, concrete type that implements Summarizable.
For example, you can call it with &Tweet or call it with &NewsArticle. However, it does not mean you can mix different concrete types within the same data structure. The compiler resolves impl Trait to a specific, single type at compile time for each use case. This means you cannot, for example, create a Vec that holds both Tweets and NewsArticles and pass it to a function expecting `Vec<impl
Summarizable>`.

```rust
// This is fine:
// notify(&my_tweet);
// notify(&my_article);
// This will NOT compile:
// let items: Vec<&impl Summarizable> = vec![&my_tweet, &my_
article];
// error: `impl Trait` not allowed in path parameters
```

The compiler needs to know the exact size of the elements in the Vec at compile time, and Tweet and NewsArticle are different types with different sizes. To handle collections of different types that share a trait, you need dynamic dispatch using trait objects (`&dyn Summarizable`), which we will cover in the next section.
