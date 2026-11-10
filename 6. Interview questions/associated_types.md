## Associated types

You meet associated types in `Deref`:

```rust
pub trait Deref {
    type Target;
    fn deref(&self) -> &Self::Target;
}
```

This note starts with a tiny example, builds up step by step, and comes back to `Deref` at the end.
See [deref_coercion.md](deref_coercion.md) for `Deref` itself.

---

## Part 1: The basic idea

### 1. The problem: who chooses the return type?

Without an associated type, the trait has to write the return type itself:

```rust
trait Container {
    fn get(&self) -> &u32;   // the type is fixed here
}
```

Now **every** implementor must return `&u32`:

```rust
impl Container for Wallet { fn get(&self) -> &u32 { /* ... */ } }    // ✅
impl Container for Book   { fn get(&self) -> &String { /* ... */ } } // ❌ error
```

The **author of the trait** chose the type, and implementors have no say.

### 2. The solution: let each implementor choose

```rust
trait Container {
    type Item;                      // an empty slot: "Item = ______"
    fn get(&self) -> &Self::Item;
}
```

Now the trait says: *"I don't know the type. **You**, the one implementing me, tell me what it is."*

Think of a trait as an **empty form**, and of `impl` as a type **filling in that form**:

```rust
struct Wallet {
    money: u32,
}

impl Container for Wallet {
    type Item = u32;                // Wallet writes "u32" in the slot
    fn get(&self) -> &Self::Item {  // here, Self::Item is u32
        &self.money
    }
}
```

Each implementor can choose a different type, but once chosen, it is **fixed for that type**:

```rust
impl Container for Wallet { type Item = u32;    /* ... */ }   // ✅
impl Container for Book   { type Item = String; /* ... */ }   // ✅
```

| Approach            | Who chooses the return type?               |
|---------------------|--------------------------------------------|
| No `type Item`      | The **trait author**, one type for everyone |
| With `type Item`    | Each **implementor**, one type for itself   |

### 3. What `Self` means

- `Container` is the **trait** (the form).
- `Self` is the **type that implements the trait** (whoever fills in the form).

Inside the trait definition, we don't know yet who `Self` is. It means "whoever implements me later".
Inside `impl Container for Wallet`, `Self` is `Wallet`.

So the trait is saying: *"Whoever implements `Container` must tell me what `Item` is."*

### 4. Why `=` and not `:`?

The two symbols mean different things in Rust:

| Symbol | Meaning                                                    | Example                        |
|--------|------------------------------------------------------------|--------------------------------|
| `:`    | "the type of this thing is..." or "must implement..."      | `let x: i32`, `T: Display`     |
| `=`    | "this type **is exactly**..."                              | `type Km = i32;`               |

`Item` is itself a **type**, not a value. We don't want to say "Item has the type u32". We want to say "Item **is** u32", just like a type alias:

```rust
type Km = i32;    // Km is another name for i32
type Item = u32;  // (for Wallet) Item is another name for u32
```

`:` can be used with associated types too, but it means a **constraint**, not a choice:

```rust
use std::fmt::Display;

trait Container {
    type Item: Display;    // constraint: whatever you choose must implement Display
    fn get(&self) -> &Self::Item;
}

impl Container for Wallet {
    type Item = u32;       // choice: Item is u32 (u32 implements Display ✅)
    // ...
}
```

So you write a constraint with `:` in the **trait** and you make a choice with `=` in the **impl**.

### 5. `Item` is defined in the `impl`, not in the struct

```rust
struct Wallet { money: u32 }       // no `Item` here

impl Container for Wallet {
    type Item = u32;                // `Item` is created here
    // ...
}
```

The struct itself has no `Item`. `Item` belongs to the **relationship between `Wallet` and `Container`**, which is the `impl` block.

This also means the struct does **not** have to be generic. An associated type is not a generic parameter:

```rust
impl Deref for String {
    type Target = str;   // String is not generic
    // ...
}

impl<T> Deref for Box<T> {
    type Target = T;     // Box is generic, Target is its parameter
    // ...
}
```

---

## Part 2: How `Self::Item` is read

### 6. `::` is a lookup

`::` is a **namespace lookup**. `Self::Item` means:
> "Go into the type `Self` and find the type named `Item`."

