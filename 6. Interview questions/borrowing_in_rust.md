## The perks of Rust’s ownership system

Rust’s ownership system is designed to ensure that:

- Data is never mutated while it’s being read
- Data is never read while it’s being mutated
- Data is never accessed after it has been destroyed

These constraints are enforced by the borrow checker, a subsystem of the Rust compiler, often the subject of jokes and memes in the Rust community.

Ownership is a key concept in Rust, and it’s what makes the language unique. Ownership enables Rust to provide memory safety without compromising performance. All these things are true at the same time for Rust:

- There is no runtime garbage collector
- As a developer, you rarely have to manage memory directly
- You can’t cause dangling pointers, double frees, and other memory-related bugs

Languages like Python, JavaScript, and Java give you 2. and 3., but not 1.
Language like C or C++ give you 1., but neither 2. nor 3.


## Borrowing

It is desirable to have methods that can read the value of a variable without taking ownership of it.
Programming would be quite limited otherwise. In Rust, that’s done via borrowing.

Whenever you borrow a value, you get a reference to it.
References are tagged with their privileges:

- Immutable references (&) allow you to read the value, but not to mutate it
- Mutable references (&mut) allow you to read and mutate the value

Going back to the goals of Rust’s ownership system:

- Data is never mutated while it’s being read
- Data is never read while it’s being mutated

To ensure these two properties, Rust has to introduce some restrictions on references:

- You can’t have a mutable reference and an immutable reference to the same value at the same time
- You can’t have more than one mutable reference to the same value at the same time
- The owner can’t mutate the value while it’s being borrowed
- You can have as many immutable references as you want, as long as there are no mutable references

In a way, you can think of an immutable reference as a “read-only” lock on the value, while a mutable reference is like a “read-write” lock.

All these restrictions are enforced at compile-time by the borrow checker.