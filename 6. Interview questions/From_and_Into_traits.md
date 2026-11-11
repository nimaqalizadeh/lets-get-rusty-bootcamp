## From and Into

Let’s go back to where our string journey started:

```rust
let ticket = Ticket::new(
    "A title".into(), 
    "A description".into(), 
    "To-Do".into()
);
```

We now know enough to start unpacking what `.into()` is doing here.

### The problem

This is the signature of the new method:

```rust
impl Ticket {
    pub fn new(
        title: String, 
        description: String, 
        status: String
    ) -> Self {
        // [...]
    }
}
```

We’ve also seen that string literals (such as `"A title"`) are of type `&str`.
We have a type mismatch here: a `String` is expected, but we have a ``&str`. No magical coercion will come to save us this time; we need to perform a conversion.

### From and Into

The Rust standard library defines two traits for **infallible conversions**: `From` and ``Into`, in the `std::convert` module.

```rust
pub trait From<T>: Sized {
    fn from(value: T) -> Self;
}

pub trait Into<T>: Sized {
    fn into(self) -> T;
}
```

These trait definitions showcase a few concepts that we haven’t seen before: supertraits and implicit trait bounds. Let’s unpack those first.

### Supertrait / Subtrait

The `From`: `Sized` syntax implies that `From` is a subtrait of `Sized`: **any type that implements From must also implement `Sized`**. Alternatively, you could say that `Sized` is a supertrait of From.

### Implicit trait bounds

**Every time you have a generic type parameter, the compiler implicitly assumes that it’s `Sized`**.

For example:

```rust
pub struct Foo<T> {
    inner: T,
}
```

is actually equivalent to:

```rust
pub struct Foo<T: Sized> 
{
    inner: T,
}
```

In the case of `From<T>`, the trait definition is equivalent to:

```rust
pub trait From<T: Sized>: Sized {
    fn from(value: T) -> Self;
}
```

In other words, both T and the type implementing `From<T>` must be `Sized`, even though the former bound is implicit.

### Negative trait bounds

You can opt out of the implicit `Sized` bound with a negative trait bound:

```rust
pub struct Foo<T: ?Sized> {
    //            ^^^^^^^
    //            This is a negative trait bound
    inner: T,
}
```

This syntax reads as **“T may or may not be `Sized`”**, and it **allows you to bind `T` to a DST (e.g. `Foo<str>`)**. It is a special case, though: **negative trait bounds are exclusive to `Sized`, you can’t use them with other traits.

> **💡 Side note: why we write `Box<dyn Foo>`**
>
> `dyn Foo` is a DST, just like `str`. Many types can implement `Foo`, each with a different size:
>
> ```rust
> struct A(u8);        // 1 byte
> struct B([u64; 10]); // 80 bytes
> impl Foo for A {}
> impl Foo for B {}
> ```
>
> `dyn Foo` means "some type that implements `Foo`", so the type alone doesn't tell you its size.
>
> **It must sit behind a pointer.** Like `&str`, a `Box<dyn Foo>` is a fat pointer, but the second word is a vtable pointer instead of a length:
>
> ```
> &str         → [ ptr | len ]
> Box<dyn Foo> → [ ptr | vtable ptr ]
> ```
>
> The vtable stores the concrete type's size, its drop function and its method pointers. That's how the right method gets called at runtime.
>
> **Link to implicit `Sized` bounds.** Generics assume `Sized` by default, so `fn f<T>(x: T)` can't take a `dyn Foo`. `Box` works because it opts out with `?Sized`:
>
> ```rust
> pub struct Box<T: ?Sized> { /* ... */ }
> ```
>
> That's what lets `Box<str>`, `Box<[u8]>` and `Box<dyn Foo>` exist.
>
> **`Box` isn't the only option.** Any pointer type that accepts `?Sized` works:
>
> ```rust
> let a: &dyn Foo     = &A(1);          // borrow
> let b: Box<dyn Foo> = Box::new(A(1)); // own
> let c: Rc<dyn Foo>  = Rc::new(A(1));  // share
> ```
>
> `Box` is the usual choice when you need to own the value, for example `Vec<Box<dyn Foo>>` to store different types in one list.

