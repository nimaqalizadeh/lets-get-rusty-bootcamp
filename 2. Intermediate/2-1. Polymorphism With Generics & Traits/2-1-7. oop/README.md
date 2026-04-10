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

## Trait method visibility

All methods defined in a trait are **public by default** — you cannot use `pub` on individual trait methods. The visibility is controlled at the **trait level**, not the method level.

```rust
pub trait Public {
    fn visible_everywhere(&self);  // public, since the trait is pub
}

pub(crate) trait CrateOnly {
    fn visible_in_crate(&self);    // accessible within the crate only
}

trait Private {
    fn visible_in_module(&self);   // accessible only within the module
}
```

If someone can see the trait, they can see all its methods. There is no way to make individual trait methods private or restricted.

When a type implements a trait, all the trait's methods become part of that type's public interface — even if the type's own `impl` block has private methods.

```rust
struct Engine {
    cylinders: u8,  // private field
}

impl Engine {
    fn secret(&self) {}  // private — only visible in this module
}

impl PowerSource for Engine {
    fn start(&self) {}              // public (because PowerSource is pub)
    fn power_output(&self) -> u32 { 200 }  // also public
}
```

The one caveat: the caller must **bring the trait into scope** (with `use`) to call its methods. So while the methods are public, they are only accessible where the trait is visible.

# Traits for polymorphism: impl Trait

One of the main benefits of traits is enabling polymorphism. We can write functions that accept any type implementing a specific trait. The simplest way is to use impl Trait syntax in the parameter type. This uses static dispatch (monomorphization) – the compiler generates specialized code for each concrete type used.

see `impl_traits.rs`

# Traits for polymorphism: trait bounds

An alternative, more verbose syntax for the same static dispatch is using trait bounds with generic type parameters. This is necessary in more complex scenarios, such as when multiple parameters need the same generic type or when implementing traits for generic types.

A common pitfall: impl Trait versus “Any” Trait It’s important to clarify a common point of confusion about impl Trait. When you see fn notify(`item: &impl Summarizable`), it means the function can be called with a reference to any single, concrete type that implements Summarizable.
For example, you can call it with &Tweet or call it with &NewsArticle. However, it does not mean you can mix different concrete types within the same data structure. The compiler resolves impl Trait to a specific, single type at compile time for each use case. This means you cannot, for example, create a Vec that holds both Tweets and NewsArticles and pass it to a function expecting `Vec<impl
Summarizable>`.

```rust
// This is fine:
// notify(&my_tweet);
// notify(&my_article);
// This will NOT compile:
// let items: Vec<&impl Summarizable> = vec![&my_tweet, &my_
article];
// error: `impl Trait` not allowed in path parameters
```

The compiler needs to know the exact size of the elements in the Vec at compile time, and Tweet and NewsArticle are different types with different sizes. To handle collections of different types that share a trait, you need dynamic dispatch using trait objects (`&dyn Summarizable`), which we will cover in the next section.

# Dynamic polymorphism with trait objects

what if you need a collection (like a Vec) that holds items of different concrete types, as long as they all implement the same trait?
(Why can not put different types with same trait implementation in a `Vec`?
Answer: Under the hood, a Vec stores all of its items right next to each other in a single, unbroken block of memory.

Because the items are packed tightly side-by-side, Rust uses a simple math formula to find a specific item. If you ask for the item at index 3, Rust calculates:
`Memory Address = Start_Address + (3 * Size_Of_Item)`

For this formula to work quickly and safely, every single slot in the Vec must be exactly the same size)