`Item` is a **name** (a label). `u32` is **what is stored under that name**.
The lookup replaces the **whole path** with what it finds:

```
Self::Item   →   u32         ✅ the whole path becomes u32
Self::Item   →   Self::u32   ❌ this never happens
```

There is no `Self::u32` at any step. It works the same way as reading a struct field:

```rust
let wallet = Wallet { money: 5 };
wallet.money   // this is 5, not `wallet.5`
```

| Lookup         | Name    | What you get back |
|----------------|---------|-------------------|
| `wallet.money` | `money` | the value `5`     |
| `Self::Item`   | `Item`  | the type `u32`    |

In both cases you ask for a name and get back what is stored there. The path is gone after the lookup.

The common mistake is to replace only `Item` with `u32` and keep the `Self::` part.
But `Self::` is part of the **question** ("where should I look?"), not part of the **answer**.

### 7. Step by step, inside `impl Container for Wallet`

1. `Self` is `Wallet`.
2. So `Self::Item` is `<Wallet as Container>::Item`.
3. Look at what `Wallet` wrote for `Item` in that impl: `u32`.
4. So `&Self::Item` is `&u32`.

The compiler ends up seeing this:

```rust
fn get(&self) -> &u32 { /* ... */ }
```

### 8. `Self::Item` works like `Wallet::new()`

Everything defined inside an `impl` block is called an **associated item**, and you reach all of them with `::`:

```rust
impl Wallet {
    const MAX: u32 = 100;          // associated const
    fn new() -> Self {             // associated function
        Wallet { money: 0 }
    }
}

impl Container for Wallet {
    type Item = u32;               // associated type
    // ...
}
```

| Inside the `impl`        | Kind                 | How you reach it | What you get   |
|--------------------------|----------------------|------------------|----------------|
| `fn new() -> Self`       | associated function  | `Wallet::new`    | a function     |
| `const MAX: u32 = 100;`  | associated const     | `Wallet::MAX`    | the value `100`|
| `type Item = u32;`       | associated type      | `Self::Item`     | the type `u32` |

So `Self::Item` does the same thing as `Wallet::new`: it finds a name inside `Wallet`.
The only difference is that `new` leads to a function, and `Item` leads to a type.

---

## Part 3: Using associated types outside the `impl`

### 9. `Wallet::Item` does not compile

`Wallet::new()` works everywhere, but `Wallet::Item` outside the `impl` is an error:

```rust
Wallet::new()                  // ✅
Wallet::Item                   // ❌ error[E0223]: ambiguous associated type
<Wallet as Container>::Item    // ✅
```

The reason is **who owns the item**:

- `new` is defined in `impl Wallet`, so it belongs to `Wallet` itself.
- `Item` is defined in `impl Container for Wallet`, so it belongs to **`Wallet`'s implementation of `Container`**.

Outside the `impl`, you have to say which trait you mean.
Inside the `impl`, `Self::Item` works, because the compiler already knows which `impl` you are in.

### 10. Fully qualified syntax: `<Type as Trait>::Item`

`<Wallet as Container>::Item` reads as:
> "Look at the `Container` form that `Wallet` filled in. What did it write for `Item`?" → `u32`

- `Wallet` is who filled in the form
- `as Container` is which form
- `::Item` is which slot

Naming the trait matters, because a type can implement many traits, and two traits can have a slot with the **same name**:

```rust
trait Describe {
    type Item;
    fn describe(&self) -> Self::Item;
}

impl Describe for Wallet {
    type Item = String;   // Describe's Item is String
    fn describe(&self) -> Self::Item {
        format!("{} coins", self.money)
    }
}
```

Now `Wallet` has two different `Item`s:

- `<Wallet as Container>::Item` is `u32`
- `<Wallet as Describe>::Item` is `String`

### 11. When can you use the short form?

| Where                                      | Short form works?                          |
|--------------------------------------------|--------------------------------------------|
| Inside the trait or the `impl` block       | ✅ `Self::Item`                            |
| On a generic with a bound (`T: Container`) | ✅ `T::Item`                               |
| On a concrete type (`Wallet`)              | ❌ `Wallet::Item` is error E0223. Write `<Wallet as Container>::Item` |

