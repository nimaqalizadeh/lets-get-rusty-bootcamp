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
// error: without annotation, compiler infers &Car from first element, then &House doesn't match
let paintable_objects = vec![&car, &house];

// correct: tell the compiler to treat all elements as &dyn Paint
let paintable_objects: Vec<&dyn Paint> = vec![&car, &house];
```

The type annotation tells the compiler to coerce each `&Car` and `&House` into `&dyn Paint`, so they all share the same type. This is one of the primary use cases for trait objects: **heterogeneous collections** where different concrete types need to be stored together under a common trait.

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

---

# Trait Object from physical view

What is a Trait Object?
At a physical, memory level, a trait object is a "fat pointer".

When you create a standard reference like &Car, it is a single pointer just pointing to the memory address of the Car data.

Because a trait object (&dyn Paint or Box<dyn Paint>) has erased the concrete type information (it forgets whether it is a Car or a House), a single pointer is no longer enough. The program needs a way to figure out which paint() method to call at runtime.

To solve this, a trait object pointer is twice as large as a normal pointer. It contains two things:

A Data Pointer: Points to the actual data instance (the Car or House).

A Vtable Pointer: Points to a Virtual Method Table (vtable). This is a static block of memory containing pointers to the actual, concrete method implementations (like Car::paint or House::paint) for that specific type.

When you call object.paint(), Rust follows the vtable pointer, looks up the correct function address for paint, and executes it. This lookup is the "small runtime overhead" you mentioned.

The primary goal of a trait object is to enable polymorphism through type erasure—allowing your code to work with different types interchangeably at runtime without needing to know what they are at compile time.

video: lets get rusty/043.Trait Objects
