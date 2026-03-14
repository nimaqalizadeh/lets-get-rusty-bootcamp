# Concrete Lifetimes

One of the memory safety gurantees of rust is that there are no dangling references. The borrow checker enforces no-dangling-references by tracking lifetimes — the scope during which a reference is valid. Specifically, it enforces one rule:

A reference must not outlive the data it points to.

Outlive means the reference lives (is in scope) longer than the data it points to.

Lifetimes become relevant in two situations:
- **Moving** — transferring ownership moves the value to a new location, ending the lifetime at the old location. Any place a value is bound to a variable involves a move — function parameters and loop variables are just syntactic sugar for `let` bindings:
  - **`let` bindings** — `let y = x` moves ownership from x's location to y's location
    ```rust
    let x = String::from("hello"); // lifetime at x starts
    let y = x;                     // lifetime at x ENDS, lifetime at y starts
    println!("{x}");               // ERROR: x is uninitialized
    ```
  - **Function parameters** — `fn foo(s: String)` is like `let s = ...`; ownership moves into the parameter
    ```rust
    fn print(s: String) { println!("{s}"); } // s's lifetime starts here, ends when function returns
    let name = String::from("Alice");
    print(name);          // lifetime at name ENDS — moved into parameter
    println!("{name}");   // ERROR: name is uninitialized
    ```
  - **`for` loop variables** — `for i in collection` is like `let i = ...` on each iteration; ownership moves into `i`
    ```rust
    let names = vec![String::from("a"), String::from("b")];
    for name in names {   // each element moves into `name`; names is moved into the loop
        println!("{name}");
    }
    println!("{names:?}"); // ERROR: names was moved into the for loop
    ```
  - **`if let` bindings** — `if let Some(x) = value` moves the value into `x`
    ```rust
    let s = Some(String::from("hello"));
    if let Some(x) = s {  // s is moved into x
        println!("{x}");
    }
    println!("{s:?}");    // ERROR: s was moved
    ```
  - **`while let` bindings** — `while let Some(x) = iter.next()` moves the value into `x` each iteration
    ```rust
    let strings = vec![String::from("a"), String::from("b")];
    let mut iter = strings.into_iter(); // strings moved into iter
    while let Some(s) = iter.next() {   // each String moves into `s`
        println!("{s}");
    }
    println!("{strings:?}"); // ERROR: strings was moved into iter
    ```
  - **`match` arms** — binding by value in a match arm moves the value into the bound variable
    ```rust
    let s = Some(String::from("hello"));
    match s {
        Some(x) => println!("{x}"), // s is moved into x
        None => {}
    }
    println!("{s:?}"); // ERROR: s was moved
    ```
  - **Closure captures** — a `move` closure takes ownership of captured variables, moving them into the closure
    ```rust
    let name = String::from("Alice");
    let greet = move || println!("{name}"); // name moves into the closure
    println!("{name}");                     // ERROR: name was moved
    greet();
    ```
  - **Struct field assignments** — assigning a variable to a struct field moves ownership into that field
    ```rust
    let name = String::from("Alice");
    let user = User { name: name }; // name moves into user.name
    println!("{name}");             // ERROR: name was moved into the struct field
    ```
  - **Enum variant construction** — wrapping a value in an enum variant moves ownership into that variant
    ```rust
    let name = String::from("Alice");
    let opt = Some(name);  // name moves into the Some variant
    println!("{name}");    // ERROR: name was moved into the enum variant
    ```
  - **Tuple construction** — placing owned values into a tuple moves each one into the tuple
    ```rust
    let name = String::from("Alice");
    let age = String::from("30");
    let t = (name, age);   // name and age both move into the tuple
    println!("{name}");    // ERROR: name was moved into the tuple
    ```
  - **Array/Vec construction** — placing owned values into an array or vec moves each element in
    ```rust
    let s1 = String::from("a");
    let s2 = String::from("b");
    let arr = [s1, s2];    // s1 and s2 move into the array
    println!("{s1}");      // ERROR: s1 was moved into the array
    ```
  - **Methods consuming `self`** — calling a method that takes `self` (not `&self`) moves the value into the method
    ```rust
    let name = String::from("Alice");
    let upper = name.to_uppercase(); // to_uppercase takes self — name is moved
    println!("{name}");              // ERROR: name was moved into to_uppercase
    ```
  - **Destructuring** — binding fields or elements by value in a destructuring pattern moves each one out of the container
    ```rust
    let t = (String::from("Alice"), String::from("30"));
    let (name, age) = t;   // name and age move out of the tuple — t is consumed
    println!("{name}");    // fine
    println!("{t:?}");     // ERROR: t was moved

    struct Point { x: String, y: String }
    let p = Point { x: String::from("1"), y: String::from("2") };
    let Point { x, y } = p; // x and y move out of p — p is consumed
    println!("{x}");        // fine
    println!("{p:?}");      // ERROR: p was moved
    ```
  - **Return values** — returning an owned value from a function moves it to the caller's location
    ```rust
    fn make_string() -> String {
        let s = String::from("hello"); // lifetime at s starts
        s                              // lifetime at s ENDS — moved to caller
    }
    let result = make_string();        // lifetime at result starts
    ```

  From the compiler's perspective, a move transitions a variable from an **initialized** state to an **uninitialized** state. The concrete lifetime of the data continues at the new binding, but the concrete lifetime of the old variable ends instantly. This is why moves are mathematically simple for the compiler to track — it's just a boolean switch: "is this value still alive here, or was it moved?" If you try to use it after a move, the compiler flags a **liveness violation** (use-after-move), not a lifetime violation. Moves use **liveness analysis**, while borrows use **lifetime analysis** (a span comparison).

