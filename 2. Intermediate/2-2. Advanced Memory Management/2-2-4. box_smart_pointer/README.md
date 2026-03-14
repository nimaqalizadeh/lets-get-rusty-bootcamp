# Box

`Box` smart pointer is included in prelude

`Box` smart pointer is similar to unique smart pointer (`unique_ptr`) in C++ . It gives you single ownership of something stored on the heap. There are a few usecases for the `Box` smart pointer:

## Stack vs Heap allocation

```rust
let button_a = Button { text: "button a".to_owned() };
let button_b = Box::new(Button { text: "button b".to_owned() });
```

- `button_a` — the `Button` struct is stored **directly on the stack**. When `button_a` goes out of scope, it is immediately dropped.
- `button_b` — only the **pointer** (8 bytes) lives on the stack. The actual `Button` struct is allocated on the **heap**. `Box` owns that heap memory and frees it when `button_b` goes out of scope.

```
Stack                  Heap
──────────────────     ──────────────────
button_a: Button {}
button_b: ptr ─────────► Button {}
```

Usecases of `Box`:
1. Avoid copying large amounts of data when transfering ownership. When transfering ownership, data is copied around on the stack to so in transfering ownership of `button_a` the entire struct will copy on the stack, but in the case of `button_b` only the smart pointer will copy on the stack and the the data remains untouched on the heap. 

2. In combination with trait objects.

A trait object (`dyn Trait` in other word `imple` in return value) has  unknown size at compile time — different types implementing the same trait can have different sizes. Because the stack requires fixed-size values, Rust won't let you store a `dyn Trait` directly. Wrapping it in `Box<dyn Trait>` puts the concrete value on the heap, and only the fixed-size pointer lives on the stack.

```rust
let components: Vec<Box<dyn UIComponent>> = vec![
    Box::new(button_c),  // Button wrapped in Box at the call site
    button_d,            // already a Box<Button>, coerces to Box<dyn UIComponent>
    Box::new(Container { // Container also implements UIComponent
        name: "panel".to_owned(),
        child: Some(Box::new(Container {
            name: "inner".to_owned(),
            child: None, // leaf node — no more children
        })),
    }),
];
```

Here `Button` and `Container` both implement `UIComponent`, but they are different sizes. By storing `Box<dyn UIComponent>`, each element is a same-sized pointer regardless of the concrete type behind it.

> **Note on `Container`:** The `child: Box<Container>` field is a recursive type — a `Container` that always owns another `Container`, with no way to stop. `Box` is required here because without it the size of `Container` would be infinite at compile time. However, the current definition forces every `Container` to have a child, which makes it impossible to create a leaf node. A better design would be `child: Option<Box<Container>>`, where `None` terminates the tree.

3. When we have a type of unknown size and want to use it in a context where the exact size is required. An example of this is recursive types.

`Container` holds a `child` of type `Box<Container>` — a `Container` inside a `Container`. Without `Box`, Rust would try to compute the size of `Container` by including the size of another `Container` inside it, which includes another, and so on — infinite size. `Box` breaks this by storing the child on the heap; the field becomes a fixed-size pointer, so Rust can compute the size of `Container` at compile time.

Common usecases for recursive types:

**1. Tree** — a node that holds child nodes of the same type (e.g. a UI component tree, a file system directory).
```rust
struct TreeNode {
    value: i32,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}

let tree = TreeNode {
    value: 1,
    left: Some(Box::new(TreeNode {
        value: 2,
        left: Some(Box::new(TreeNode {
            value: 4,
            left: Some(Box::new(TreeNode { value: 8, left: None, right: None })),
            right: Some(Box::new(TreeNode { value: 9, left: None, right: None })),
        })),
        right: Some(Box::new(TreeNode { value: 5, left: None, right: None })),
    })),
    right: Some(Box::new(TreeNode {
        value: 3,
        left: Some(Box::new(TreeNode { value: 6, left: None, right: None })),
        right: Some(Box::new(TreeNode { value: 7, left: None, right: None })),
    })),
};
```

**2. Linked list** — each node holds a value and an optional pointer to the next node.
```rust
enum List {
    Cons(i32, Box<List>),
    Nil,
}

let list = List::Cons(1, Box::new(List::Cons(2, Box::new(List::Nil))));
```

**3. JSON-like value** — a value that can itself contain nested values of the same kind.
```rust
enum Json {
    Number(f64),
    Str(String),
    Array(Vec<Box<Json>>),
}

let json = Json::Array(vec![
    Box::new(Json::Number(1.0)),
    Box::new(Json::Str("hello".to_owned())),
    Box::new(Json::Array(vec![Box::new(Json::Number(2.0))])),
]);
// the shpe is [1, "hello", [2]]
```

In all these cases the depth is only known at runtime, so the data must live on the heap via `Box`.

