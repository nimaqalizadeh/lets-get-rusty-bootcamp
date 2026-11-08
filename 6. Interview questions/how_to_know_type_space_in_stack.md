std::mem::size_of

You can verify how much space a type would take on the stack using the `std::mem::size_of` function.

For a u8, for example:

```rust
// We'll explain this funny-looking syntax (`::<u8>`) later on.
// Ignore it for now.
assert_eq!(std::mem::size_of::<u8>(), 1);
```

1 makes sense, because a u8 is 8 bits long, or 1 byte.


## Heap

```rust
pub struct Ticket {
    title: String,
    description: String,
    status: String,
}

// TODO: based on what you learned in this section, replace `todo!()` with
//  the correct **stack size** for the respective type.
#[cfg(test)]
mod tests {
    use super::Ticket;
    use std::mem::size_of;

    #[test]
    fn string_size() {
        assert_eq!(size_of::<String>(), 24);
    }

    #[test]
    fn ticket_size() {
        // This is a tricky question!
        // The "intuitive" answer happens to be the correct answer this time,
        // but, in general, the memory layout of structs is a more complex topic.
        // If you're curious, check out the "Type layout" section of The Rust Reference
        // https://doc.rust-lang.org/reference/type-layout.html for more information.
        assert_eq!(size_of::<Ticket>(), 72);
    }
}
```

### usize

How much space do we need to store pointer, length and capacity on the stack?
It depends on the architecture of the machine you’re running on.

Every memory location on your machine has an address, commonly represented as an unsigned integer. Depending on the maximum size of the address space (i.e. how much memory your machine can address), this integer can have a different size. Most modern machines use either a 32-bit or a 64-bit address space.

Rust abstracts away these architecture-specific details by providing the usize type: an unsigned integer that’s as big as the number of bytes needed to address memory on your machine. On a 32-bit machine, usize is equivalent to u32. On a 64-bit machine, it matches u64.

**Capacity, length and pointers are all represented as usizes in Rust**


Most references1 in Rust are represented, in memory, as a pointer to a memory location.
It follows that their size is the same as the size of a pointer, a usize.

You can verify this using ```std::mem::size_of```:

```rust
assert_eq!(std::mem::size_of::<&String>(), 8);
assert_eq!(std::mem::size_of::<&mut String>(), 8);
```

A &String, in particular, is a pointer to the memory location where the String’s metadata is stored.

You can verify this using std::mem::size_of:

assert_eq!(std::mem::size_of::<&String>(), 8);
assert_eq!(std::mem::size_of::<&mut String>(), 8);

A &String, in particular, is a pointer to the memory location where the String’s metadata is stored.
If you run this snippet:

let s = String::from("Hey");
let r = &s;

you’ll get something like this in memory:
```bash
           --------------------------------------
           |                                    |
      +----v----+--------+----------+      +----|----+
Stack | pointer | length | capacity |      | pointer |
      |  |      |   3    |    5     |      |         |
      +--|  ----+--------+----------+      +---------+
         |          s                           r
         |
         v
       +---+---+---+---+---+
Heap   | H | e | y | ? | ? |
       +---+---+---+---+---+
```
It’s a pointer to a pointer to the heap-allocated data, if you will. The same goes for &mut String.