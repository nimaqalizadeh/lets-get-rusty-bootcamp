# Ownership
```rust
fn print_a(a: A) -> A {
    println!("{a:?}");
    let mut a = a;
    a.x = 15;
    a
}

#[derive(Debug)]
struct A {
    x: u32,
}

fn main() {
    let a = A { x: 10 };
    let b = print_a(a);
    println!("{b:?}");
}

```

## Why `let mut a = a;` works inside the function

In the `print_a` function, `a` is received as an immutable binding, then shadowed as mutable with `let mut a = a;`. This is valid because:

- **Ownership transfer**: When `a` is passed to `print_a`, the function takes full ownership of the value. The caller's immutability does not carry over.
- **Shadowing, not mutation**: `let mut a = a;` creates a new mutable binding named `a`. The old immutable `a` is moved into it.
- **Mutability is a property of the binding, not the value**: Once you own a value, you choose how to bind it. Since there's only one owner at a time, no other code can observe the value changing unexpectedly.