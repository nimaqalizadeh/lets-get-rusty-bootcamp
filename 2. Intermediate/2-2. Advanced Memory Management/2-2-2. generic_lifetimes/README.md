# Generic lifetime

In generic lifetime remember that `'a` is not a concrete lifetime, it describes a relationship between lifetimes. By adding `'a`, we are telling the borrow checker: in any scope where `p1` and `p2` are passed to this function, the return value must not outlive the shortest lifetime of `p1` and `p2`.

lifetime specifiers also known as generic lifetime annotations are a way to describe a relationship between lifetimes of references.

By adding Generic lifetime annotation, we are able to help the borrow checker analyze our code and catch the bugs

```rust
// 'a is NOT a concrete lifetime — it describes the relationship between
// the lifetimes of the inputs and the return value.
// The return value will live at least as long as the shorter of p1 and p2.
fn first_turn<'a>(p1: &'a str, p2: &'a str) -> &'a str {
    if rand::random() {
        p1
    } else {
        p2
    }
}
```

If you use two separate lifetime parameters like `fn first_turn<'a, 'b>(p1: &'a str, p2: &'b str) -> &'a str`, the compiler will reject it — because the function can return either `p1` or `p2` at runtime, but `'b` and `'a` are unrelated, so there's no guarantee `p2` lives long enough. You would need to add a bound `'b: 'a` to tell the borrow checker that `'b` lives at least as long as `'a`. Using a single `'a` for both is simpler — it resolves to the shorter of the two lifetimes.

## Why manual lifetime annotations are needed

The compiler cannot infer the intent across function boundaries on its own. Rust intentionally avoids **whole-program analysis** — it checks each function in isolation. This makes compilation faster and errors more local.

The compiler checks lifetimes in two separate steps:

1. **Inside the body** — it verifies that the implementation follows the lifetime rules declared in the signature
2. **At call sites** — it only looks at the signature to enforce safety for callers; the body is never re-analyzed for callers

The signature is a **contract**. When you call a function from an external crate, the compiler only has access to the compiled binary and the function signature — not the source code. Even in your own code, the rule is the same: the signature must be self-contained and tell the full story.

As the Rust Book states:

> "The lifetime annotations become part of the contract of the function, much like the types in the signature. Having function signatures contain the lifetime contract means the analysis the Rust compiler does can be simpler."

In short: **the compiler checks, but you define the rules.**

### Why not let the compiler infer it from the body?

There are actually three valid interpretations the compiler could choose from at the signature level:

1. Return lives as long as `p1` only → tie return to `p1`'s lifetime
2. Return lives as long as `p2` only → tie return to `p2`'s lifetime
3. Return lives as long as the shorter of both → tie return to both

If the compiler did whole-program analysis and looked at the body, it would see that either `p1` or `p2` is returned, and could safely conclude option 3 — the return lifetime must be the intersection of both:

```rust
// The body — compiler can see either p1 or p2 is returned,
// so it could conclude: return lifetime = intersection of p1 and p2.
fn first_turn(p1: &str, p2: &str) -> &str {
    if rand::random() { p1 } else { p2 }
}
```

But the compiler deliberately does **not** look at the body from the caller's perspective. At the call site, all it sees is the signature — no body, no idea which input is returned:

```rust
// The caller — compiler only sees the signature.
// Without lifetime annotations, it cannot conclude
// whether the return is tied to p1, p2, or both.
let p1: &str = "player 1";
let p2: &str = "player 2";
let result: &str = first_turn(p1, p2); // which lifetime does `result` have?
```

So you must declare it explicitly. Your annotation `<'a>` on both parameters is exactly you saying: "the return lifetime is the intersection of both" — which is precisely what the compiler would have concluded if it had looked at the body.

Note: when you **write** the function, the compiler checks both the signature and the body — it verifies that the body actually follows the lifetime rules you declared. But when you **call** the function, the compiler only checks the signature — the body is never re-analyzed for callers.

