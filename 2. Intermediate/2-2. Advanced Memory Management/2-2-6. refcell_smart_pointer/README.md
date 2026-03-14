# RefCell smart pointer

The `Rc` smart pointer only allows immutable shared ownership of a value. We can get around this by using the `RefCell` smart pointer.

## Interior Mutability

`RefCell<T>` uses the **interior mutability** design pattern, which allows mutably borrowing data even when there is an immutable reference to that data. This breaks the compile-time Rust borrowing rules, so `unsafe` code is used internally — but that unsafe code is wrapped in a safe public API.

The advantage of this pattern is that it allows certain memory-safe scenarios which would otherwise be disallowed. Rust's ownership and borrowing rules are conservative by nature. Sometimes you as a developer know that something is memory safe but cannot prove it to the compiler. Interior mutability exists to get around those limitations when needed.

## Borrow Rules at Runtime

`RefCell<T>` enforces the same borrowing rules as Rust normally does — but at **runtime** instead of compile time:

| Allowed | Not Allowed |
|--------|-------------|
| Multiple immutable borrows (`borrow()`) at once | Mutable + any other borrow at the same time |
| One mutable borrow (`borrow_mut()`) at a time | Two simultaneous `borrow_mut()` calls |

If you break these rules, `RefCell` will **panic at runtime** instead of giving a compile error:

```rust
let mut r1 = db.borrow_mut();
let r2 = db.borrow_mut(); // panics: already mutably borrowed
r1.max_connections = 200;
```

## Rc\<RefCell\<T\>\> — Shared Ownership with Mutability

One of the most common use cases for `RefCell` is combining it with `Rc` to get **shared ownership and mutability** at the same time:

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct Database {
    max_connections: u32,
    connected_users: Vec<String>,
}

struct AuthService {
    db: Rc<RefCell<Database>>,
}

struct ContentService {
    db: Rc<RefCell<Database>>,
}

fn main() {
    let db = Rc::new(RefCell::new(Database {
        max_connections: 100,
        connected_users: vec![],
    }));

    let auth_service = AuthService { db: Rc::clone(&db) };
    let content_service = ContentService { db: Rc::clone(&db) };

    // AuthService mutates the shared database
    auth_service.db.borrow_mut().connected_users.push("alice".to_string());

    // ContentService reads the same database
    let db_ref = content_service.db.borrow();
    println!("Users: {:?}", db_ref.connected_users); // ["alice"]
}
```

Both `AuthService` and `ContentService` hold an `Rc` clone pointing to the **same** `Database`. `RefCell` allows either service to mutate it — one at a time.

## Immutable Borrows Can Coexist

Multiple immutable borrows are perfectly fine simultaneously, just like with regular references:

```rust
let r1 = db.borrow();
let r2 = db.borrow();
println!("{} and {}", r1.max_connections, r2.max_connections); // fine
```

## Other Use Cases

The common theme across all `RefCell` use cases is: **you hold a `&self` (immutable) reference but need internal mutation**. `Rc` is just one way you end up in that situation.

### Memoization / Caching

Memoization is an optimization technique where you cache the result of a function call so that if the same inputs are given again, you return the cached result instead of recomputing it. The name comes from "memo" — you're writing a note to yourself so you don't have to figure it out again.

Without memoization, `fibonacci(50)` makes ~2²⁵ recursive calls. With memoization it makes exactly 50:

```rust
// Without memoization — recomputes every time
fn fibonacci(n: u64) -> u64 {
    if n <= 1 { return n; }
    fibonacci(n - 1) + fibonacci(n - 2) // called exponentially many times
}

// With memoization — computes once, reuses the result
fn fibonacci(n: u64, cache: &mut HashMap<u64, u64>) -> u64 {
    if n <= 1 { return n; }
    if let Some(&v) = cache.get(&n) {
        return v; // already computed, return immediately
    }
    let result = fibonacci(n - 1, cache) + fibonacci(n - 2, cache);
    cache.insert(n, result); // store for future calls
    result
}
```

The naive solution of passing `&mut HashMap` leaks the implementation detail to every caller — they are forced to own and manage the cache themselves:

```rust
let mut cache = HashMap::new();
fibonacci(50, &mut cache); // caller is forced to manage the cache
```

When wrapped in a struct, it gets worse. To mutate `cache` through `self`, the method must take `&mut self`:

```rust
struct Fibonacci {
    cache: HashMap<u64, u64>,
}

