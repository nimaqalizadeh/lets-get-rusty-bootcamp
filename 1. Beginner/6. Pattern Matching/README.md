# Pattern Matching in Rust — A Comprehensive Guide

Pattern matching is one of the most powerful features of Rust's `enum`s, but **it is not limited to enums!** You can use pattern matching with `struct`s, tuples, arrays, slices, references, and even plain variables.

When you use pattern matching to "pull apart" a value into its components, it is called **destructuring**.

This document covers:

1. The two flavors of patterns: **refutable** vs **irrefutable**
2. **Where** patterns can appear in code (8 places)
3. **What** can be a pattern (literals, ranges, bindings, wildcards, guards, `@` bindings, refs, etc.)
4. Pattern matching on every Rust data shape: structs, enums, tuples, arrays/slices, references, boxes
5. The `?` operator
6. Common pitfalls and best practices

---

## Refutable vs. Irrefutable Patterns

Every pattern in Rust falls into one of two categories:

| Type            | Definition                                                                | Example                  |
| --------------- | ------------------------------------------------------------------------- | ------------------------ |
| **Irrefutable** | A pattern that will match for **any possible value** of the type.         | `let x = 5;` — `x` is irrefutable. |
| **Refutable**   | A pattern that **might fail** to match some values.                       | `if let Some(v) = opt` — fails when `opt` is `None`. |

**Rule of thumb:**

- `let` and function parameters require **irrefutable** patterns (they must always succeed).
- `if let`, `while let`, and `match` arms (except the wildcard) use **refutable** patterns.
- `match` as a whole must be **exhaustive** — every possible value must be covered by some arm.

```rust
// ✅ Irrefutable — a single variable binding always matches
let x = 5;

// ✅ Irrefutable — a struct only has one shape
let Point { x, y } = p;

// ❌ Refutable used where irrefutable is required — compile error!
let Some(v) = maybe_value; // error: refutable pattern in local binding

// ✅ Use `if let` or `let else` instead
if let Some(v) = maybe_value { /* ... */ }
let Some(v) = maybe_value else { return; };
```

---

## Where Patterns Can Appear

### 1. `match` Expressions

The classic. Each arm has a pattern, an optional guard, and a body.

```rust
let number = 13;

match number {
    1 => println!("one"),
    2 | 3 | 5 | 7 | 11 => println!("prime"),
    13..=19 => println!("a teen"),
    _ => println!("something else"),
}
```

`match` must be **exhaustive** — the compiler will reject it if any value could slip through.

### 2. `if let`

Use when you only care about **one** variant and want to ignore the rest.

```rust
let maybe = Some(7);

if let Some(n) = maybe {
    println!("got {}", n);
} else {
    println!("nothing");
}
```

### 3. `let else` (Rust 1.65+)

A clean way to **bail out early** when a pattern doesn't match. The `else` branch must diverge (`return`, `break`, `continue`, `panic!`, etc.).

```rust
fn parse_age(s: &str) -> Option<u32> {
    let Ok(n) = s.parse::<u32>() else {
        return None;
    };
    // `n` is in scope below — no nesting, no `.unwrap()`
    Some(n)
}
```

### 4. `while let`

Loop while a pattern keeps matching.

```rust
let mut stack = vec![1, 2, 3];

while let Some(top) = stack.pop() {
    println!("{}", top);
}
// Loop exits when `pop()` returns `None`
```

### 5. `for` Loops

The "loop variable" in `for` is itself a pattern.

```rust
let pairs = vec![("a", 1), ("b", 2), ("c", 3)];

for (letter, number) in pairs {
    println!("{} = {}", letter, number);
}

// .enumerate() gives back a tuple, which we destructure
for (i, value) in vec!["x", "y", "z"].iter().enumerate() {
    println!("{}: {}", i, value);
}
```

### 6. `let` Statements

Every `let` binding is a pattern. `let x = 5;` is just the simplest possible pattern (a single name).

```rust
let (a, b, c) = (1, 2, 3);           // destructure a tuple
let [first, second, third] = [10, 20, 30]; // destructure an array
let Point { x, y } = Point { x: 1, y: 2 }; // destructure a struct
```

### 7. Function and Closure Parameters

Parameters are patterns too.

