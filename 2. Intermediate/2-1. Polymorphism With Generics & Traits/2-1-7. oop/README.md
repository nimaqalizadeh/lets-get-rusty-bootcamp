# Understanding OOP principles

Rust implements OOP concepts differently

• Data encapsulation: Rust uses structs and enums to define custom data structures. Encapsulation (controlling access) is achieved through Rust’s module system and privacy rules (pub, private by default).

• Behavior encapsulation: Methods are associated with structs/enums using impl blocks, bundling data and behavior.

• Abstraction and polymorphism (via traits): Instead of classes and inheritance, Rust uses traits. Traits define shared interfaces (a set of method signatures) that different types can implement. This allows for polymorphism (treating different types implementing the same trait uniformly) through generics (static dispatch) and trait objects (dynamic dispatch).

• No class inheritance: Rust deliberately omits implementation inheritance.

# Shared behavior with traits

how do we define shared behavior across different types? In traditional OOP, this is often done via inheritance or interfaces.

A trait defines a collection of method signatures (and sometimes associated types or constants)
