# Structs and lifetime elision

when storing references on structs, we must add generic lifetime annotations. By doing this, we are telling the borrow checker that the struct instance _cannot outlive_ the data it references

## How the compiler enforces this

For **functions and methods**, the compiler checks lifetimes in two separate steps:

- **Inside the body** — it verifies that the implementation follows the lifetime rules declared in the signature
- **At call sites** — it only looks at the signature; the body is never re-analyzed for callers

For **structs**, this distinction does not apply. There is no body separate from the definition — the struct definition is both the contract and the complete specification. The field types and lifetimes are fully visible, and callers see everything there is to see.

## How a caller knows the struct shouldn't outlive the referenced data

The lifetime parameter `'a` in the struct definition _is_ the encoding of that constraint:

```rust
struct Excerpt<'a> {
    text: &'a str,
}
```

The compiler records: "an `Excerpt<'a>` is only valid for the duration of `'a`." The lifetime becomes part of the struct's type — `Excerpt<'a>` and `Excerpt<'b>` are different types.

At instantiation, the compiler infers `'a` from the concrete reference passed in. From that point, the type itself carries the constraint — the caller never needs to inspect the fields. The compiler simply enforces that `Excerpt<'a>` is not used after `'a` expires, the same way it enforces that a `&'a str` is not used after `'a` expires.

## Multiple lifetime parameters

A struct can have multiple lifetime parameters when its fields come from independent references:

```rust
struct Multi<'a, 'b> {
    text: &'a str,
    number: &'b i32,
}
```

The struct becomes invalid as soon as **either** reference expires. So in both the single and multi-parameter cases, the struct lives for `min('a, 'b)`.

The reason to use separate parameters is **flexibility in method return types**. When a method returns a reference to one specific field, it can declare which lifetime that return value is tied to:

```rust
impl<'a, 'b> Multi<'a, 'b> {
    fn get_text(&self) -> &'a str {  // tied to 'a specifically, not 'b
        self.text
    }
}

fn main() {
    let text = String::from("hello");
    let result;
    {
        let number = 42;
        let m = Multi { text: &text, number: &number };
        result = m.get_text(); // result has lifetime 'a (tied to text)
    } // number drops here — m is invalid, but result is still ok
    println!("{}", result); // works: 'a (text) is still alive
}
```

With a single `'a`, the compiler would unify both references to the shorter lifetime (`number`'s). `get_text` would return `&'a str` tied to `number`'s lifetime, and the `println!` would fail — even though `text` is still alive.

## Comprehensive example

```rust
struct Excerpt<'a> {
    text: &'a str,
}

impl<'a> Excerpt<'a> {
    // elision rule 3: output lifetime = &self lifetime = 'a
    fn announce(&self, announcement: &str) -> &str {
        println!("Attention: {}", announcement);
        self.text
    }
}

// explicit lifetime: both inputs and output share 'a
// the returned reference lives as long as the shorter of x and y
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

### Cases that break lifetime rules

**Case 1: struct outlives the referenced data**

```rust
let excerpt;
{
    let text = String::from("hello");
    excerpt = Excerpt { text: &text };  // 'a = lifetime of text
}   // text dropped here — 'a ends
println!("{}", excerpt.text);  // ERROR: excerpt used after 'a expired
```

**Case 2: function returns a dangling reference**

```rust
fn broken<'a>() -> &'a str {
    let local = String::from("created inside");
    &local  // ERROR: local is dropped at end of function, 'a cannot be satisfied
}
```

The compiler rejects this because no caller-supplied `'a` can cover a reference to a local variable that is destroyed before the function returns.

**Case 3: `longest` used beyond the shorter lifetime**

```rust
let result;
let s1 = String::from("long string");
{
    let s2 = String::from("xyz");
    result = longest(&s1, &s2);  // 'a unified to shorter of s1 and s2 = s2's lifetime
}   // s2 dropped here
println!("{}", result);  // ERROR: result is tied to 'a which ended with s2
```

Even though `s1` is still alive and might be the one returned, the compiler does not analyze the body of `longest` at the call site — it only sees the signature, which says the return is tied to `'a`, the shorter of the two inputs.

Rust compiler follow three lifetime elision rules, after applying these three rules if the lifetime is still ambiguous, it requires explicit lifetime annotation:

1. Each parameter that is a reference gets its own lifetime parameter.
2. If there is exactly one input lifetime parameter, that lifetime
   is assigned to all output lifetime parameters.
3. If there are multiple input lifetime parameters, but one of them is
   `&self` or `&mut self`, the lifetime of `self` is assigned to all output
   lifetime parameters.


lifetime of function or method parameters are called input lifetimes and lifetime of output values are called output lifetimes. 


video: lets get rusty/048.Struct and lifetime elision