```rust
fn print_point(&Point { x, y }: &Point) {
    println!("({}, {})", x, y);
}

let add = |(a, b): (i32, i32)| a + b;
add((3, 4));
```

### 8. Inside Closures / Iterator Chains

Same idea — the closure argument is a pattern.

```rust
let points = vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }];
let xs: Vec<i32> = points.iter().map(|Point { x, .. }| *x).collect();
```

---

## Pattern Building Blocks

### Literals

Match an exact value.

```rust
match x {
    0 => println!("zero"),
    1 => println!("one"),
    _ => println!("other"),
}
```

### Named Variables (Bindings)

A bare identifier in a pattern **binds** the matched value to that name.

```rust
let x = Some(5);

match x {
    Some(n) => println!("got {}", n), // `n` binds to the inner value
    None => println!("nothing"),
}
```

⚠️ **Shadowing trap:** Inside a `match`, a bare name **always creates a new binding** — it never compares against an outer variable of the same name.

```rust
let y = 10;
let x = Some(5);

match x {
    Some(y) => println!("matched, y = {}", y), // prints 5, NOT compared to outer y!
    None => (),
}
```

To compare against an outer variable, use a **match guard** (see below) or a constant.

### Multiple Patterns with `|`

The OR operator. Matches if **any** pattern matches.

```rust
match x {
    1 | 2 | 3 => println!("one, two, or three"),
    _ => println!("anything else"),
}
```

### Range Patterns: `..=`

Match an inclusive range. Works for integers and `char`.

```rust
match age {
    0..=12 => println!("child"),
    13..=19 => println!("teen"),
    20..=64 => println!("adult"),
    _ => println!("senior"),
}

match letter {
    'a'..='z' => println!("lowercase"),
    'A'..='Z' => println!("uppercase"),
    _ => println!("not a letter"),
}
```

Exclusive ranges (`..`) in patterns are unstable — stick to `..=`.

### Wildcard `_`

Match anything, **without binding** to a name.

```rust
match (x, y) {
    (0, _) => println!("x is zero"),
    (_, 0) => println!("y is zero"),
    _ => println!("neither is zero"),
}
```

The compiler will not warn about unused `_` bindings — useful for prefixed names like `_unused` too.

### Rest Pattern `..`

Ignore the **rest** of a struct, tuple, or slice.

```rust
// Tuple
let nums = (1, 2, 3, 4, 5);
let (first, .., last) = nums;        // first = 1, last = 5

// Slice
let arr = [1, 2, 3, 4, 5];
match arr {
    [first, .., last] => println!("{} ... {}", first, last),
}

// Struct — ignore fields you don't care about
struct Config { host: String, port: u16, timeout: u32, retries: u8 }
let Config { host, port, .. } = config;
```

### Match Guards: `if`

An extra `if` condition on a match arm. The arm only matches if the pattern matches **and** the guard returns `true`.

```rust
let x = Some(4);

match x {
    Some(n) if n < 5 => println!("less than 5: {}", n),
    Some(n) if n % 2 == 0 => println!("even: {}", n),
    Some(n) => println!("other: {}", n),
    None => println!("none"),
}
```

Guards also solve the shadowing problem:

```rust
let y = 10;
match Some(5) {
    Some(n) if n == y => println!("matched outer y"),
    Some(n) => println!("got {}", n),
    None => (),
}
```

> ⚠️ **Note:** Guards are opaque to the exhaustiveness checker. `match x { Some(n) if n > 0 => ..., None => ... }` is **not** exhaustive — you still need an arm for `Some(n)` where `n <= 0`.

### `@` Bindings (The "At" Operator)

`@` lets you **bind a value to a name while simultaneously testing it against a sub-pattern**. The general form is:

```text
name @ pattern
```

It's the answer to a common dilemma: writing a literal or range pattern like `3..=7` **checks** that the value falls in that range, but doesn't give you a name to use the value afterward. A bare binding like `id` **gives you a name** but doesn't restrict the value. `@` does both at once.

Think of it as "**also** call this matched value `name`."

#### Example 1: The classic problem `@` solves

