# Super traits

A trait can rely on other trait, the trait relyed on is called super trait.

```rust
trait Vehicle: Paint { ... }
```

Any type implementing `Vehicle` must also implement `Paint`.

Traits can also contain associated functions (no `self` parameter, called as `Type::function()`).

## Multiple supertraits

A trait can require several supertraits at once:

```rust
trait Vehicle: Paint + Display + Clone { ... }
```

## Calling supertrait methods inside a subtrait

Within a subtrait's method body, you can call the supertrait's methods:

```rust
trait Vehicle: Paint {
    fn repaint(&self) {
        self.paint("default".to_owned()); // Paint method available here
    }
}
```

## Supertrait bounds are inherited

When a function requires `T: Vehicle`, `T: Paint` is implied — no need to list both:

```rust
fn paint_vehicle_red<T>(object: &T) where T: Vehicle {
    object.paint("red".to_owned()); // works without T: Vehicle + Paint
}
```

## Chained supertraits

Supertraits can chain: if `A: B` and `B: C`, then implementing `A` requires both `B` and `C`.

```rust
trait C {
    fn c_method(&self);
}

trait B: C {
    fn b_method(&self);
}

trait A: B {
    fn a_method(&self);
}

struct Foo;

// Must implement all three: A, B, and C
impl C for Foo {
    fn c_method(&self) { println!("C"); }
}

impl B for Foo {
    fn b_method(&self) { println!("B"); }
}

impl A for Foo {
    fn a_method(&self) { println!("A"); }
}
```

A function that requires `T: A` can also call `b_method` and `c_method` — all are implied.

A real-world example from `std`: `Copy: Clone` — implementing `Copy` requires `Clone`.

```rust
#[derive(Clone, Copy)] // Copy requires Clone, so both must be derived/implemented
struct Point { x: f32, y: f32 }
```

If you try to `derive(Copy)` without `Clone`, the compiler will error.

video: lets get rusty/42.Supertraits