### `&str` to `String`

In `std`’s documentation you can see which `std` types implement the `From` trait.
You’ll find that `String` implements `From<&str>` for `String`. Thus, we can write:

```rust
let title = String::from("A title");
```

We’ve been primarily using `.into()`, though.
If you check out the implementors of `Into` you **won’t** find `Into<String> for &str`. What’s going on?

`From` and `Into` are dual traits.
In particular, `Into` is implemented for any type that implements `From` using a blanket implementation:

```rust
impl<T, U> Into<U> for T
where
    U: From<T>,
{
    fn into(self) -> U {
        U::from(self)
    }
}
```

If a type `U` implements `From<T>`, then `Into<U> for T` is automatically implemented. That’s why we can write `let title = "A title".into();`.

> **💡 Side note: reading the blanket impl, and why `Into` exists**
>
> **Reading `U: From<T>`:**
> - `T` is the **source** type (what we have).
> - `U` is the **destination** type (what we want).
> - `U: From<T>` means `U` implements `From<T>`: `U` knows how to build itself from a `T`.
> - `from(value: T) -> Self` takes a `T` and returns `Self`, which here is `U`.
> - `into(self)` just calls `U::from(self)`. It adds no conversion logic.
>
> Example: `"A title".into()` → `String`: `T = &str`, `U = String`, and `.into()` runs `String::from("A title")`.
>
> **So `Into` is a wrapper around `From`, written in the other direction:**
> - `From`: `U::from(t)`. You name the destination type.
> - `Into`: `t.into()`. You call it on the value, and the destination is inferred from context.
>
> **Why `Into` is still useful:**
>
> 1. **Method syntax with inference**, so you don't have to name the target type:
>
>    ```rust
>    Ticket::new("A title".into(), ...) // compiler infers String from the signature
>    ```
>
> 2. **Flexible function parameters**, a common pattern in real code:
>
>    ```rust
>    fn set_title(title: impl Into<String>) {
>        let title: String = title.into();
>    }
>
>    set_title("literal");             // &str
>    set_title(String::from("owned")); // String
>    ```
>
>    The same thing with `From` would be clumsier (`where String: From<T>`).
>
> **Rule of thumb:** implement `From`, never `Into` directly, because you get `Into` for free from the blanket impl. In bounds, use `Into` when **accepting** values, because it's more flexible.


### `.into()`

Every time you see `.into()`, you’re witnessing a conversion between types.
What’s the target type, though?

In most cases, the target type is either:

- Specified by the signature of a function/method (e.g. `Ticket::new` in our example above)
- Specified in the variable declaration with a type annotation (e.g. `let title: String = "A title".into()`;)

`.into()` will work out of the box as long as the compiler can infer the target type from the context without ambiguity.

---

کلمه **Infallible** در زبان فارسی به معنی **«خطاناپذیر»**، **«بدون خطا»** یا **«تضمین‌شده»** است.

در برنامه‌نویسی و زبان Rust (متنی که فرستادید)، عبارت **infallible conversions** به معنی **«تبدیل‌های خطاناپذیر»** است.

این یعنی وقتی از `From` یا `Into` برای تبدیل یک نوع داده (Type) به نوعی دیگر استفاده می‌کنید، این عملیات **صد در صد تضمین‌شده است و هرگز شکست نمی‌خورد**. به زبان ساده‌تر، در حین انجام این تبدیل هیچ‌گونه خطایی (Error) رخ نمی‌دهد و برنامه متوقف (Panic) نمی‌شود.

برای درک بهتر:

* تبدیل یک عدد کوچک مثل `u8` به یک فضای بزرگتر مثل `u32` همیشه و بدون استثنا ممکن است (**infallible**).
* اما تبدیل یک فایل متنی به یک عدد ممکن است با حروف نامربوط مواجه شود و شکست بخورد، که به این نوع تبدیل‌ها در Rust تبدیل‌های خطاپذیر یا **fallible** می‌گویند (که برای آن‌ها از `TryFrom` و `TryInto` استفاده می‌شود).