```rust
enum Message {
    Hello { id: i32 },
}

let msg = Message::Hello { id: 5 };

match msg {
    // ✅ Tests range AND binds — we can print id_var
    Message::Hello { id: id_var @ 3..=7 } => {
        println!("id in range 3..=7: {}", id_var);
    }
    // ⚠️ Tests range but NO binding — `id` is not accessible here
    Message::Hello { id: 10..=12 } => {
        println!("id in another range (value not captured)");
    }
    // ✅ Binds but no range check — `id` is any other i32
    Message::Hello { id } => {
        println!("any other id: {}", id);
    }
}
```

#### Example 2: `@` with literals

You can `@`-bind even when the sub-pattern is a single literal (mostly useful when you want a named "constant-but-also-bound" alias):

```rust
match x {
    n @ 0 => println!("zero, stored in n: {}", n),
    n @ 1 => println!("one, stored in n: {}", n),
    n => println!("other: {}", n),
}
```

#### Example 3: `@` with the OR pattern

Bind a value when it matches **any** of several alternatives. Note the parentheses are required around the alternatives:

```rust
let x = 5;

match x {
    n @ (1 | 2 | 3) => println!("small: {}", n),
    n @ (4..=10) => println!("medium: {}", n),
    n => println!("other: {}", n),
}
```

#### Example 4: `@` with struct destructuring

You can capture a **whole struct** with `@` while also restricting one of its fields:

```rust
#[derive(Debug)]
struct Point { x: i32, y: i32 }

let p = Point { x: 0, y: 7 };

match p {
    // Capture the whole point as `origin` while requiring x == 0 AND y == 0
    origin @ Point { x: 0, y: 0 } => println!("origin: {:?}", origin),

    // Capture the whole point as `on_axis` while requiring x == 0
    on_axis @ Point { x: 0, .. } => println!("on y-axis: {:?}", on_axis),

    // Bind a sub-value too — both `whole` and `xv` are usable
    whole @ Point { x: xv @ 1..=10, .. } => {
        println!("x is small ({}), full point: {:?}", xv, whole);
    }

    other => println!("other: {:?}", other),
}
```

#### Example 5: `@` with enum variants

Useful when you want to keep the entire variant around but also peek inside:

```rust
#[derive(Debug)]
enum Event {
    Click { x: i32, y: i32 },
    KeyPress(char),
    Scroll(i32),
}

fn handle(event: Event) {
    match event {
        // Bind the entire Click variant AND extract the x coordinate
        click @ Event::Click { x: cx @ 0..=100, .. } => {
            println!("click in left zone (x={}): forwarding {:?}", cx, click);
            forward(click); // we still own the whole event
        }
        Event::KeyPress(c @ ('a'..='z' | 'A'..='Z')) => {
            println!("letter pressed: {}", c);
        }
        Event::Scroll(amount @ ..=0) => {
            println!("scrolling up by {}", -amount);
        }
        other => println!("other event: {:?}", other),
    }
}
# fn forward(_: Event) {}
```

#### Example 6: `@` with slice patterns

This is where `@` really shines — capturing a **sub-slice** with `name @ ..`:

```rust
let nums = [1, 2, 3, 4, 5];

match nums {
    [first, middle @ .., last] => {
        println!("first: {}", first);
        println!("middle: {:?}", middle); // &[i32] containing [2, 3, 4]
        println!("last: {}", last);
    }
}

// On a Vec / slice
fn process(slice: &[i32]) {
    match slice {
        [] => println!("empty"),
        [only] => println!("one element: {}", only),
        [head, tail @ ..] => {
            println!("head: {}, tail length: {}", head, tail.len());
        }
    }
}
```

You can even **constrain** the captured slice with a guard:

```rust
match nums {
    [first, middle @ .., last] if middle.len() >= 2 => {
        println!("at least 4 elements, middle: {:?}", middle);
    }
    _ => println!("too short"),
}
```

#### Example 7: Nested `@` bindings

Multiple `@`s in one pattern? Totally fine. Each one binds its own name at its own level:

```rust
struct Order { id: u32, item: Item }
struct Item { name: String, price: u32 }

let order = Order {
    id: 42,
    item: Item { name: "book".into(), price: 25 },
};

match order {
    whole @ Order {
        id: oid @ 1..=99,
        item: it @ Item { price: p @ 10..=50, .. },
    } => {
        println!("order id {} (in range), item {:?} priced at {}", oid, it.name, p);
        println!("full order kept too: id={}", whole.id);
    }
    _ => println!("doesn't match"),
}
```