impl Fibonacci {
    fn compute(&mut self, n: u64) -> u64 { // forced to be &mut self
        if n <= 1 { return n; }
        if let Some(&v) = self.cache.get(&n) {
            return v;
        }
        let result = self.compute(n - 1) + self.compute(n - 2);
        self.cache.insert(n, result);
        result
    }
}
```

This causes two real problems:

**1. Traits that define the method as `&self` become impossible to implement**

Traits don't always require `&self` — that's a design choice made by whoever defines the trait. A trait method can be defined with `&self`, `&mut self`, or `self` (owned). However, trait authors often choose `&self` following the **principle of least restriction** — a `&self` method imposes fewer requirements on the caller, making the trait usable in more situations:

- A `&self` method can be called on shared references, behind `Rc`, behind `Arc`, etc.
- A `&mut self` method requires the caller to have exclusive mutable access, which rules out all those scenarios.

So if the trait author knows the method is logically a "read" operation, they'll define it as `&self` — even if a particular implementation happens to need internal mutation for caching. A real-world example is `std::hash::Hash`:

```rust
pub trait Hash {
    fn hash<H: Hasher>(&self, state: &mut H); // &self — hashing is a read
}
```

If it were `&mut self`, you couldn't hash something inside an `Rc` or behind a shared reference.

If you want your `Fibonacci` struct to implement a trait that defines `compute` as `&self`, the compiler rejects it because your implementation requires `&mut self`:

```rust
trait Compute {
    fn compute(&self, n: u64) -> u64; // trait requires &self
}

impl Compute for Fibonacci {
    fn compute(&mut self, n: u64) -> u64 { // compile error: mismatched signature
        if n <= 1 { return n; }
        if let Some(&v) = self.cache.get(&n) {
            return v;
        }
        let result = self.compute(n - 1) + self.compute(n - 2);
        self.cache.insert(n, result);
        result
    }
}
```

**2. You can't share `&mut` references**
If two parts of your code hold a reference to the same `Fibonacci`, only one can be `&mut` at a time — the other is blocked.

`RefCell` solves both problems. The method stays `&self` (satisfying any trait), mutation is handled internally, and the cache remains a hidden implementation detail that callers never have to manage:

```rust
trait Compute {
    fn compute(&self, n: u64) -> u64; // trait requires &self
}

struct Fibonacci {
    cache: RefCell<HashMap<u64, u64>>,
}

impl Compute for Fibonacci {
    fn compute(&self, n: u64) -> u64 { // &self — trait is now satisfied
        if n <= 1 { return n; }
        if let Some(&v) = self.cache.borrow().get(&n) {
            return v;
        }
        let result = self.compute(n - 1) + self.compute(n - 2);
        self.cache.borrow_mut().insert(n, result);
        result
    }
}
```

The only differences from the `&mut self` version are: `&self` instead of `&mut self`, `RefCell` wrapping the cache field, and `borrow()`/`borrow_mut()` wrapping the cache accesses. The logic is identical.

### Mock Objects in Tests

**The scenario:** the `Logger` trait's job is to write a log message somewhere — to a file, to the terminal, etc. The `login()` function accepts any logger that implements the trait and calls it when login succeeds:

```rust
trait Logger {
    fn log(&self, msg: &str);
}

fn login(user: &str, logger: &dyn Logger) {
    // ... login logic ...
    logger.log("user logged in");
}
```

In production you'd pass a real logger that writes to a file or prints to the terminal. But in a test you want to verify that `login()` actually called `log()` with the right message. A real logger just prints and the output is gone — you can't check it programmatically.

So instead you pass a `MockLogger` — a fake logger that saves every message into a `Vec` instead of printing it. After `login()` returns, you inspect the vec to confirm the right messages were logged. You are not testing the logger — you are testing that `login()` used the logger correctly.

**The problem:** `login()` accepts the logger as `&dyn Logger` — an immutable reference. So when it calls `logger.log(...)`, Rust uses `&self`. The `MockLogger` must implement `log` with `&self` to satisfy the trait, but it needs to `push` into the vec — which is mutation. You can't mutate through `&self` by default.

**`RefCell` solves it:** `login()` thinks it has an immutable logger, but `RefCell` secretly allows mutation inside at runtime:

```rust
struct MockLogger {
    logs: RefCell<Vec<String>>,
}

