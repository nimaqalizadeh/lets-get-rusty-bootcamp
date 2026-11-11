# Generics and associated types

Let’s re-examine the definition for two of the traits we studied so far, `From` and `Deref`:

```rust
pub trait From<T> {
    fn from(value: T) -> Self;
}

pub trait Deref {
    type Target;
    
    fn deref(&self) -> &Self::Target;
}
```

They both feature type parameters.
In the case of `From`, it’s a generic parameter, `T`.
In the case of `Deref`, it’s an associated type, Target.

### What’s the difference? Why use one over the other?

#### At most one implementation

Due to how `deref` coercion works, there can only be one “target” type for a given type. E.g. String can only `deref` to `str`. **It’s about avoiding ambiguity**: if you could implement `Deref` multiple times for a type, which Target type should the compiler choose when you call a `&self` method?

That’s why `Deref` uses an associated type, Target.
An associated type is uniquely determined by the trait implementation. Since **you can’t implement `Deref` more than once, you’ll only be able to specify one Target for a given type and there won’t be any ambiguity**.

## Generic traits

On the other hand, you can implement `From` multiple times for a type, as long as the input type `T` is different. For example, you can implement `From` for `WrappingU32` using both `u32` and `u16` as input types:

```rust
impl From<u32> for WrappingU32 {
    fn from(value: u32) -> Self {
        WrappingU32 { inner: value }
    }
}

impl From<u16> for WrappingU32 {
    fn from(value: u16) -> Self {
        WrappingU32 { inner: value.into() }
    }
}
```

This works because `From<u16>` and `From<u32>` are considered different traits.
There is no ambiguity: the compiler can determine which implementation to use based on type of the value being converted.

### Case study: Add

As a closing example, consider the `Add` trait from the standard library:

```rust
pub trait Add<RHS = Self> {
    type Output;
    
    fn add(self, rhs: RHS) -> Self::Output;
}
```

It uses both mechanisms:

- it has a generic parameter, RHS (right-hand side), which defaults to Self
- it has an associated type, Output, the type of the result of the addition

#### What does `RHS = Self` mean?

> `RHS = Self` is a **default**, not a requirement.
>
> - If you write `impl Add for Point` without a type parameter, `RHS` becomes `Self`, so you add a `Point` to a `Point`.
> - You can still pick a different right-hand side type by writing it out:
>
> ```rust
> struct Millimeters(u32);
> struct Meters(u32);
>
> impl Add<Meters> for Millimeters {
>     type Output = Millimeters;
>
>     fn add(self, rhs: Meters) -> Millimeters {
>         Millimeters(self.0 + rhs.0 * 1000)
>     }
> }
> ```
>
> Here `Self` is `Millimeters` and `RHS` is `Meters`, so the two sides have different types.
>
> **In short:** `RHS` is the type on the right-hand side of `+`. If you don't specify it, it defaults to the implementing type (`Self`).

### RHS

RHS is a generic parameter to allow for different types to be added together.
For example, you’ll find these two implementations in the standard library:

```rust
impl Add<u32> for u32 {
    type Output = u32;
    
    fn add(self, rhs: u32) -> u32 {
      //                      ^^^
      // This could be written as `Self::Output` instead.
      // The compiler doesn't care, as long as the type you
      // specify here matches the type you assigned to `Output` 
      // right above.
      // [...]
    }
}

impl Add<&u32> for u32 {
    type Output = u32;
    
    fn add(self, rhs: &u32) -> u32 {
        // [...]
    }
}
```

This allows the following code to compile:

```rust
let x = 5u32 + &5u32 + 6u32;
```

because `u32` implements `Add<&u32>` as well as `Add<u32>`.

## Output

Output represents the type of the result of the addition.

Why do we need Output in the first place? Can’t we just use Self as output, the type implementing Add? We could, but it would limit the flexibility of the trait. In the standard library, for example, you’ll find this implementation:

impl Add<&u32> for &u32 {
    type Output = u32;

    fn add(self, rhs: &u32) -> u32 {
        // [...]
    }
}

The type they’re implementing the trait for is &u32, but the result of the addition is u32.
It would be impossible1 to provide this implementation if add had to return Self, i.e. &u32 in this case. Output lets std decouple the implementor from the return type, thus supporting this case.

#### What does `Self::Output` mean?

> `Self::Output` means "the implementor's `Output` type".
>
> - `Self` is still the implementor type, the same `Self` as in `RHS = Self`.
> - `::Output` is the associated type that the implementor sets in its `impl`.
> - The implementor can set it to any type. It doesn't have to be `Self`.
>
> ```rust
> impl Add<&u32> for &u32 {
>     type Output = u32;   // the implementor (&u32) sets Output to u32
>
>     fn add(self, rhs: &u32) -> Self::Output { ... }
>     //                         ^^^^^^^^^^^^
>     // Self = &u32, so Self::Output = u32
> }
> ```
>
> **In short:** `Self::Output` goes through `Self` (the implementor), but its value can be any type the implementor chooses.

On the other hand, Output can’t be a generic parameter. The output type of the operation must be uniquely determined once the types of the operands are known. That’s why it’s an associated type: for a given combination of implementor and generic parameters, there is only one Output type.
Conclusion

### To recap:

- Use an associated type when the type must be uniquely determined for a given trait implementation.
- Use a generic parameter when you want to allow multiple implementations of the trait for the same type, with different input types.

---
کلمه **Associated** در لغت به معنی **«مرتبط»**، **«وابسته»** یا **«پیوسته»** است.

دلیل اینکه در زبان راست (Rust) به این ویژگی **Associated Type (نوع وابسته/مرتبط)** می‌گویند، این است که این نوع داده به طور جدانشدنی به پیاده‌سازیِ همان تریت (Trait) برای یک ساختار خاص **گره خورده است**.

برای درک چراییِ استفاده از این کلمه، تفاوت آن را با جنریک‌ها ببینید:

* **جنریک‌ها (Generics) مستقل هستند:** مثل تنظیماتی هستند که از بیرون به تریت می‌دهید. برنامه‌نویس موقع استفاده تصمیم می‌گیرد چه نوعی را به آن‌ها پاس بدهد.
* **انواع وابسته (Associated Types) مرتبط به درون هستند:** از بیرون تزریق نمی‌شوند. وقتی شما تریت `Add` را برای یک نوع خاص (مثل `&u32`) پیاده‌سازی می‌کنید، نوع `Output` دقیقاً در همان لحظه و در داخل همان بلوک کد تعیین می‌شود. یعنی نوع خروجی، مستقیماً به آن پیاده‌سازیِ خاص **وابسته (Associated)** است و متعلق به آن است.

پس وقتی می‌گوییم Associated Type، منظورمان نوع داده‌ای است که در دلِ یک پیاده‌سازی تعریف می‌شود و سرنوشتش به همان پیاده‌سازی **گره خورده (مرتبط)** است، نه اینکه از بیرون و به صورت دلخواه تغییر کند.