#### Example 8: `@` in `if let` and `let else`

`@` works anywhere a pattern works, not just in `match`:

```rust
let value = Some(42);

// if let
if let Some(n @ 1..=100) = value {
    println!("got a small positive number: {}", n);
}

// let else
fn small_positive(opt: Option<i32>) -> i32 {
    let Some(n @ 1..=100) = opt else {
        return 0;
    };
    n * 2
}
```

#### Example 9: `@` with references / match ergonomics

When matching on a reference, the bound name takes the same reference-ness as the matched value:

```rust
let words = vec!["hello".to_string(), "world".to_string()];

for w @ s in &words {
    // `w` is &String (same as `s`); both refer to the same value.
    // Useful when one name is needed for a sub-pattern and another for the whole.
    println!("{} has {} chars", w, s.len());
}
```

A more realistic case — keep both the wrapping `Option` and its inner value:

```rust
let pair = (Some(5), "label");

match pair {
    (opt @ Some(n), label) if n > 0 => {
        println!("{}: positive value {}, full option: {:?}", label, n, opt);
    }
    _ => println!("other"),
}
```

#### When you actually need `@`

You need `@` whenever **all three** are true:

1. You want to **test** a value against a sub-pattern (range, literal, struct shape, etc.).
2. You **also** want to use the value (or the larger surrounding value) in the arm body.
3. The sub-pattern doesn't already give you a name (e.g. `3..=7`, `Some(_)`, `Point { x: 0, .. }`).

If the sub-pattern already binds names you can use (e.g. `Point { x, y }`), you don't need `@` — just use those names.

> 💡 **Mnemonic:** Read `name @ pattern` as "name, **at** the position of this pattern." The value at that position gets bound to `name` if the pattern matches.

---

## Pattern Matching on Every Data Shape

### Structs

```rust
struct Point { x: i32, y: i32 }
let p = Point { x: 0, y: 7 };

// With match — checking specific field values
match p {
    Point { x, y: 0 } => println!("on x axis at {}", x),
    Point { x: 0, y } => println!("on y axis at {}", y),
    Point { x, y } => println!("({}, {})", x, y),
}

// With let — pure destructuring (irrefutable)
let Point { x, y } = p;

// Field renaming — bind `x` field to local name `a`
let Point { x: a, y: b } = p;

// Shorthand — `Point { x, y }` is sugar for `Point { x: x, y: y }`
```

**Tuple structs** work the same way, but with positional fields:

```rust
struct Color(u8, u8, u8);
let c = Color(255, 128, 0);

let Color(r, g, b) = c;
match c {
    Color(0, 0, 0) => println!("black"),
    Color(r, _, _) if r > 200 => println!("very red"),
    Color(r, g, b) => println!("rgb({}, {}, {})", r, g, b),
}
```

**Unit structs** match by name only:

```rust
struct Marker;
let m = Marker;
let Marker = m; // matches
```

### Enums

The classic motivation for pattern matching. Each variant has its own pattern shape.

```rust
enum Message {
    Quit,                       // unit variant
    Move { x: i32, y: i32 },    // struct-like variant
    Write(String),              // tuple variant with one field
    ChangeColor(i32, i32, i32), // tuple variant with three fields
}

match msg {
    Message::Quit => println!("quit"),
    Message::Move { x, y } => println!("move to ({}, {})", x, y),
    Message::Write(text) => println!("write: {}", text),
    Message::ChangeColor(r, g, b) => println!("color: {}, {}, {}", r, g, b),
}
```

`Option<T>` and `Result<T, E>` are just enums:

```rust
match maybe_num {
    Some(n) if n > 0 => println!("positive: {}", n),
    Some(0) => println!("zero"),
    Some(n) => println!("negative: {}", n),
    None => println!("nothing"),
}

match parse_result {
    Ok(value) => println!("ok: {}", value),
    Err(e) => println!("err: {}", e),
}
```

### Tuples