```rust
fn main() {
    let player1: String = String::from("player 1");
    let result: &str;

    {
        let player2: String = String::from("player 2");
        result = first_turn(player1.as_str(), player2.as_str());
        // player2 is dropped here
    }

    // This compiles fine — the compiler checked the signature of first_turn
    // and saw the return lifetime is tied only to p1 ('a), not p2.
    // So even though player2 is dropped above, the borrow checker is satisfied
    // because result's lifetime only depends on player1, which is still alive.
    println!("Player going first is: {}", result);
}

// When defining this function, the compiler checks both the signature and the body:
// - signature says: return lives as long as p1 ('a), p2 has no lifetime constraint on the return
// - body confirms: we always return p1, so the annotation is correct
// When calling this function, the compiler only looks at the signature:
// - return is tied to p1's lifetime → result is valid as long as player1 is alive ✓
fn first_turn<'a>(p1: &'a str, p2: &str) -> &'a str {
    p1
}
```

Lifetime annotations have **zero runtime overhead** — no extra code is generated, no runtime checks are added. They are completely erased after compilation and exist only for the borrow checker at compile time.

### Compile time vs runtime

|               | What happens                                                                                 |
| ------------- | -------------------------------------------------------------------------------------------- |
| `cargo check` | `rustc` reads source, checks types and lifetimes, reports errors — **no binary produced**    |
| `cargo build` | Same as check, then also translates to assembly and produces a binary                        |
| `cargo run`   | Compiles first (like `cargo build`), then executes the binary                                |
| **runtime**   | The binary runs — no lifetime information exists at this point, just raw memory and pointers |

Lifetime annotations are a **compile-time only** concept. During `cargo check` or `cargo build`, no functions are called and no code runs — `rustc` just analyzes the structure of your code statically. Once the binary is produced, all lifetime annotations are stripped out completely.

The red squiggles and errors you see in your IDE are shown by **rust-analyzer**, which runs the same compilation analysis continuously in the background as you type. It is not a separate checker — it is just a faster feedback loop on top of `rustc`, so you can catch and fix errors without running `cargo build` every time. Do not confuse this with compile time — rust-analyzer is a development tool; the real compile time is when `rustc` runs via `cargo check` or `cargo build`.

In general the lifetime of a returned value must be tied to the lifetime of the input parameters. There are three rules:

**1. The return reference must come from the inputs:**

```rust
fn get(s: &str) -> &str {
    s  // valid — return points to something passed in
}
```

**2. You cannot return a reference to something created inside the function:**

```rust
fn get() -> &str {
    let s = String::from("hello");
    &s  // invalid — s is dropped when function ends, reference would dangle
}
```

`s` is created inside the function and dropped when it returns, so the reference would point to freed memory.

**3. The one exception is `'static`:**

```rust
fn get() -> &'static str {
    "hello"  // valid — string literals live for the entire program
}
```

`'static` means the data lives for the entire duration of the program, so returning a reference to it is always safe.

```rust
fn main() {
    let player1: String = String::from("player 1");
    let result: &str;

    {
        let player2: String = String::from("player 2");
        result = first_turn(player1.as_str(), player2.as_str());
    }

    println!("Player going first is: {}", result);
}

// Although the signature says return lives as long as 'a (p1's lifetime),
// the body returns a 'static str. This compiles because 'static outlives
// everything — it satisfies any lifetime constraint including 'a.
fn first_turn<'a>(p1: &'a str, p2: &str) -> &'a str {
    let s: &'static str = "Let's Get Rusty!";
    s
}
```

Since the return is always `'static`, the `'a` annotation is unnecessary. The cleaner version removes the lifetime parameters entirely:

```rust
fn first_turn(p1: &str, p2: &str) -> &'static str {
    "Let's Get Rusty!"
}
```

video: lets get rusty/047.Generic Lifetimes
