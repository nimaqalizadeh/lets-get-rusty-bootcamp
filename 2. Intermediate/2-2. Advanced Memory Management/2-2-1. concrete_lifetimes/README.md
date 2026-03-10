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