- **Borrowing / Referencing** — this is where the heavy lifting happens. Borrows don't terminate lifetimes; they create strict **dependency constraints**. When you create a reference, the compiler calculates the concrete lifetime of that borrow — from the moment it's created to its very last use (Non-Lexical Lifetimes). The golden rule is a **subset calculation**:

  > The concrete lifetime of the borrow must be a strict subset of the concrete lifetime of the owner.

  - **Local variable references** — `let r = &x`; the simplest form of borrowing; r must not outlive x
    ```rust
    let r;
    {
        let x = 5;
        r = &x;        // r borrows x
    }                  // x is dropped — r's constraint is violated
    println!("{r}");   // ERROR: r outlives x
    ```
  - **Function parameters** — passing a reference into a function; the value must be alive for the duration of the call
    ```rust
    fn print(s: &str) { println!("{s}"); }
    let name = String::from("Alice");
    print(&name);      // name is borrowed for the duration of the call
    println!("{name}"); // fine — borrow ended when function returned
    ```
  - **Function return types** — when returning a reference, the compiler needs to know which input it's tied to
    ```rust
    fn first_word(s: &str) -> &str {
        s.split_whitespace().next().unwrap()
    }   // returned reference is tied to s — cannot outlive the input
    ```
  - **Non-move closure captures** — by default, closures capture variables by reference; the closure must not outlive the borrowed value
    ```rust
    let x = 5;
    let print = || println!("{x}"); // x is borrowed by the closure
    print();                        // fine — x is still alive
    // closure cannot outlive x
    ```
  - **Struct/enum fields holding references** — the struct's or enum variant's lifetime is constrained by the reference it holds
    ```rust
    struct Important<'a> {
        content: &'a str,  // struct cannot outlive the referenced str
    }
    let text = String::from("critical");
    let note = Important { content: &text };
    drop(text);            // ERROR: text is still borrowed by note
    ```
  - **Trait objects** — `Box<dyn Trait>` implicitly requires `'static`; storing a reference inside requires an explicit lifetime bound
    ```rust
    let s = String::from("hello");
    let obj: Box<dyn std::fmt::Display> = Box::new(&s); // ERROR: &s is not 'static
                                                         // Box<dyn Trait> expects 'static by default

    // Fix: explicitly allow a non-static lifetime
    let obj: Box<dyn std::fmt::Display + '_> = Box::new(&s); // fine
    ```
  - **`'static` lifetime** — a special case meaning the value lives for the entire program (e.g. string literals, global constants)
    ```rust
    let s: &'static str = "hello"; // string literals are always 'static
    fn needs_static(s: &'static str) { println!("{s}"); }
    needs_static(s); // fine
    ```

## The Intersection: Where Borrow Checker Errors Happen

Almost every borrow checker error occurs because an operation from **Moving** attempts to occur while a constraint from **Borrowing** is still active — the move tries to end or transfer ownership of a value, but an outstanding borrow says the value must still be alive.

```rust
let mut v = vec![1, 2, 3];
let first = &v[0];   // Borrow starts (constraint active)
v.push(4);           // Move/mutation attempt
                     // ERROR: cannot mutate v while borrow of v[0] is active
println!("{first}"); // borrow ends here (last use)
```