### 12. Where do you actually use `<T as Trait>::Item`?

It is a **type**, so you can write it anywhere Rust expects a type, just like `i32`:

```rust
// As the type of a variable
let a: &<Wallet as Container>::Item = wallet.get();      // a: &u32
let b: <Wallet as Describe>::Item = wallet.describe();   // b: String

// As the return type of a generic function (the main use case)
fn show<T: Container>(c: &T) -> &T::Item {
    c.get()
}
```

In `show` we don't know what `T` is, so we don't know what `get` returns. We say: "whatever `T` chose as `Item` when it implemented `Container`".

### 13. This is not the turbofish

| Syntax                         | Who chooses the type?                 | Direction             |
|--------------------------------|---------------------------------------|-----------------------|
| `"5".parse::<i32>()`           | **You** (the caller) choose it        | You **pass** a type in (generic) |
| `<T as Container>::Item`       | The **implementor** already chose it  | You **read** a type out (associated type) |

Note: the turbofish is written `parse::<i32>()`, not `parse()::<i32>()`.

---

## Part 4: Back to `Deref`

### 14. How to read `fn deref(&self) -> &Self::Target;`

`Deref` works exactly like `Container`. `Target` plays the role of `Item`.

| Piece           | Meaning                                                              |
|-----------------|----------------------------------------------------------------------|
| `fn deref`      | Any type that implements `Deref` must write this method              |
| `&self`         | It is called on an instance and **borrows** it (no ownership taken)  |
| `-> &`          | It returns a **reference**, not a value                              |
| `Self::Target`  | The type the **implementor chose** as `Target` in its `impl` block. Short for `<Self as Deref>::Target` |

In one sentence:
> `deref` returns a reference to whatever type this type chose as its `Target` when it implemented `Deref`.

For `String`, `Target = str`, so `deref` returns `&str`.
For `Box<T>`, `Target = T`, so `deref` returns `&T`.

### 15. Why not a generic `Deref<T>`?

You might ask: why not write the trait with a generic parameter instead?

```rust
pub trait Deref<T> {
    fn deref(&self) -> &T;
}
```

#### A generic trait can be implemented many times

`Container` can be written both ways, and both compile:

```rust
// With a generic parameter
trait Container<T> {
    fn get(&self) -> &T;
}

struct Wallet { money: u32, owner: String }

impl Container<u32> for Wallet {
    fn get(&self) -> &u32 { &self.money }
}

impl Container<String> for Wallet {     // ✅ a second impl is allowed
    fn get(&self) -> &String { &self.owner }
}
```

`Container<u32>` and `Container<String>` count as **two different traits**, so `Wallet` can implement both.
The price is that the compiler can't guess which one you mean:

```rust
let m: &u32 = wallet.get();                      // ✅ the annotation picks Container<u32>
let x = <Wallet as Container<u32>>::get(&wallet); // ✅ fully qualified syntax picks it
let y = wallet.get();                            // ❌ error[E0283]: type annotations needed
                                                 //    multiple `impl`s satisfying `Wallet: Container<_>` found
```

With an associated type, a second impl is not allowed at all:

```rust
impl Container for Wallet { type Item = u32;    /* ... */ }
impl Container for Wallet { type Item = String; /* ... */ }
// ❌ error[E0119]: conflicting implementations of trait `Container` for type `Wallet`
```

So `wallet.get()` always has exactly one answer.

#### Why this matters for `Deref`

If `Deref` were generic, `String` could implement both `Deref<str>` and `Deref<[u8]>`.
Then `*s` or `s.len()` would have two possible meanings, and you would need a type annotation every time.

That would break **deref coercion**, whose whole point is that the compiler turns `&String` into `&str` **by itself**.
With `type Target`, each type implements `Deref` only once, so it has only one `Target`, and `*x` always has exactly one meaning.

A smaller benefit: bounds stay short. You write `T: Deref`, not `T: Deref<U>` with an extra `U` parameter on the function.

#### Rule of thumb

Ask: *"Does it make sense for one type to implement this trait for several different types?"*

