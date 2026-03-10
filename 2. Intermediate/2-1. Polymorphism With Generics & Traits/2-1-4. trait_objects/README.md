# Trait objects

`impl Paint` is a syntax sugar for using a generic with a trait bound
When using a generic as a return type that generic must be substitued with one concrete type at compile time

So this code has an error and to fix that we should return a trait object

```rust
fn create_paintable_object(vehicle: bool) -> impl Paint {
    if vehicle {
        Car {
            info: VehicleInfo {
                make: "Honda".to_owned(),
                model: "Civic".to_owned(),
                year: 1995
            }
        }
    } else {
        House {}  // error: `if` and `else` have incompatible types
    }
}
```

The fix is to return a trait object wrapped in a `Box`:

```rust
fn create_paintable_object(vehicle: bool) -> Box<dyn Paint> {
    if vehicle {
        Box::new(Car { ... })
    } else {
        Box::new(House {})
    }
}
```

Trait objects allow us to define a type which implements a trait without knowing what that concrete type is at compile time.

`dyn` stands for dynamic dispatch, and must be behind some type of pointer like `Box`, `&`, or `Arc` because trait objects are unsized — the compiler doesn't know their size at compile time.

You can also store trait objects in a collection to hold mixed types:

```rust
let paintable_objects: Vec<&dyn Paint> = vec![&car, &house];
```

## Static dispatch vs dynamic dispatch

**Static dispatch** (`impl Trait` / generics with trait bounds):
- The compiler generates a separate concrete implementation for each type at compile time (monomorphization)
- Zero runtime overhead — function calls are resolved at compile time
- Results in larger binary size due to code duplication

```rust
fn paint_vehicle_red<T: Vehicle>(object: &T) {
    object.paint("red".to_owned());
}
```

**Dynamic dispatch** (`dyn Trait`):
- The concrete type is resolved at runtime via a vtable (a table of function pointers)
- Small runtime overhead due to the vtable lookup on each method call
- More flexible — allows heterogeneous collections and returning different types from one function

```rust
fn paint_red(object: &dyn Paint) {
    object.paint("red".to_owned());
}
```

Use static dispatch when performance is critical and the types are known at compile time. Use dynamic dispatch when you need flexibility, such as storing different types in a collection or returning different types from a function.