Fix — let the borrow end first:
```rust
let mut v = vec![1, 2, 3];
let first = &v[0];
println!("{first}"); // last use — borrow ENDS here
v.push(4);           // no active borrow — mutation is now allowed
```

When you hit a borrow checker error, the mental model is:
> "Something from Moving is trying to happen while something from Borrowing is still constraining that value."

---

> **Note — Non-Lexical Lifetimes (NLL):**
> **Lexical** means "based on source code block structure (`{}`)". Before NLL, borrows ended at the closing `}` — purely based on syntax.
> **Non-Lexical** means the borrow ends at the **last point it is actually used**, regardless of where the `}` is:
> ```rust
> // With NLL (modern Rust)
> let mut x = 5;
> let r = &x;        // borrow starts
> println!("{r}");   // last use — borrow ENDS HERE
> x = 10;            // fine: no active borrow, even though we're still inside the same block
> ```
> NLL made the borrow checker significantly smarter — it tracks the actual **usage span** rather than the syntactic block boundary, eliminating many false positives that frustrated early Rust programmers.

> **Note — The Borrowing Rule:**
> At any given time, you can have either:
> - **one mutable reference** (`&mut T`), OR
> - **any number of immutable references** (`&T`)
>
> But never both at the same time. This prevents data races at compile time.
> ```rust
> let mut x = 5;
> let r1 = &x;      // immutable borrow
> let r2 = &x;      // fine — multiple immutable borrows allowed
> let r3 = &mut x;  // ERROR: cannot borrow x as mutable while immutable borrows exist
> println!("{r1} {r2} {r3}");
> ```
> ```rust
> let mut x = 5;
> let r1 = &mut x;  // mutable borrow
> let r2 = &mut x;  // ERROR: cannot have two mutable borrows at the same time
> ```

## Lifetime-Related Compiler Errors

These are the compiler errors that signal a lifetime violation. Each one maps to a specific way lifetimes can be misused.

---

### Category 1: Missing Lifetime Annotations

#### E0106 — `missing lifetime specifier`
The compiler cannot apply lifetime elision and needs an explicit lifetime parameter.
```rust
struct Foo {
    x: &bool,        // ERROR E0106: missing lifetime specifier
}

fn longest(x: &str, y: &str) -> &str { // ERROR E0106: missing lifetime specifier
    if x.len() > y.len() { x } else { y }
}
```
Fix: add a lifetime parameter — `struct Foo<'a> { x: &'a bool }` / `fn longest<'a>(x: &'a str, y: &'a str) -> &'a str`.