```rust
let pair = (1, -1);

match pair {
    (0, 0) => println!("origin"),
    (x, 0) => println!("on x axis at {}", x),
    (0, y) => println!("on y axis at {}", y),
    (x, y) if x == y => println!("on the diagonal at {}", x),
    (x, y) => println!("({}, {})", x, y),
}

// Destructuring assignment via let
let (a, b, c) = (1, 2, 3);

// Nested tuples
let ((a, b), c) = ((1, 2), 3);
```

### Arrays and Slices

Pattern matching on arrays/slices is incredibly expressive in Rust.

```rust
let nums = [1, 2, 3, 4, 5];

match nums {
    [] => println!("empty (impossible — array has fixed size 5)"),
    [only] => println!("one: {}", only),
    [first, second] => println!("two: {}, {}", first, second),
    [first, .., last] => println!("first {} last {}", first, last),
    [first, middle @ .., last] => {
        println!("first: {}, middle: {:?}, last: {}", first, middle, last);
    }
}
```

#### What does `@` do in `[first, middle @ .., last]`?

The `@` **captures the "rest" of the slice into a named variable** called `middle`.

```rust
let nums = [1, 2, 3, 4, 5];

match nums {
    [first, middle @ .., last] => {
        // first  = 1
        // last   = 5
        // middle = &[2, 3, 4]  ← this is what `@` gives you
    }
}
```

**Without `@` vs with `@`:**

| Pattern                            | What it does                                                                 |
| ---------------------------------- | ---------------------------------------------------------------------------- |
| `[first, .., last]`                | Matches arrays with first/last, **ignores** the middle. No way to access it. |
| `[first, middle @ .., last]`       | Same shape, **but** the middle elements are bound to a name (`middle`).      |

The `..` on its own is the **rest pattern** — it says "zero or more elements here, I don't care about them." Adding `name @` in front says "also, give me a name for those elements so I can use them."

Without `@`, you'd have to compute the middle yourself:

```rust
// Without @ — clunky
match nums {
    [first, .., last] => {
        let middle = &nums[1..nums.len() - 1]; // manual slicing
        println!("middle: {:?}", middle);
    }
}

// With @ — Rust does it for you
match nums {
    [first, middle @ .., last] => {
        println!("middle: {:?}", middle); // ← directly available
    }
}
```

This is the **same** `@` operator as in `id_var @ 3..=7` — `name @ sub_pattern` means "match `sub_pattern`, and also bind the matched portion to `name`." Here the `sub_pattern` is `..` (the rest pattern), so `name` binds to whatever slice the rest pattern absorbed.

For slices (`&[T]`), all lengths are possible, so all of these arms are reachable.

```rust
fn describe(slice: &[i32]) -> &'static str {
    match slice {
        [] => "empty",
        [_] => "one element",
        [_, _] => "two elements",
        [first, .., last] if first == last => "first equals last",
        _ => "many elements",
    }
}
```

The `name @ ..` syntax captures the "rest" into a slice:

```rust
match v.as_slice() {
    [head, tail @ ..] => println!("head: {}, tail: {:?}", head, tail),
    [] => println!("empty"),
}
```

### References, `ref`, and `ref mut`

When matching on a reference, you can either dereference it explicitly with `&` in the pattern, or use `ref` to take a reference to a matched value.

```rust
let reference = &4;

// Destructuring the reference
match reference {
    &val => println!("got value {}", val),
}

// Equivalent — dereferencing before match
match *reference {
    val => println!("got value {}", val),
}
```

`ref` (and `ref mut`) goes the **other** direction — it creates a reference instead of moving the value.

```rust
let s = String::from("hello");

match s {
    ref r => println!("got a reference to {}", r),
    // Without `ref`, `s` would be moved into the arm
}

let mut value = 10;
match value {
    ref mut r => *r += 1,
}
println!("{}", value); // 11
```

In modern Rust (2021+), **match ergonomics** often insert these for you, so `ref` is needed less often than it used to be:

```rust
let opt = Some(String::from("hi"));

// `s` is automatically &String — no explicit `ref` needed
match &opt {
    Some(s) => println!("{}", s),
    None => (),
}
```

### Boxes and Smart Pointers

`Box<T>` derefs automatically in match ergonomics:

