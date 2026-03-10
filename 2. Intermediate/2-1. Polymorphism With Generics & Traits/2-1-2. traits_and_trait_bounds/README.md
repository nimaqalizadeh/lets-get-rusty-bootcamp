# Traits

## What is a Trait?

A trait defines shared behavior. It can contain:

- **Signature only** — no body, every implementing type must provide its own definition.
- **Default implementation** — a body is provided, types get it for free but can override it. If the default is acceptable, the `impl` block can be left empty.

```rust
trait Park {
    fn park(&self);                    // signature only — must be implemented
}

trait Paint {
    fn paint(&self, color: String) {   // default implementation — can be overridden
        println!("painting object: {}", color);
    }
}
```

## Why do we need Traits?

### 1. Prevent duplication

Instead of writing the same method logic for every type, you define the behavior once in a trait and implement it per type. Each type can customize the behavior, but you only describe _what_ is needed once.

### 2. Enforce an exact interface (contracts)

A trait is a contract. It says: "any type that implements me _must_ provide these methods." This makes code safer — if a type doesn't implement all required methods, it won't compile.

### 3. Polymorphism — write code that works for many types

Traits let you write a function that works for _any_ type that satisfies a given interface, instead of hardcoding a single type.

There are two forms:

- **Compile-time (generics)**: `fn foo<T: Trait>(x: T)` — the compiler generates a specific version for each concrete type used. Zero runtime cost.
- **Runtime (trait objects)**: `fn foo(x: &dyn Trait)` — the concrete type is resolved at runtime. More flexible, small overhead.

## Traits vs Inheritance

In classical inheritance, a subclass inherits both **data** (fields) and **behavior** (methods) from a parent.

Traits only share **behavior** (methods) — they cannot carry data fields.

So when multiple types need to share common data, you extract it into a separate struct and embed it as a field:

```rust
struct VehicleInfo { make: String, model: String, year: u16 }

struct Car   { info: VehicleInfo }
struct Truck { info: VehicleInfo }
```

`Car` and `Truck` share data by composition (`VehicleInfo` field), and share behavior by both implementing the same traits (`Park`, `Paint`).

## Trait Bounds

A **trait bound** is a constraint on a generic type: "this `T` must implement this trait." It restricts which types are allowed to be passed in.

A "trait bound" (or type bound) is how you tell the compiler/type-checker: "I don't care exactly what type this is, as long as it implements this specific behavior." This is incredibly powerful because it allows you to write generic functions that are still safe to use.

To call a user-defined method on `T` inside a generic function, that method must come from a trait listed as a bound — the compiler needs a guarantee that every possible `T` has that method. Without a bound, you can accept any type but cannot call any user-defined methods on it.

There are four ways to write trait bounds:

**1. Inline with the generic**

```rust
fn paint_red<T: Paint>(object: &T) { ... }
```

**2. `where` clause** — cleaner when there are multiple bounds

```rust
fn paint_vehicle_red<T>(object: &T) where T: Paint + Park { ... }
```

`+` combines multiple bounds — `T` must implement both `Paint` and `Park`.

**3. `impl Trait` in parameter position** — syntactic sugar for inline, generic (caller chooses the type)

```rust
fn paint_red(object: &impl Paint) { ... }
// equivalent to:
fn paint_red<T: Paint>(object: &T) { ... }
```

Use when you have a single parameter and don't need to name or reuse the type. However, if you need two parameters to be the **same** type, use inline instead:

```rust
fn paint_both<T: Paint>(a: &T, b: &T) { ... }   // a and b must be the same type
fn paint_both(a: &impl Paint, b: &impl Paint) { ... }  // a and b can be different types
```

**4. `impl Trait` in return position** — not generic, the function decides the type

```rust
fn create_paintable_object() -> impl Paint { House {} }
```

The concrete type (`House`) is resolved at compile time — no overhead. The `impl Paint` simply hides the type from the caller, who can only use `Paint` methods:

```rust
let object = create_paintable_object();
object.paint("red".to_owned());  // OK — Paint method
object.some_house_method();      // ERROR — caller only sees impl Paint, not House
```

Useful for encapsulation (swap `House` for another type later without breaking callers) and required when returning closures or complex iterator chains whose types can't be named.

**Key constraint:** only one concrete type can be returned. This is illegal:

```rust
fn create_paintable_object(flag: bool) -> impl Paint {
    if flag { House {} } else { Car { ... } }  // ERROR — two different types
}
```

For returning different types at runtime, use `dyn Paint` instead.

### Note: trait bounds and `impl` blocks

An `impl` block on a concrete type (e.g. `impl Car { fn park(&self) {} }`) only belongs to that type. It has no effect inside a generic function:

```rust
fn do_something<T>(object: &T) {
    object.park();  // ERROR — T is not Car, compiler doesn't know T has park()
}
```

The existence of traits or `impl` blocks elsewhere in the code does not affect an unbound generic. The compiler only errors when you try to call a method on `T` that isn't guaranteed by a bound. A function body that doesn't call anything on `T` compiles fine without any bounds:

```rust
fn do_something<T>(object: &T) -> String {
    "placeholder".to_owned()  // fine — nothing is called on T
}
```

## Traits vs Python Inheritance

In Python, a class bundles data, behavior, and inheritance into one concept. Rust separates these into distinct building blocks:

| Python (OOP)                    | Rust equivalent                          |
| ------------------------------- | ---------------------------------------- |
| Class with attributes           | Struct with fields                       |
| Class with methods              | `impl` block on a struct                 |
| Inheritance of data             | Composition (embed a struct as a field)  |
| Inheritance of behavior         | Implement a trait                        |
| Method shared across subclasses | Default implementation in a trait        |
| Polymorphism via base class     | Generics with trait bounds / `dyn Trait` |

The key difference: Rust separates data (struct), behavior (impl/trait), and polymorphism (generics/trait bounds) into distinct concepts, while Python bundles them all into a class hierarchy.

video: lets get rusty/040.Trait 041.Trait Bounds