> Source: [E0106 — Rust error codes](https://doc.rust-lang.org/error_codes/E0106.html)

#### E0637 — Invalid lifetime identifier
`'_` (anonymous lifetime) or a bare `&T` reference was used in a position where an explicit named lifetime is required.
```rust
fn foo<'_>(x: &'_ str) -> &'_ str { x } // ERROR E0637: '_  cannot be used as a lifetime identifier here
```
Fix: use a proper named lifetime — `fn foo<'a>(x: &'a str) -> &'a str`.

> Source: [E0637 — Rust error codes](https://doc.rust-lang.org/error_codes/E0637.html)

---

### Category 2: Dangling References

#### E0597 — `borrowed value does not live long enough`
The most common lifetime error. A reference outlives the value it points to — the value was dropped while still borrowed.
```rust
let r;
{
    let x = 5;
    r = &x;        // ERROR E0597: x does not live long enough
}                  // x dropped here — r is now dangling
println!("{r}");
```

> Source: [E0597 — Rust error codes](https://doc.rust-lang.org/error_codes/E0597.html)

---

### Category 3: Lifetime Bounds Violations

#### E0309 — `the parameter type T may not live long enough`
A generic type is used in a position that requires it to outlive a specific lifetime, but no `T: 'a` bound was declared.
```rust
struct Ref<'a, T> {
    r: &'a T,      // ERROR E0309: T may not live long enough — needs T: 'a
}
```
Fix: add the outlives bound — `struct Ref<'a, T: 'a> { r: &'a T }`.

> Source: [E0309 — Rust error codes](https://doc.rust-lang.org/error_codes/E0309.html)

#### E0310 — `the parameter type T may not live long enough` (static)
Same as E0309, but the required bound is `'static`. The type must live for the entire program but no `T: 'static` constraint was declared.
```rust
fn store<T>(val: T) -> &'static T {
    Box::leak(Box::new(val)) // ERROR E0310: T may not live long enough — needs T: 'static
}
```
Fix: add `T: 'static`.

> Source: [E0310 — Rust error codes](https://doc.rust-lang.org/error_codes/E0310.html)

#### E0491 — `a reference has a longer lifetime than the data it references`
A reference's lifetime outlives the data it references in a trait implementation with multiple lifetime parameters.
```rust
// lifetime 'a outlives 'b but the impl tries to return a reference constrained by 'b
```
Fix: add a lifetime relationship bound — `'a: 'b` ("`'a` outlives `'b`").

> Source: [E0491 — Rust error codes](https://doc.rust-lang.org/error_codes/E0491.html)

---

### Category 4: Lifetime Mismatches

#### E0621 — `lifetime mismatch` (signature vs body)
The function signature declares a specific lifetime relationship, but the body returns data that violates it.
```rust
fn foo<'a>(x: &'a str, y: &str) -> &'a str {
    y  // ERROR E0621: lifetime mismatch — y has a different lifetime than 'a
}
```
Fix: give both parameters the same lifetime — `fn foo<'a>(x: &'a str, y: &'a str) -> &'a str`.

> Source: [E0621 — Rust error codes](https://doc.rust-lang.org/error_codes/E0621.html)

#### E0623 — `lifetime mismatch` (unrelated lifetimes)
Two distinct lifetimes are used in a context that requires them to be related or equal.
```rust
fn foo<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
    y  // ERROR E0623: lifetime mismatch — 'b is unrelated to 'a
}
```
Fix: establish a relationship — `'b: 'a` ("`'b` outlives `'a`") or collapse to a single lifetime.

> Source: [E0623 — Rust error codes](https://doc.rust-lang.org/error_codes/E0623.html)

---

### Category 5: Async / Coroutines

#### E0626 — `borrow in coroutine persists across yield point`
A borrow is held across a `yield` point in a coroutine, which is unsafe because the coroutine may be moved between yields.
```rust
// inside a coroutine/generator:
let s = String::from("hello");
let r = &s;
yield;             // ERROR E0626: borrow of s persists across yield
println!("{r}");
```
Fix: drop the borrow before yielding, use owned values, or mark the coroutine as `static`.

> Source: [E0626 — Rust error codes](https://doc.rust-lang.org/error_codes/E0626.html)

---

### Summary

| Error | Category | Message |
|-------|----------|---------|
| E0106 | Missing annotation | `missing lifetime specifier` |
| E0637 | Invalid syntax | `'_` used as lifetime identifier in illegal position |
| E0597 | Dangling reference | `borrowed value does not live long enough` |
| E0309 | Bounds violation | `parameter type T may not live long enough` (needs `T: 'a`) |
| E0310 | Bounds violation | `parameter type T may not live long enough` (needs `T: 'static`) |
| E0491 | Bounds violation | `reference has a longer lifetime than the data it references` |
| E0621 | Mismatch | `lifetime mismatch` (body vs signature) |
| E0623 | Mismatch | `lifetime mismatch` (unrelated lifetimes) |
| E0626 | Async/coroutines | `borrow in coroutine persists across yield point` |

---

There are two types of lifetimes: concrete and generic

Concrete lifetime is the time during which a value exists at a particular memory location.

A lifetime starts when a value is created and moved into a particular memory location and ends when a value is dropped or moved out of a particular memory location.

It is more accurate to look at lifetimes from a **location view** rather than a scope view — a variable name may still be in scope, but the lifetime at its memory location can have already ended.

Example 1 — lifetime ends at drop (end of scope):
```rust
let x = 5;    // lifetime at x's location starts
              // ...
              // lifetime at x's location ends when x is dropped at end of scope
```

Example 2 — lifetime ends at move, before end of scope:
```rust
let x = String::from("hello"); // lifetime at x's location starts
let y = x;                     // value moved out — lifetime at x's location ENDS here
                               // x is still "in scope" but its location holds no valid value
println!("{x}");               // ERROR: x's lifetime has ended
```

Example 3 — dangling reference (borrow checker rejects):
```rust
let r;
{
    let x = 5;    // lifetime at x's location starts
    r = &x;       // r points to x's location
}                 // x is dropped — lifetime at x's location ENDS
println!("{r}");  // ERROR: r outlives the value at x's location
```