impl Logger for MockLogger {
    fn log(&self, msg: &str) {                         // &self — satisfies the trait ✓
        self.logs.borrow_mut().push(msg.to_string());  // mutation happens inside via RefCell
    }
}
```

**After the test**, inspect what was recorded:
```rust
let mock = MockLogger { logs: RefCell::new(vec![]) };
login("alice", &mock);
assert_eq!(*mock.logs.borrow(), vec!["user logged in"]); // ✓
```

### Lazy Initialization

Defer expensive setup until the value is first accessed:

```rust
struct Config {
    data: RefCell<Option<String>>,
}

impl Config {
    fn get(&self) -> String {
        if self.data.borrow().is_none() { // shared borrow created and immediately dropped after this expression
            *self.data.borrow_mut() = Some(load_from_disk()); // safe: no shared borrow is alive
        }
        self.data.borrow().clone().unwrap()
    }
}
```

**Idiomatic rule:** always use `borrow()` for reads and `borrow_mut()` only when actually writing. Even though using `borrow_mut()` for a read-only check would technically work (the temporary is still dropped), it misleadingly signals mutation intent.

**Wrong alternative — keeping the borrow alive with `let`:**

```rust
fn get(&self) -> String {
    let check = self.data.borrow();     // shared borrow kept alive in `check`
    if check.is_none() {
        *self.data.borrow_mut() = ...;  // PANIC: can't mutably borrow while `check` is still alive
    }
    check.clone().unwrap()
}
```

`check` holds the shared borrow for the entire scope, so `borrow_mut()` panics at runtime.

#### Why `get` takes `&self` instead of `&mut self`

The whole point of this pattern is that `get` *looks* read-only from the outside.

If you used `&mut self`:
- Callers would need a mutable reference to `Config`
- You couldn't call `get` on a shared `Rc<Config>` or any shared reference
- The struct would feel "mutable" to users, leaking the implementation detail that it does lazy loading

With `&self` + `RefCell`, callers treat `Config` as immutable — the internal mutation (loading data) is hidden as an implementation detail. This is the core idea behind **interior mutability**: the *outside* sees immutability, the *inside* can still mutate when needed.

#### Why use this pattern at all

Lazy initialization earns its keep when `load_from_disk()` is expensive. If `Config` is created but `get` is never called, you've wasted nothing — you only pay the cost when the value is actually needed.

**Real-world scenarios:** reading config files, opening database connections, parsing large files, making network requests.

| | Eager | Lazy (`RefCell`) |
|---|---|---|
| Load time | At construction | On first access |
| Wasted work | Possible | Never |
| Complexity | Simple | Slightly more complex |

Use lazy init when construction is frequent but the value might not always be needed, or when startup time matters and you want to defer heavy work.

## When to Use RefCell

- You need **interior mutability** — mutation through a shared (`&`) reference.
- You are using `Rc<T>` and need multiple owners that can also mutate the value.
- A trait or API forces `&self` but you need to mutate internal state (caching, logging, lazy init).
- You can reason that the borrow rules are respected at runtime, but the compiler cannot verify it statically.

`RefCell` should be used with caution because the **responsibility of following the ownership rules is on the programmer**. A violation will not be caught at compile time — it will panic at runtime.

```rust
let cell = RefCell::new(5);
let b1 = cell.borrow();
let b2 = cell.borrow_mut(); // PANIC: already borrowed as shared
```

If you want to handle borrow violations gracefully instead of panicking, use the `try_` variants which return a `Result`:

```rust
let cell = RefCell::new(5);
let b1 = cell.borrow();
match cell.try_borrow_mut() {
    Ok(mut val) => *val += 1,
    Err(e) => println!("couldn't borrow mutably: {}", e), // handled, no panic
}
```

| Method | On violation |
|---|---|
| `borrow()` / `borrow_mut()` | Panics at runtime |
| `try_borrow()` / `try_borrow_mut()` | Returns `Err`, no panic |
