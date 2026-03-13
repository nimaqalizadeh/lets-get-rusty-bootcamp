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

A concrete consequence: you cannot mix instances with different lifetimes in the same collection. Once the `Vec` is created with `Excerpt<'a>`, it locks in `'a`. An excerpt tied to a shorter-lived reference is a different type and cannot be pushed in:

```rust
let s1 = String::from("long lived");
let mut v = vec![Excerpt { text: &s1 }]; // Vec<Excerpt<'a>>, 'a = s1's lifetime

{
    let s2 = String::from("short lived");
    v.push(Excerpt { text: &s2 }); // ERROR: s2 doesn't live long enough
}                                  // s2 dropped here, but v is still alive

println!("{}", v[0].text);
```

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

fn main() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let excerpt = Excerpt { text: &novel[..16] };
    let result = excerpt.announce("Breaking news");
    println!("{}", result); // "Call me Ishmael"
}
```

Since the return is tied to `self`'s lifetime (`'a`), `result` becomes invalid the moment `excerpt` is dropped — even if the underlying data (`novel`) is still alive:

```rust
fn main() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let result;
    {
        let excerpt = Excerpt { text: &novel[..16] };
        result = excerpt.announce("Breaking news");
    }   // excerpt dropped here — 'a ends
    println!("{}", result);  // ERROR: result is tied to excerpt's lifetime ('a), which expired
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

**Case 2: mixing struct instances with different lifetimes in a collection**

```rust
let s1 = String::from("long lived");
let mut v = vec![Excerpt { text: &s1 }]; // Vec<Excerpt<'a>>, 'a = s1's lifetime

{
    let s2 = String::from("short lived");
    v.push(Excerpt { text: &s2 }); // ERROR: s2 doesn't live long enough
}   // s2 dropped here, but v is still alive

println!("{}", v[0].text);
```

Once the `Vec` locks in `'a`, any excerpt tied to a shorter lifetime is a different type and cannot be pushed in.

**Case 3: function returning a struct that holds a reference to a local variable**

```rust
fn make_excerpt() -> Excerpt {
    let local = String::from("created inside");
    Excerpt { text: &local }  // ERROR: local is dropped at end of function
}
```

The struct tries to carry a reference out of the function, but the data it points to is destroyed when the function returns.

## Enum lifetimes

Enums work exactly like structs — if a variant holds a reference, the enum needs a lifetime parameter:

```rust
enum Message<'a> {
    Text(&'a str),
    Number(i32),  // no reference — no lifetime needed for this variant
}
```

The lifetime `'a` is attached to the enum type, not to individual variants. So even if only one variant holds a reference, the entire enum instance is constrained by `'a`:

```rust
fn main() {
    let result;
    {
        let s = String::from("hello");
        let msg = Message::Text(&s);  // 'a = s's lifetime
        result = msg;
    }   // s dropped here — 'a ends
    println!("{:?}", result);  // ERROR: result (Message<'a>) used after 'a expired
}
```

## Lifetime bounds on generic types (`T: 'a`)

`T: 'a` means "T must not contain any references shorter than `'a`". When you write `&'a T`, this bound is implied automatically — the compiler knows T must outlive `'a` for the reference to be valid. You only need to write `T: 'a` explicitly when T is stored as an **owned field** (not `&'a T`), but you still have a separate `'a` lifetime in the struct that T's inner references must respect:

```rust
struct Cache<'a, T: 'a> {
    data: T,        // T is owned — compiler cannot infer T: 'a without the explicit bound
    label: &'a str, // separate reference with lifetime 'a
}
```

Without `T: 'a`, the compiler has no way to know that `data`'s inner references must outlive `'a`. With it, the struct is safe: both `label` and any references inside `T` are guaranteed to live at least as long as `'a`.

```rust
let label = String::from("my cache");
let data = String::from("data");          // String owns its data — fine
let c = Cache { data, label: &label };

// would fail:
let c;
{
    let label = String::from("label");
    c = Cache { data: String::from("x"), label: &label }; // ERROR: label dropped too early
}
println!("{}", c.label);
```

## `T: 'static` vs `&'static T`

These look similar but mean different things:

**`&'static T`** — `T` is a type parameter, `'static` is just the lifetime of the reference — same as `&'a T` but with `'a = 'static`. Since `'static` is never shorter than any other lifetime, the reference never expires:

```rust
fn needs_static<T: std::fmt::Debug>(s: &'static T) {
    println!("{:?}", s);
}

needs_static(&42);            // fine — integer literals are 'static

let s = String::from("hello");
needs_static(&s);             // ERROR: &s is tied to s's lifetime, not 'static
                              // s will be dropped at end of scope
```

**`T: 'static`** — T contains no references shorter than `'static`. T itself does not need to live forever — it just must not borrow from anything that could expire:

```rust
fn store<T: 'static>(value: T) {
    // T can be an owned String, i32, Vec<u8>, etc.
    // T cannot be &'a str where 'a is not 'static
}

store(String::from("hello")); // fine — String owns its data, no short-lived refs
store(42);                    // fine — i32 has no references at all

let s = String::from("world");
store(&s);                    // ERROR: &s is tied to s's lifetime, not 'static
```

The key distinction: `&'static T` is about *where the data lives*. `T: 'static` is about *whether T contains any short-lived borrows*.

Rust compiler follow three lifetime elision rules, after applying these three rules if the lifetime is still ambiguous, it requires explicit lifetime annotation:

1. Each parameter that is a reference gets its own lifetime parameter.
2. If there is exactly one input lifetime parameter, that lifetime
   is assigned to all output lifetime parameters.
3. If there are multiple input lifetime parameters, but one of them is
   `&self` or `&mut self`, the lifetime of `self` is assigned to all output
   lifetime parameters.


lifetime of function or method parameters are called input lifetimes and lifetime of output values are called output lifetimes. 


video: lets get rusty/048.Struct and lifetime elision