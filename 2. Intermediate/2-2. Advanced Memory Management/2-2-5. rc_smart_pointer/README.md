# Reference Counter

`Rc<T>` (Reference Counted) allows multiple owners of the same heap-allocated value in single-threaded code.

`Rc::clone(&val)` does not deeply clone the data — it just increments the reference count. When an `Rc` is created, the count starts at 1; each `Rc::clone` increments it; each time an `Rc` drops, the count decrements. When the count reaches zero, the value is dropped.

> Prefer `Rc::clone(&val)` over `val.clone()` to make it visually clear you are doing a cheap reference-count bump, not an expensive deep copy.

You can inspect the current count at runtime with `Rc::strong_count(&val)`.

**`Rc` is read-only by default.** It only gives shared *immutable* access. For shared *mutable* access in single-threaded code, combine it with `RefCell<T>` (interior mutability): `Rc<RefCell<T>>`.

**Avoid reference cycles.** Two `Rc`s pointing to each other will never drop, causing a memory leak. Use `Weak<T>` (via `Rc::downgrade`) for non-owning back-references to break cycles.

## Why not `&T` (borrowing)?

```rust
struct AuthService<'a> { db: &'a Database }
struct ContentService<'a> { db: &'a Database }
```

This works in a simple `main` scope, but the moment your architecture grows:

```rust
struct App {
    auth: AuthService,    // compiler asks: what lifetime?
    content: ContentService,
}
```

You are forced into:

```rust
struct App<'a> {
    auth: AuthService<'a>,
    content: ContentService<'a>,
}
```

The lifetime `'a` **bleeds upward** through every struct that contains these services. Every function that touches `App` must carry `'a`. In a real app with many layers, this becomes unmanageable — the annotation infects the entire codebase.

There is also a deeper problem: `Database` must be owned **outside** all the services. You cannot store it inside `App` alongside the services that borrow it — Rust forbids self-referential structs.

## Why not separate `Box<T>` per service?

```rust
struct AuthService    { db: Box<Database> }
struct ContentService { db: Box<Database> }

let auth    = AuthService    { db: Box::new(Database {}) };
let content = ContentService { db: Box::new(Database {}) };
```

Now each service has its **own isolated database instance**. The problem depends on what `Database` represents:

- **Connection pool** — two separate pools means double the connections, double the resources, inconsistent pool state
- **Cache** — two caches go out of sync immediately
- **In-memory state** — mutations in one are invisible to the other
- **File/socket handle** — two handles can conflict and cause locking issues

You lose the **single source of truth**. The whole point of sharing `db` is that both services operate on the **same state**. Two `Box`es give you two independent, isolated copies — that is duplication, not sharing.

## What `Rc` actually solves here

```rust
struct AuthService    { db: Rc<Database> }
struct ContentService { db: Rc<Database> }
```

- Both services point to the **exact same `Database`** in memory — single source of truth
- No lifetime annotations — `App` stays clean with no `'a`
- `Database` drops automatically when **both** services are done with it — no manual lifetime management
- Read-only by default — both services get safe, shared immutable access

### `Rc` is the Rust way of saying: *"I want shared ownership of this resource, and I want the compiler to stop asking me who the real owner is."*

| | `Box<T>` | `&T` | `Rc<T>` |
|---|---|---|---|
| Owners | 1 | 0 (borrows) | Many |
| Lifetime constraint | No | Yes | No |
| Runtime cost | None | None | Ref-count bump |
| Use when | Single owner, heap allocation | Temporary access, clear owner exists | Shared ownership, lifetimes inconvenient |

The `Rc` smart pointer in Rust is similar to `shared_ptr` in C++.

**`Rc` is single-threaded only.** For multi-threaded code use `Arc<T>` (`std::sync::Arc`) — the atomically reference-counted equivalent.

video: Lets get rusty/50. rc smart pointer