```rust
let boxed: Box<i32> = Box::new(5);
match *boxed {
    5 => println!("five"),
    n => println!("{}", n),
}
```

For pattern-matching **inside** a box without dereferencing, the `box` pattern is unstable — usually you just `*box_val` first.

### Nested Destructuring

Patterns compose. You can destructure as deep as the shape goes.

```rust
struct User {
    name: String,
    address: Address,
}
struct Address {
    city: String,
    zip: String,
}

let user = User {
    name: "Alice".into(),
    address: Address { city: "Berlin".into(), zip: "10115".into() },
};

let User { name, address: Address { city, .. } } = user;
println!("{} lives in {}", name, city);
```

You can mix enum and struct destructuring freely:

```rust
let messages = vec![
    Ok(Some(42)),
    Ok(None),
    Err("boom"),
];

for msg in messages {
    match msg {
        Ok(Some(n)) => println!("got {}", n),
        Ok(None) => println!("empty ok"),
        Err(e) => println!("error: {}", e),
    }
}
```

---

## The `?` Operator

The `?` operator uses pattern matching **under the hood**, but is hardcoded for types implementing the `Try` trait — primarily `Result` and `Option`.

For `Result<T, E>`, `expr?` roughly desugars to:

```rust
match expr {
    Ok(value) => value,
    Err(e) => return Err(e.into()), // early return!
}
```

For `Option<T>`, it desugars to:

```rust
match expr {
    Some(value) => value,
    None => return None,
}
```

Example chain:

```rust
fn read_first_line(path: &str) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?; // returns Err on failure
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf.lines().next().unwrap_or("").to_string())
}
```

Since `Result` and `Option` are enums, the `?` operator **is** technically an enum-specific feature — it's a stylized shortcut for the common "destructure and early-return" pattern.

---

## Common Pitfalls

### 1. Forgetting that bindings shadow

```rust
let n = 10;
match Some(5) {
    Some(n) => println!("{}", n), // prints 5, not 10!
    None => (),
}
```

Use a match guard (`Some(x) if x == n`) or a `const` to compare against an outer value.

### 2. Non-exhaustive match guards

```rust
match x {
    n if n > 0 => /* ... */,
    n if n < 0 => /* ... */,
    // ❌ missing the n == 0 case — compiler won't catch this is exhaustive logically,
    //    but it CAN tell that you didn't cover every i32 value pattern-wise.
}
```

Always add a wildcard arm or an arm without a guard for the leftover case.

### 3. Refutable pattern in `let`

```rust
let Some(x) = some_option; // ❌ compile error
```

Use `if let`, `let else`, or `match`.

### 4. Move vs. borrow inside match

A `match` on an owned value can **move** parts of it into arms. Match on a reference (`match &value`) or use `ref` to avoid this.

```rust
let s = Some(String::from("hi"));

match s {
    Some(text) => println!("{}", text), // moves the String out of s
    None => (),
}
// `s` is no longer usable here

// Fix: borrow instead
match &s {
    Some(text) => println!("{}", text), // text: &String
    None => (),
}
// `s` is still owned and usable
```

### 5. Forgetting `..` in struct patterns

If you add a new field to a struct, every pattern that listed all its fields breaks. Using `..` makes patterns future-proof when you don't care about every field.

```rust
let Config { host, port, .. } = config; // adding fields later won't break this
```

---

## Summary

- **Enums** require pattern matching (`match`, `if let`) because the compiler needs to check which variant is present at runtime.
- **Structs**, **tuples**, **arrays**, **slices**, **references**, and even **`let` bindings** all use pattern matching — most often to **destructure** their inner data.
- Patterns can appear in: `match`, `if let`, `let else`, `while let`, `for` loops, `let` statements, function parameters, and closure parameters.
- Pattern building blocks: literals, named bindings, `|`, ranges (`..=`), `_`, `..`, match guards (`if`), `@` bindings, and `ref`/`ref mut`.
- `match` must be **exhaustive**; `let` and parameters require **irrefutable** patterns.
- The `?` operator is a stylized desugaring of `match` for `Result`/`Option`.

If you can read a value's shape, you can pattern match on it. That uniform mechanism — across enums, structs, slices, and references — is what makes Rust's pattern matching one of its defining features.
