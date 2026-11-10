## Trait bounds

We’ve seen two use cases for traits so far:

- Unlocking “built-in” behaviour (e.g. operator overloading)
- Adding new behaviour to existing types (i.e. extension traits)

There’s a third use case: generic programming.

Generics allow us to write code that works with a type parameter instead of a concrete type:

```rust
trait IsEven {
    fn is_even(&self) -> bool;
}

impl IsEven for i32 {
    fn is_even(&self) -> bool {
        self % 2 == 0
    }
}

impl IsEven for i64 {
    fn is_even(&self) -> bool {
        self % 2 == 0
    }
}


fn print_if_even<T>(n: T)
where
    T: IsEven + Debug
{
    if n.is_even() {
        println!("{n:?} is even");
    }
}
```
This contract is expressed with a trait bound: `T: IsEven + Debug`.
The `+` symbol is used to require that `T` implements multiple traits. `T: IsEven + Debug` is equivalent to “where T implements IsEven **and** Debug”.

### Syntax: inlining trait bounds

All the examples above used a where clause to specify trait bounds:

```rust
fn print_if_even<T>(n: T)
where
    T: IsEven + Debug
//  ^^^^^^^^^^^^^^^^^
//  This is a `where` clause
{
    // [...]
}
```

If the trait bounds are simple, you can inline them directly next to the type parameter:

```rust
fn print_if_even<T: IsEven + Debug>(n: T) {
    //           ^^^^^^^^^^^^^^^^^
    //           This is an inline trait bound
    // [...]
}
```