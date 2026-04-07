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