Or,if you need to return different types implementing a trait from a function?
(Why can not use different types implementing a same trait in a function return?
Answer: When a function finishes running, it has to hand its return value back to the code that called it. Rust allocates a specific slot of memory on the stack for this hand-off. If the compiler doesn't know how big that slot needs to be, it will refuse to compile.)

So this requires dynamic dispatch, where the specific method to call is determined at runtime

A trait object is created by taking a reference (or a smart pointer like `Box` or `Rc`) to an instance of a type that implements a trait and specifying the type as `&dyn` Trait or `Box<dyn Trait>`. The dyn
keyword indicates that method calls on this object will use dynamic dispatch.

# Object safety

Not all traits can be made into trait objects. For a trait to be used with dyn Trait, it must be object-safe. This is a set of rules that ensures the compiler can work with the trait when the concrete type is unknown at compile time. The main rules for object safety are that all methods in the trait must meet the following criteria:

1. Have a receiver (self, &self, or &mut self) as the first parameter.

2. Not use Self as a return type or in parameter types (except for the receiver).

3. Not have generic type parameters.

## A deep dive into object safety

### What problem does `dyn Trait` solve?

Say you have several types that share behavior:

```rust
trait Animal {
    fn speak(&self);
}

struct Dog;
struct Cat;
struct Cow;

impl Animal for Dog { fn speak(&self) { println!("Woof"); } }
impl Animal for Cat { fn speak(&self) { println!("Meow"); } }
impl Animal for Cow { fn speak(&self) { println!("Moo"); } }
```

Now you want a **list of mixed animals**:

```rust
let animals = vec![Dog, Cat, Cow]; // ERROR: different types
```

This doesn't work because `Vec<T>` requires all elements to be the same type `T`. The solution is a **trait object**:

```rust
let animals: Vec<Box<dyn Animal>> = vec![
    Box::new(Dog),
    Box::new(Cat),
    Box::new(Cow),
];

for a in &animals {
    a.speak(); // works — each calls its own speak()
}
```

`Box<dyn Animal>` means: "a heap-allocated something that implements `Animal`, but I've forgotten the exact type." This is **type erasure**.

### How does `dyn Trait` actually work under the hood?

A `&dyn Animal` or `Box<dyn Animal>` is **not** a regular pointer. It's a **fat pointer** — two pointers glued together:

```
    Box<dyn Animal>
    ┌─────────────┬─────────────┐
    │  data ptr   │  vtable ptr │
    └──────┬──────┴──────┬──────┘
           │             │
           ▼             ▼
      ┌──────┐    ┌───────────────────┐
      │ Dog  │    │  drop_in_place    │
      └──────┘    │  size: 0          │
                  │  align: 1         │
                  │  speak: Dog::speak│
                  └───────────────────┘
```

- **data pointer** → points to the actual `Dog` (or `Cat`, etc.) in memory
- **vtable pointer** → points to a static table holding:
  - how to drop it
  - its size and alignment
  - one function pointer per method in the trait

When you write `a.speak()`, the compiler generates roughly:

```rust
let function = vtable.speak;   // look up the function pointer
function(data_ptr);            // call it with the data pointer as `&self`
```

**Key insight:** the only thing dynamic dispatch knows about the object is a `*const ()` (the data pointer) plus the vtable. It has completely forgotten whether it was a `Dog`, `Cat`, or `Cow`.

### Two common points of confusion

**"But `Dog`'s size is known at compile time — why do we need a data pointer?"**

You're right that `Dog` itself is sized. The subtlety is that **`dyn Animal` is a different type than `Dog`**, and it's the `dyn Animal` side that's unsized. Once you coerce a `Dog` into `Box<dyn Animal>`, the compiler is no longer allowed to assume it's a `Dog`:

```rust
let a: Box<dyn Animal> = Box::new(Dog);  // Dog is known here...
// ...but from this line onward, `a` has type Box<dyn Animal>.
// The compiler has forgotten it was a Dog.

for item in &animals {       // item: &Box<dyn Animal>
    item.speak();            // "what type is behind this?" is unknowable
}
```

Inside the loop, different elements of the `Vec` might hold a `Dog` (0 bytes), a `Cat` (0 bytes), or an `Elephant` (500 bytes). They have **different sizes**, so the `Box` can't store them inline — it must store them indirectly and carry the vtable so it can figure out what to do with them at runtime. The data pointer isn't there because `Dog`'s size is unknown; it's there because `dyn Animal` is a type that _refuses to commit_ to any one concrete type.

**"The diagram shows `speak: Dog::speak` — what about `Cat` and `Cow`?"**

The diagram depicts **one specific instance** — a `Box<dyn Animal>` that happens to wrap a `Dog`. But the compiler actually generates **one vtable per `(type, trait)` pair**, baked into the binary as static read-only data:

```
<Dog as Animal> vtable      <Cat as Animal> vtable      <Cow as Animal> vtable
┌──────────────────┐        ┌──────────────────┐        ┌──────────────────┐
│ drop_in_place    │        │ drop_in_place    │        │ drop_in_place    │
│ size: 0          │        │ size: 0          │        │ size: 0          │
│ align: 1         │        │ align: 1         │        │ align: 1         │
│ speak: Dog::speak│        │ speak: Cat::speak│        │ speak: Cow::speak│
└──────────────────┘        └──────────────────┘        └──────────────────┘
```

When you write `Box::new(Dog)` and coerce it into `Box<dyn Animal>`, the fat pointer gets stamped with the address of the `<Dog as Animal>` vtable. `Box::new(Cat)` gets stamped with `<Cat as Animal>`'s vtable, and so on. Each fat pointer carries its _own_ vtable address:

```
Vec<Box<dyn Animal>>
┌─────────────────────┬─────────────────────┬─────────────────────┐
│ data→Dog | vt→Dog's │ data→Cat | vt→Cat's │ data→Cow | vt→Cow's │
└─────────────────────┴─────────────────────┴─────────────────────┘
```

So when the loop executes `a.speak()`, the generated code is literally:

```rust
(a.vtable_ptr.speak)(a.data_ptr)
```

It blindly follows whichever vtable pointer _that specific element_ carries. The first element follows `Dog`'s vtable and ends up calling `Dog::speak`; the second follows `Cat`'s and calls `Cat::speak`. The loop body has no idea which is which — **the vtable pointer itself is the selector**. That's the whole trick of dynamic dispatch: every trait object carries, as runtime data, the answer to "which implementation should I use?"

With that model in mind, every object-safety rule follows naturally from one question:

> **"Can this method be called using only a data pointer + vtable lookup?"**

If yes → object-safe. If no → not object-safe.

### Rule 1: The receiver must be "pointer-shaped"

```rust
trait Animal {
    fn speak(&self);         // receiver is &self
    fn rename(&mut self);    // receiver is &mut self
    fn consume(self);        // receiver is self (by value)
}
```

**`&self` and `&mut self` are fine.** At the machine level, `&Dog` is just a pointer (`*const Dog`). The data pointer in the fat pointer is exactly that pointer, so calling becomes:

```rust
(vtable.speak)(data_ptr)  // data_ptr IS the &self
```

**`self` by value is a problem.** Taking `self` means the function receives the entire object on the stack, not a pointer to it. To call this, the caller must know:

1. **How big** `Self` is (so it can reserve stack space)
2. **How to move** it from wherever it lived

But behind `dyn Animal`, the compiler has erased the type. Is it a 1-byte `Dog`? A 500-byte `Elephant`? It has no idea.

```rust
let a: Box<dyn Animal> = Box::new(Dog);
a.consume();
// ❌ Compiler: "I need to move Self onto the stack, but I don't know its size."
```

So the receiver must be a thin pointer (`&self`, `&mut self`, `Box<Self>`, `Rc<Self>`, `Arc<Self>`, `Pin<&mut Self>`, …). These are all one pointer at runtime, matching the data pointer in the fat pointer.

### Rule 2: Methods cannot return `Self`

```rust
trait Cloneable {
    fn dup(&self) -> Self;
}
```

Imagine calling it through a trait object:

```rust
let a: Box<dyn Cloneable> = /* something */;
let b = a.dup();
//  ^ what type is `b`?
```

The caller is looking at `dyn Cloneable`. It has no idea whether `a` is a `Dog` or a `Cat`. If `dup` returns "a `Self`", the caller can't know:

- How big the return value is
- What type to assign `b`
- Where to put it in memory

The compiler refuses because it can't generate code that handles an unknown-sized return value.

**Why `Clone` isn't object-safe:** because `Clone::clone(&self) -> Self` returns `Self`. That's why `Box<dyn Clone>` is forbidden.

**Fix:** return something concrete, or a boxed trait object:

```rust
trait Cloneable {
    fn dup(&self) -> Box<dyn Cloneable>; // ✅ fixed, known size (fat pointer)
}
```

### Rule 3: Methods cannot have generic type parameters

```rust
trait Printer {
    fn print<T: Display>(&self, value: T);
}
```

Rust generics are **monomorphized** — the compiler generates a separate copy of `print` for every concrete `T` used in the program:

```rust
// After monomorphization:
fn print_i32(&self, value: i32)
fn print_str(&self, value: &str)
fn print_bool(&self, value: bool)
// ... one per type
```

Now think about the vtable. A vtable is a **fixed-size static table** built at compile time. How many slots do we reserve for `print`?

- One for `print::<i32>`?
- One for `print::<String>`?
- One for `print::<MyCustomType>` that doesn't even exist yet?

It's impossible — `T` could be any type in the universe, even types from crates you don't control. The vtable can't enumerate them.

**Fix:** make the trait itself generic, or use a trait object as the parameter:

```rust
// Option A: generic trait (one impl per T, but no trait objects over T)
trait Printer<T> {
    fn print(&self, value: T);
}

// Option B: use &dyn to keep it object-safe
trait Printer {
    fn print(&self, value: &dyn Display);
}
```

### Rule 4: The trait cannot require `Self: Sized`

```rust
trait Animal: Sized {   // ← supertrait bound
    fn speak(&self);
}
```

`Sized` means "the size is known at compile time." Every concrete type like `Dog` or `Cat` is `Sized`. But `dyn Animal` is **unsized** — its size depends on the hidden concrete type, which is erased.

So:

- `Sized` says: "only types with a known compile-time size can implement me"
- `dyn Animal` is a type whose size is NOT known
- Therefore `dyn Animal` cannot even exist as a type that "implements Animal"

**Intuition:** requiring `Self: Sized` is explicitly saying "no trait objects allowed." It's the opposite of what we want for dynamic dispatch.

### Rule 5: No associated constants

```rust
trait Animal {
    const LEGS: u32;
    fn speak(&self);
}
```

Associated constants are resolved at compile time using the concrete type (`Dog::LEGS`, `Chicken::LEGS`). But with a trait object:

```rust
let a: Box<dyn Animal> = Box::new(Dog);
let legs = <dyn Animal>::LEGS;
//          ^^^^^^^^^^ which impl's constant?
```

The vtable doesn't store associated constants — only method function pointers. So traits with associated constants are excluded from `dyn`.

### Rule 6: `Self` can only appear in the receiver

This generalizes Rule 2. `Self` in a method signature — anywhere except the receiver — means "the concrete type of the implementor," which is erased behind `dyn`.

```rust
trait Compare {
    fn equals(&self, other: &Self) -> bool; // ❌ &Self as a parameter
}
```

If you have two `Box<dyn Compare>`, the compiler can't guarantee they hold the same concrete type:

```rust
let a: Box<dyn Compare> = Box::new(Dog);
let b: Box<dyn Compare> = Box::new(Cat);
a.equals(&*b);
// ❌ a expects &Self = &Dog, but b is a Cat.
// The type system can't enforce "both are the same concrete type" at runtime.
```

This is why `PartialEq` is not object-safe — `fn eq(&self, other: &Self)`.

### The escape hatch — `where Self: Sized`

Sometimes you want a trait that's **mostly** object-safe but has one or two methods that need `Self` or return `Self`. You can mark those methods `where Self: Sized` to **exclude them from the vtable**:

```rust
trait Animal {
    fn speak(&self);  // object-safe, goes in vtable

    fn dup(&self) -> Self           // not object-safe...
    where
        Self: Sized;                 // ...but excluded from the vtable
}

impl Animal for Dog {
    fn speak(&self) { println!("Woof"); }
    fn dup(&self) -> Self { Dog }
}

fn main() {
    // trait object works for speak()
    let a: Box<dyn Animal> = Box::new(Dog);
    a.speak();     // ✅ in vtable
    // a.dup();    // ❌ not callable through dyn — requires Sized

    // but on the concrete type, all methods work
    let d = Dog;
    d.speak();
    let _d2 = d.dup(); // ✅ works on concrete Dog
}
```

The `where Self: Sized` clause says "this method is only valid when `Self` has a known size" — and since `dyn Animal` is unsized, this method simply doesn't exist on trait objects. That lets the rest of the trait remain dyn-compatible.

### Cheat sheet

| Rule violation                       | Why it breaks dynamic dispatch                                 |
| ------------------------------------ | -------------------------------------------------------------- |
| `fn method(self)` (by value)         | Caller can't move an unknown-sized value onto the stack        |
| `fn method() -> Self`                | Caller can't receive an unknown-sized return value             |
| `fn method<T>(&self, x: T)`          | Vtable can't have infinite monomorphized slots                 |
| `trait X: Sized`                     | `dyn X` is unsized, so it can't satisfy the bound              |
| `const FOO: u32;` in trait           | Vtables hold methods, not constants                            |
| `Self` in a parameter (e.g. `&Self`) | No way to enforce "both are the same concrete type" at runtime |

**The unifying principle:**

> Dynamic dispatch replaces a concrete type with `(data_ptr, vtable_ptr)`. A trait is object-safe if — and only if — every method can be called using nothing but those two pointers.

Anything that would require knowing the real concrete type (its size, its layout, its associated constants, a type parameter it depends on) is forbidden in the vtable.

### A fully worked example

```rust
// ❌ NOT object-safe — violates several rules
trait Bad {
    const ID: u32;                              // Rule 5
    fn new() -> Self;                           // Rule 2
    fn consume(self);                           // Rule 1
    fn compare(&self, other: &Self) -> bool;    // Rule 6
    fn process<T>(&self, item: T);              // Rule 3
}

// ✅ Object-safe version
trait Good {
    fn id(&self) -> u32;                        // method instead of const
    fn speak(&self);                            // &self — pointer-shaped
    fn compare(&self, other: &dyn Good) -> bool;// &dyn instead of &Self
    fn process(&self, item: &dyn Display);      // &dyn instead of generic

    fn new() -> Self                            // excluded from vtable
    where
        Self: Sized;
}

fn main() {
    let xs: Vec<Box<dyn Good>> = vec![
        /* mix of any types implementing Good */
    ];
    for x in &xs {
        x.speak();
    }
}
```

# Composition over inheritance

This preference for composition is a deliberate design choice that Rust developers generally favor. It is particularly effective for modeling "has-a" relationships, where one type contains another as a component (for example, a car has an engine). Compared to inheritance (which models an "is-a" relationship), composition often leads to designs with less tight coupling between components.

## The core distinction

- **Inheritance ("is-a"):** A `Car` _is a_ `Vehicle`. The child class inherits all the parent's fields and methods automatically.
- **Composition ("has-a"):** A `Car` _has an_ `Engine`, _has_ `Wheels`, _has a_ `Transmission`. The car is built by combining smaller, independent pieces.

## Why Rust chose composition

Rust doesn't have class inheritance at all — no `extends`, no parent/child class hierarchies. Instead, you build complex types by:

1. **Embedding structs inside other structs** (composition)
2. **Sharing behavior via traits** (like interfaces)

```rust
struct Engine {
    horsepower: u32,
    fuel_type: FuelType,
}

impl Engine {
    fn ignite(&self) {
        println!("Engine ignited!");
    }
}

struct Car {
    engine: Engine,        // Car HAS-AN engine
    wheels: [Wheel; 4],    // Car HAS wheels
    make: String,
}

impl Car {
    fn start(&self) {
        self.engine.ignite();  // delegate to the component
    }
}
```

## Why "less tight coupling"?

### The problem with inheritance: the fragile base class

In an OOP language:

```
Vehicle
  └── MotorVehicle
        └── Car
              └── SportsCar
```

If you change `Vehicle`, every descendant can break — even ones you didn't know existed. Subclasses depend on the _internal implementation_ of their parents, not just their public interface. This is called **tight coupling**.

### How composition avoids this

With composition, `Car` only depends on `Engine`'s **public interface** — meaning the methods you can call from outside, not the internal fields or private logic.

```rust
struct Engine {
    // private internal state
    temperature: f32,
    rpm: u32,
    cylinders: u8,
}

impl Engine {
    // PUBLIC interface — what the outside world can use
    pub fn start(&self) {
        println!("Vroom!");
    }

    pub fn power_output(&self) -> u32 {
        200
    }
}

struct Car {
    engine: Engine,
}

impl Car {
    fn drive(&self) {
        self.engine.start();                     // uses public method
        let power = self.engine.power_output();  // uses public method
        println!("Driving with {} HP", power);
    }
}
```

Notice that `Car::drive` **never touches** `temperature`, `rpm`, or `cylinders`. It only calls `start()` and `power_output()`.

Tomorrow, you could rewrite `Engine` with completely different internals:

```rust
struct Engine {
    // COMPLETELY different internals
    fuel_injection_rate: f64,
    turbo_pressure: f32,
    // no more temperature, rpm, cylinders!
}

impl Engine {
    // Same public methods, different implementation
    pub fn start(&self) {
        println!("Whoooosh!");
    }

    pub fn power_output(&self) -> u32 {
        350
    }
}
```

**`Car` doesn't need to change at all.** It still calls `start()` and `power_output()`, which still exist. That's what "depends only on the public interface" means — `Car` is shielded from internal changes.

## Swapping components via a trait

A **trait** is like a contract: "any type that implements me promises to provide these methods."

```rust
trait PowerSource {
    fn start(&self);
    fn power_output(&self) -> u32;
}
```

Now multiple different types can all satisfy this contract:

```rust
struct Engine {
    cylinders: u8,
}

impl PowerSource for Engine {
    fn start(&self) {
        println!("Gasoline engine starting: Vroom!");
    }
    fn power_output(&self) -> u32 {
        200
    }
}

struct ElectricMotor {
    battery_kwh: f32,
}

impl PowerSource for ElectricMotor {
    fn start(&self) {
        println!("Electric motor humming silently");
    }
    fn power_output(&self) -> u32 {
        300
    }
}
```

`Engine` and `ElectricMotor` are **completely different types** with different fields, but both implement `PowerSource`, so both provide `start()` and `power_output()`.

## The generic Car

```rust
struct Car<P: PowerSource> {
    power: P,
    wheels: [Wheel; 4],
}
```

Read this as: **"A `Car` works with ANY type `P`, as long as `P` implements `PowerSource`."**

- `P` is a placeholder for a concrete type
- `P: PowerSource` is a constraint — it says "P must implement PowerSource"

### How do you know `PowerSource` is a trait and not a struct or enum?

You can't tell just from the name `PowerSource` alone — the syntax `P: PowerSource` is what tells you. In Rust, the `:` in a generic parameter **always** means a trait bound. If `PowerSource` were a struct or enum, the compiler would reject it:

```
error: expected trait, found struct `PowerSource`
```

The position in the syntax determines what it must be:

| Syntax                     | What it means                   | `PowerSource` must be |
| -------------------------- | ------------------------------- | --------------------- |
| `<P: PowerSource>`         | Trait bound on a generic type   | A **trait**           |
| `field: PowerSource`       | A field's concrete type         | A **struct or enum**  |
| `impl PowerSource for ...` | Implementing a trait for a type | A **trait**           |
| `fn foo(x: PowerSource)`   | A function parameter's type     | A **struct or enum**  |

So whenever you see `:` between a generic type parameter and a name inside angle brackets `< >`, what follows the `:` is guaranteed to be a trait.

Now you can create different kinds of cars:

```rust
let gas_car: Car<Engine> = Car {
    power: Engine { cylinders: 6 },
    wheels: [/* ... */],
};

let electric_car: Car<ElectricMotor> = Car {
    power: ElectricMotor { battery_kwh: 75.0 },
    wheels: [/* ... */],
};
```

Both are `Car`s, but one is powered by an `Engine` and the other by an `ElectricMotor`. The `Car` struct itself didn't need to be written twice.

```rust
trait PowerSource {
    fn start(&self);
    fn power_output(&self) -> u32;
}

struct Car<P: PowerSource> {
    power: P,
    wheels: [Wheel; 4],
}

impl<P: PowerSource> Car<P> {
    fn drive(&self) {
        self.power.start();  // works for BOTH Engine and ElectricMotor
        println!("Power: {}", self.power.power_output());
    }
}

struct Engine {
    cylinders: u8,
}

impl PowerSource for Engine {
    fn start(&self) {
        println!("Gasoline engine starting: Vroom!");
    }
    fn power_output(&self) -> u32 {
        200
    }
}

struct ElectricMotor {
    battery_kwh: f32,
}

impl PowerSource for ElectricMotor {
    fn start(&self) {
        println!("Electric motor humming silently");
    }
    fn power_output(&self) -> u32 {
        300
    }
}
let gas_car: Car<Engine> = Car {
    power: Engine { cylinders: 6 },
    wheels: [/* ... */],
};

let electric_car: Car<ElectricMotor> = Car {
    power: ElectricMotor { battery_kwh: 75.0 },
    wheels: [/* ... */],
};

gas_car.drive();
// Output: "Gasoline engine starting: Vroom!"
//         "Power: 200"

electric_car.drive();
// Output: "Electric motor humming silently"
//         "Power: 300"
```

The **same `drive` method** works for both, because `Car` doesn't care _which_ power source it has — only that it _has one that implements `PowerSource`_.

## The practical benefits

1. **Flexibility** — You can mix and match components. A truck and a car might share the same engine type without forcing them into a class hierarchy.
2. **Clearer ownership** — Rust's ownership model works naturally with composition. Each component has a well-defined lifetime and owner.
3. **Easier testing** — You can test `Engine` in isolation, then test `Car` using a mock engine.
4. **No diamond problem** — Inheritance in languages that allow multiple parents (like C++) leads to ambiguity when two parents define the same method. Composition sidesteps this entirely.
5. **Behavior via traits, not hierarchy** — Want your `Car` to be `Drivable`? Implement the `Drivable` trait. Want it to also be `Serializable`? Implement that trait too. No hierarchy needed, and any type can opt into any combination of behaviors.

## The mental model shift

Coming from OOP, you might instinctively reach for "what does this inherit from?" In Rust, ask instead:

- **What does it contain?** (fields / composition)
- **What can it do?** (traits / behavior)

This separation — data composition on one side, behavior via traits on the other — is what makes Rust designs tend to be more modular and easier to refactor than deep inheritance trees.

## Open for extension

If next year someone invents a `HydrogenFuelCell`, they just write:

```rust
struct HydrogenFuelCell { /* ... */ }

impl PowerSource for HydrogenFuelCell {
    fn start(&self) { /* ... */ }
    fn power_output(&self) -> u32 { /* ... */ }
}
```

And instantly: `Car<HydrogenFuelCell>` works. You never had to touch the `Car` code. **That's the flexibility composition gives you that inheritance can't.**

# Supertrait (`:`) vs `+` Trait Bounds

These two syntaxes both combine traits, but they serve different purposes:

## Supertrait — permanent constraint on the trait itself

```rust
trait PrintableSummary: Display {
    fn print_summary(&self);
}
```

This means: **any type that implements `PrintableSummary` must also implement `Display`**, always, for every type. It's baked into the trait's definition — there's no way around it.

The methods inside `PrintableSummary` can rely on `Display` being available (e.g., using `println!("{}", self)`).

## `+` bound — local constraint on a specific context

```rust
fn process_item<T: Debug + Clone>(item: T) { ... }
```

This means: **this particular function** requires `T` to implement both `Debug` and `Clone`. Another function could accept `T: Debug` alone — the traits themselves have no relationship to each other.

## Key differences

|                  | Supertrait (`:`)                    | `+` bound                                    |
| ---------------- | ----------------------------------- | -------------------------------------------- |
| **Where**        | Trait definition                    | Function / impl / where clause               |
| **Scope**        | Global, permanent                   | Local to that context                        |
| **Meaning**      | Trait A **always** requires trait B | This specific usage requires both A and B    |
| **Relationship** | Creates a hierarchy between traits  | No relationship — just combines requirements |

**Rule of thumb**: If every type implementing trait A should _always_ implement trait B too, use a supertrait. If you just need multiple capabilities for a specific function, use `+`.

# Associated Types

Sometimes a trait needs to refer to a type that isn't known until the trait is implemented. Associated types let the **implementor** choose that type, while enforcing that each struct can only choose **once**.

## The Problem: Why Not Just Use Generics on the Trait?

You _could_ put a generic parameter on the trait itself:

```rust
trait Iterator<T> {
    fn next(&mut self) -> Option<T>;
}
```

But this allows **multiple implementations** for the same struct:

```rust
struct Counter {
    current: u32,
    max: u32,
}

impl Iterator<i32> for Counter {
    fn next(&mut self) -> Option<i32> {
        if self.current < self.max {
            let val = self.current;
            self.current += 1;
            Some(val as i32)
        } else {
            None
        }
    }
}

impl Iterator<String> for Counter {
    fn next(&mut self) -> Option<String> {
        if self.current < self.max {
            let val = self.current;
            self.current += 1;
            Some(val.to_string())
        } else {
            None
        }
    }
}

fn main() {
    let mut counter = Counter { current: 0, max: 3 };

    counter.next(); // ❌ compiler error! Which next()? i32 or String?

    // You'd have to disambiguate every single call using turbofish syntax:
    let val_i32 = Iterator::<i32>::next(&mut counter);       // verbose and annoying
    let val_str = Iterator::<String>::next(&mut counter);     // verbose and annoying
}
```

> **Turbofish syntax (`::<T>`)** — The `::<i32>` above is called "turbofish". It tells the compiler
> which generic variant you mean. `Iterator::<i32>` is not accessing a field — it's selecting the
> `Iterator<i32>` version of the trait, then calling `::next` on it. You see turbofish in other
> places too:
>
> ```rust
> // Turbofish on a function — tell collect() what type to produce
> let nums: Vec<i32> = vec![1, 2, 3];
> let strings = nums.iter().map(|n| n.to_string()).collect::<Vec<String>>();
>
> // Turbofish on a generic function — tell parse() what type to return
> let n = "42".parse::<i32>().unwrap();
>
> // Turbofish on a struct — tell Vec which type it holds
> let v = Vec::<f64>::new();
> ```
>
> Turbofish is useful when the compiler can't infer the type on its own. But having to use it
> on **every call** (like with generic traits) is a sign that associated types would be a better fit.

For something like an iterator, this doesn't make sense. A `Counter` should produce **one kind** of item, not sometimes `i32` and sometimes `String`.

## The Solution: Associated Types

Associated types enforce a **one-to-one** relationship: each struct picks its type **once**, and the compiler always knows what it is.

```rust
pub trait SimpleIterator {
    type Item;  // associated type — implementor fills this in
    fn next(&mut self) -> Option<Self::Item>;
}
```

> **What does `Self::Item` mean?** — `Self` always refers to the **struct** that implements the
> trait, not the trait itself. So inside `impl SimpleIterator for Counter`, `Self` is `Counter` and
> `Self::Item` is `Counter::Item`, which is `u32`. Inside `impl SimpleIterator for Names`, `Self`
> is `Names` and `Self::Item` is `Names::Item`, which is `String`. Think of `Self::Item` as
> "the `Item` type that **this specific struct** chose."

Now each struct chooses its own `Item`, but can only choose once:

```rust
struct Counter {
    current: u32,
    max: u32,
}

impl SimpleIterator for Counter {
    type Item = u32;  // Counter produces u32 — locked in

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.max {
            let val = self.current;
            self.current += 1;
            Some(val)
        } else {
            None
        }
    }
}
```

> **Why can't you also set `type Item = String` for `Counter`?** — Because `SimpleIterator` has
> no generic parameter, so there's only **one trait** and therefore only **one impl** allowed per struct.
> The compiler sees two `impl SimpleIterator for Counter` blocks and rejects it with a
> "conflicting implementations" error — regardless of what `Item` is set to.
>
> Compare with the generic version: `Iterator<i32>` and `Iterator<String>` are **different traits**
> (the generic parameter creates separate versions), so two impl blocks are allowed. Associated
> types keep it as **one trait**, so only one impl is possible. That's exactly the constraint we want.

Calling it requires no extra annotation — the compiler knows `Item` is `u32`:

```rust
fn main() {
    let mut counter = Counter { current: 0, max: 3 };
    println!("{:?}", counter.next()); // Some(0) — compiler knows it's Option<u32>
    println!("{:?}", counter.next()); // Some(1)
    println!("{:?}", counter.next()); // Some(2)
    println!("{:?}", counter.next()); // None
}
```

## Different Structs Can Choose Different Types

The **trait** is not limited to one type. Each **struct** picks its own `Item`:

```rust
struct Names {
    data: Vec<String>,
    index: usize,
}

impl SimpleIterator for Names {
    type Item = String;  // Names produces String

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            let val = self.data[self.index].clone();
            self.index += 1;
            Some(val)
        } else {
            None
        }
    }
}
```

- `Counter` iterates over `u32`
- `Names` iterates over `String`
- Each is locked to its own type — no ambiguity

## What You Cannot Do

You **cannot** implement the same trait twice for the same struct with a different associated type:

```rust
impl SimpleIterator for Counter {
    type Item = u32;
}

impl SimpleIterator for Counter {  // ❌ compiler error!
    type Item = String;            // Counter already implemented SimpleIterator
}
```

## Real-World Use Cases of Associated Types

Associated types are used throughout Rust's standard library. Here are the most important ones with full explanations.

### 1. `Iterator` — what items does it produce?

This is the most common example. Every iterator produces items of one specific type. A range of numbers produces `i32`, a `.chars()` iterator produces `char`, etc. The associated type `Item` locks this in:

```rust
trait Iterator {
    type Item;  // each iterator produces one kind of item
    fn next(&mut self) -> Option<Self::Item>;
}

struct Countdown {
    value: u32,
}

impl Iterator for Countdown {
    type Item = u32;  // Countdown always produces u32

    fn next(&mut self) -> Option<u32> {
        if self.value > 0 {
            self.value -= 1;
            Some(self.value)
        } else {
            None
        }
    }
}

fn main() {
    let mut c = Countdown { value: 3 };
    println!("{:?}", c.next()); // Some(2)
    println!("{:?}", c.next()); // Some(1)
    println!("{:?}", c.next()); // Some(0)
    println!("{:?}", c.next()); // None
}
```

Why associated type? Because a `Countdown` should always produce `u32`. It wouldn't make sense for the same iterator to sometimes give you `u32` and sometimes `String`.

### But wait — does `Vec<u32>`, `Vec<String>`, etc. each need a separate implementation?

No! This is where **generics and associated types work together**. The standard library implements `IntoIterator` **once** for all `Vec<T>`:

```rust
// One implementation covers ALL Vec types
impl<T> IntoIterator for Vec<T> {
    type Item = T;                // whatever T the Vec holds
    type IntoIter = IntoIter<T>;  // the iterator struct (see below)

    fn into_iter(self) -> IntoIter<T> {
        // ... returns an iterator over the Vec's elements
    }
}
```

> **What is `IntoIter<T>`?** — It's a **struct** defined in the standard library (`std::vec::IntoIter`)
> that does the actual iterating. When you call `.into_iter()` on a `Vec`, it doesn't iterate
> by itself — it creates an `IntoIter` struct that tracks the position and gives you elements
> one by one:
>
> ```rust
> // Simplified version of what the standard library defines:
> pub struct IntoIter<T> {
>     // internal fields that track position in the Vec's data
>     buf: *const T,
>     len: usize,
>     // ...
> }
>
> impl<T> Iterator for IntoIter<T> {
>     type Item = T;
>     fn next(&mut self) -> Option<T> {
>         // move the next element out and advance position
>     }
> }
> ```
>
> So `IntoIterator` has **two** associated types:
>
> - `Item = T` — the type of each element you get
> - `IntoIter = IntoIter<T>` — the struct that implements `Iterator` and tracks progress
>
> This is why `for x in vec` works — Rust calls `vec.into_iter()` to get the `IntoIter` struct,
> then calls `.next()` on it repeatedly until it returns `None`.

This single `impl<T>` block automatically works for every possible `Vec`:

```rust
fn main() {
    // Vec<u32> → Item is u32
    let numbers = vec![1u32, 2, 3];
    for n in numbers {           // n is u32
        println!("{n}");
    }

    // Vec<String> → Item is String
    let names = vec![String::from("Alice"), String::from("Bob")];
    for name in names {          // name is String
        println!("{name}");
    }

    // Vec<bool> → Item is bool
    let flags = vec![true, false, true];
    for flag in flags {          // flag is bool
        println!("{flag}");
    }
}
```

No separate implementation for each type — the generic `<T>` handles that. But the associated type still does its job: for any **specific** `Vec`, the `Item` is locked. A `Vec<u32>` always gives you `u32`, never `String`.

Think of it as two levels working together:

| Level                          | What it does                                           | Example                               |
| ------------------------------ | ------------------------------------------------------ | ------------------------------------- |
| **Generic `<T>`**              | One implementation works for **all** `Vec` types       | `impl<T> IntoIterator for Vec<T>`     |
| **Associated type `Item = T`** | For each **specific** `Vec<T>`, the item type is fixed | `Vec<u32>` always iterates over `u32` |

Generics give you flexibility at the **definition** level. Associated types give you certainty at the **usage** level.

### 2. `FromStr` — what error can parsing return?

When you parse a string into a type (like `"42".parse::<i32>()`), the parsing might fail. But **different types fail with different errors**. Parsing `"hello"` as `i32` gives a `ParseIntError`, parsing it as `bool` gives a `ParseBoolError`. The associated type `Err` lets each type specify its own error:

```rust
trait FromStr {
    type Err;  // each type has its own parsing error
    fn from_str(s: &str) -> Result<Self, Self::Err>;
}

struct Percentage {
    value: u8,
}

// Define our own error type
struct PercentageError;

impl FromStr for Percentage {
    type Err = PercentageError;  // parsing a Percentage can fail with PercentageError

    fn from_str(s: &str) -> Result<Self, PercentageError> {
        let num: u8 = s.parse().map_err(|_| PercentageError)?;
        if num <= 100 {
            Ok(Percentage { value: num })
        } else {
            Err(PercentageError)
        }
    }
}

fn main() {
    let p: Result<Percentage, _> = "85".parse();   // Ok — valid percentage
    let p2: Result<Percentage, _> = "200".parse();  // Err — PercentageError
}
```

Why associated type? Because `Percentage` always fails with `PercentageError`. It wouldn't make sense for `Percentage::from_str` to sometimes return `ParseIntError` and sometimes `PercentageError`.

### 3. `Deref` — what does a smart pointer dereference to?

When you use `*my_box` or access methods through a `Box`, `Rc`, or `String`, Rust calls `Deref` behind the scenes. The associated type `Target` tells Rust what type is **inside** the wrapper:

```rust
trait Deref {
    type Target;  // what you get when you dereference
    fn deref(&self) -> &Self::Target;
}
```

The standard library implements `Deref` for wrapper types like `String`, `Box`, and `Vec`:

```rust
// String always derefs to str
impl Deref for String {
    type Target = str;

    fn deref(&self) -> &str {
        // returns a reference to the inner str data
    }
}

// Box<T> always derefs to T (uses generics + associated type together)
impl<T> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &T { ... }
}

// Vec<T> always derefs to [T] (a slice)
impl<T> Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] { ... }
}
```

Rust uses these `Deref` implementations automatically through **deref coercion** — when you pass a `&String` where `&str` is expected, Rust calls `deref()` behind the scenes:

```rust
fn greet(name: &str) {
    println!("Hello, {name}!");
}

fn takes_slice(s: &[i32]) {
    println!("Got {} items", s.len());
}

fn main() {
    let s = String::from("Alice");
    greet(&s);              // &String → &str via Deref, automatically

    let v = vec![1, 2, 3];
    takes_slice(&v);        // &Vec<i32> → &[i32] via Deref, automatically

    let b = Box::new(42);
    let n: i32 = *b;        // Box<i32> → i32 via Deref
}
```

Why associated type? Because `String` always derefs to `str`, never to something else. `Box<i32>` always derefs to `i32`. Each wrapper has exactly **one** inner type.

### 4. `Add` — what type does addition produce?

When you use the `+` operator, Rust calls the `Add` trait. The associated type `Output` tells Rust what type the result is:

```rust
use std::ops::Add;

// Rhs: Right-Hand side
// left + right
// Self   Rhs
trait Add<Rhs = Self> {
    type Output;  // what + produces
    fn add(self, rhs: Rhs) -> Self::Output;
}
```

> **What is `Rhs`?** — `Rhs` stands for **Right-Hand Side**. In `a + b`, `a` is `Self` (the left
> side) and `b` is `Rhs` (the right side).
>
> `Rhs = Self` is a **default type parameter** — if you don't specify `Rhs`, it defaults to the
> same type as `Self`. So `impl Add for i32` is the same as `impl Add<i32> for i32`, meaning
> `i32 + i32`:
>
> ```rust
> impl Add for i32 {
> //   Add<Rhs = Self> → Add<i32> since Self is i32
>     type Output = i32;
>
>     fn add(self, rhs: i32) -> i32 {
>         // self is the left i32, rhs is the right i32
>     }
> }
> ```
>
> But you can specify a **different** `Rhs` to add different types together, like
> `Meters + Centimeters` below — where `Self` is `Meters` and `Rhs` is `Centimeters`.
>
> **But where is `Self` in `trait Add<Rhs = Self>`?** — In the trait definition, `Self` doesn't
> refer to any concrete type yet. It's a placeholder meaning "whatever type will implement this
> trait later." The trait doesn't know who `Self` is — it only gets filled in at each `impl`:
>
> ```rust
> impl Add for i32 {
> //   Self = i32, so Rhs defaults to i32
> }
>
> impl Add for f64 {
> //   Self = f64, so Rhs defaults to f64
> }
>
> impl Add<Centimeters> for Meters {
> //   Self = Meters, Rhs = Centimeters (overriding the default)
> }
> ```
>
> So `Rhs = Self` is a **rule**, not a value: "by default, `Rhs` will be the same type as
> whoever implements this trait."

For example, adding two `i32` values gives an `i32`. But you could define a type where adding produces something different:

```rust
use std::ops::Add;

struct Meters(f64);
struct Centimeters(f64);

// Adding Centimeters to Meters gives Meters
// Self = Meters (left side), Rhs = Centimeters (right side)
impl Add<Centimeters> for Meters {
    type Output = Meters;  // result is always Meters

    fn add(self, rhs: Centimeters) -> Meters {
        Meters(self.0 + rhs.0 / 100.0)
    }
}

fn main() {
    let total = Meters(1.5) + Centimeters(50.0);
    // total is Meters(2.0)
}
```

Why associated type? Because `Meters + Centimeters` always produces `Meters`. The result type is fixed by the implementation, not chosen by the caller.

> **Note:** `Add` uses **both** a generic parameter (`Rhs`) and an associated type (`Output`).
> The generic `Rhs` is the **input** — you can add different types to `Meters` (like `Centimeters`,
> `Millimeters`, etc.). The associated `Output` is the **result** — for each combination, the
> result type is fixed. This is a common pattern: generics for inputs, associated types for outputs.

> **Know more: How does `Add` work for `i32`, `f64`, etc.?** — The standard library implements
> `Add` **separately** for each numeric type. There's no generic magic — each type has its own
> `impl`:
>
> ```rust
> impl Add for i32 {
>     type Output = i32;
>     fn add(self, rhs: i32) -> i32 { /* uses CPU integer addition */ }
> }
>
> impl Add for f64 {
>     type Output = f64;
>     fn add(self, rhs: f64) -> f64 { /* uses CPU floating-point addition */ }
> }
>
> // ... and so on for i8, i16, i64, i128, u8, u16, u32, u64, u128, f32
> ```
>
> To avoid writing all these by hand, the standard library uses a **macro** to generate them:
>
> ```rust
> macro_rules! impl_add {
>     ($($t:ty)*) => {
>         $(
>             impl Add for $t {
>                 type Output = $t;
>                 fn add(self, rhs: $t) -> $t {
>                     self + rhs
>                 }
>             }
>         )*
>     }
> }
>
> // One line generates impl blocks for ALL numeric types:
> impl_add! { i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 }
> ```
>
> This is different from `Vec<T>` where one `impl<T>` covers all types. Addition needs different
> CPU instructions for integers vs floats, so each type genuinely needs its own implementation.
> The macro just saves the repetitive typing.

### 5. `Future` — what does an async computation return?

Every `async` function in Rust returns a `Future`. The associated type `Output` says what value the future eventually produces when it completes:

```rust
trait Future {
    type Output;  // what you get when the future completes
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output>;
}
```

When you write:

```rust
async fn fetch_name() -> String {
    // ... some async work
    String::from("Alice")
}
```

The compiler creates a type that implements `Future` with `type Output = String`. When you `.await` it, you get a `String`.

Why associated type? Because a specific async computation always produces one type. `fetch_name()` always resolves to a `String`, never to something else.

## Associated Types vs Generic Parameters

|                     | Associated Type (`type Item`)         | Generic Parameter (`Trait<T>`)            |
| ------------------- | ------------------------------------- | ----------------------------------------- |
| **Implementations** | One per struct                        | Multiple per struct (one per `T`)         |
| **Call site**       | No annotation needed                  | May need turbofish (`::<T>`)              |
| **Use when**        | One-to-one: struct produces one type  | One-to-many: struct works with many types |
| **Example**         | `Iterator` (one item type per struct) | `From<T>` (convert from many sources)     |