| Choice                 | Impls per type                | Example |
|------------------------|-------------------------------|---------|
| Generic `Trait<T>`     | **Many**, one per `T`         | `From<T>`: `String` has both `From<&str>` and `From<char>` |
| Associated `type X`    | **Only one**                  | `Deref::Target`, `Iterator::Item` |

- `From`: yes. A `String` can be built from many different things, so it is generic.
- `Deref`: no. `*x` must have only one answer, so it uses an associated type.
- `Container`: it depends on what you mean. If a container holds one kind of thing, use an associated type. If it should hand out several kinds of things, use a generic.

Compare this with section 1. There are three ways to decide the return type:

| Approach                   | Who chooses the type?      | How many per implementor? |
|----------------------------|----------------------------|---------------------------|
| Fixed type (`-> &u32`)     | The trait author           | One, the same for everyone |
| Associated type            | The implementor            | One                       |
| Generic parameter          | The implementor, per impl  | Many                      |

---

## Part 5: Other ways to use an associated type

### 16. `-> &Self::Target` is not the only pattern

In `Deref`, the associated type is the output, behind a reference.
But inside a trait, `Self::Item` is a normal type, so it can go **anywhere a type can go**.
Here are common patterns from the standard library.

#### 1. Wrapped in another type

```rust
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;   // inside Option
}
```

#### 2. As the error type

```rust
trait FromStr: Sized {
    type Err;
    fn from_str(s: &str) -> Result<Self, Self::Err>;   // only the error part
}
```

`"5".parse::<i32>()` uses this trait. For `i32`, `Err = ParseIntError`.

#### 3. Returned by value, not by reference

```rust
trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}
```

#### 4. As an input parameter

```rust
trait Storage {
    type Item;
    fn put(&mut self, item: Self::Item);   // the input, not the output
}

struct Shelf { items: Vec<u32> }

impl Storage for Shelf {
    type Item = u32;
    fn put(&mut self, item: u32) {
        self.items.push(item);
    }
}
```

#### 5. Several associated types, with bounds, linked to each other

```rust
trait IntoIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;   // a bound that refers to the other one
    fn into_iter(self) -> Self::IntoIter;
}
```

`IntoIter` must be an iterator, and the items it yields must be the same `Item`.

#### 6. Reused by another trait

```rust
trait DerefMut: Deref {
    fn deref_mut(&mut self) -> &mut Self::Target;   // Target comes from Deref
}
```

`DerefMut` has no `type Target` of its own. Because `DerefMut: Deref`, every `DerefMut` type is also a `Deref` type, so `Self::Target` already exists.

| Pattern                   | Example                      | Where `Self::X` appears        |
|---------------------------|------------------------------|--------------------------------|
| Behind a reference        | `Deref::deref`               | `-> &Self::Target`             |
| Wrapped in another type   | `Iterator::next`             | `-> Option<Self::Item>`        |
| As the error type         | `FromStr::from_str`          | `-> Result<Self, Self::Err>`   |
| By value                  | `Add::add`                   | `-> Self::Output`              |
| As an input               | `Storage::put`               | `item: Self::Item`             |
| In a bound                | `IntoIterator`               | `type IntoIter: Iterator<Item = Self::Item>` |
| From a supertrait         | `DerefMut::deref_mut`        | `-> &mut Self::Target`         |

### 17. The real `Deref` definition: `?Sized`

The `Deref` at the top of this note is simplified. The real one has a bound on `Target`:

```rust
pub trait Deref {
    type Target: ?Sized;   // allows unsized types like str and [T]
    fn deref(&self) -> &Self::Target;
}
```

By default, every type parameter and associated type must be `Sized`, which means its size is known at compile time.
`str` and `[T]` have no fixed size, so without `?Sized`, `String` could not choose `Target = str`:

```rust
trait MyDeref {
    type Target;            // no ?Sized
    fn my_deref(&self) -> &Self::Target;
}

impl MyDeref for String {
    type Target = str;      // ❌ error[E0277]: the size for values of type `str`
                            //    cannot be known at compilation time
    // ...
}
```

`?Sized` means "this type **may or may not** be `Sized`". It removes the default requirement.

This is also a reason `deref` returns a reference: you can't return an unsized `str` by value, but you can return `&str`, because a reference always has a known size.
