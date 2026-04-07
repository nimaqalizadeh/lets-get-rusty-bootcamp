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

Inside the loop, different elements of the `Vec` might hold a `Dog` (0 bytes), a `Cat` (0 bytes), or an `Elephant` (500 bytes). They have **different sizes**, so the `Box` can't store them inline — it must store them indirectly and carry the vtable so it can figure out what to do with them at runtime. The data pointer isn't there because `Dog`'s size is unknown; it's there because `dyn Animal` is a type that *refuses to commit* to any one concrete type.

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

When you write `Box::new(Dog)` and coerce it into `Box<dyn Animal>`, the fat pointer gets stamped with the address of the `<Dog as Animal>` vtable. `Box::new(Cat)` gets stamped with `<Cat as Animal>`'s vtable, and so on. Each fat pointer carries its *own* vtable address:

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

It blindly follows whichever vtable pointer *that specific element* carries. The first element follows `Dog`'s vtable and ends up calling `Dog::speak`; the second follows `Cat`'s and calls `Cat::speak`. The loop body has no idea which is which — **the vtable pointer itself is the selector**. That's the whole trick of dynamic dispatch: every trait object carries, as runtime data, the answer to "which implementation should I use?"

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
