# Recoverable Errors

## Ways to Match / Handle a `Result<T, E>`

### 1. `match` expression — full control

```rust
let file = match File::open("example.txt") {
    Ok(file) => file,
    Err(error) => panic!("Failed to open file: {:?}", error),
};
```

Both arms must return the same type — but `panic!` is an exception. It has the
type `!` (the **never type**), which means "this code never returns at all". The
compiler treats `!` as compatible with any type, so the `Err` arm satisfies the
`File` type requirement even though it never actually produces a `File`.

**What if you don't want to panic?** Use one of these instead:

**1. Return a default value of the same type**
```rust
// parse_input() returns Result<String, _>
let text = match parse_input() {
    Ok(val) => val,                     // val is a String
    Err(_)  => String::from("default"), // also a String — both arms match
};
```

**2. Propagate the error up to the caller with `?`**
```rust
fn run() -> Result<File, io::Error> {
    let file = File::open("example.txt")?; // caller decides what to do
    Ok(file)
}
```

**3. Use `unwrap_or` / `unwrap_or_else` for a fallback**
```rust
let file = File::open("example.txt").unwrap_or_else(|e| {
    println!("Using default because: {e}");
    File::open("default.txt").unwrap() // open fallback file — if this also fails, panic
                                       // (acceptable: if even the fallback is missing,
                                       //  something is seriously wrong)
});
```

> `panic!` is for situations that **should never happen** (programmer bugs).
> For expected failures like missing files, bad input, or network errors —
> use `Result` and handle it gracefully.

---

### 2. `if let` — only care about one variant

```rust
// Only handle Ok
if let Ok(file) = File::open("example.txt") {
    println!("Opened: {:?}", file);
}

// Only handle Err
if let Err(e) = File::open("example.txt") {
    println!("Error: {:?}", e);
}
```

Cleaner than `match` when you only care about one arm.

---

### 3. `while let` — loop while Ok (or while Err)

```rust
while let Ok(line) = reader.read_line(&mut buf) {
    // process line...
}
```

Stops the loop as soon as the pattern no longer matches. When `read_line` returns `Err`, the pattern doesn't match and the loop stops silently — **the error is dropped and ignored**. You never know if the loop stopped because the file ended normally or because an actual IO error occurred.

The safer approach is a regular `loop` with a full `match`:

```rust
loop {
    match reader.read_line(&mut buf) {
        Ok(0)  => break,             // end of file
        Ok(_)  => { /* process */ }  // got a line
        Err(e) => {                  // actual error
            println!("Error: {e}");
            break;
        }
    }
}
```

> Use `while let` only when you genuinely don't care about errors — like iterating over an `Option` or a known-safe iterator.

---

### 4. `unwrap()` — panic on `Err`, return inner value on `Ok`

```rust
let file = File::open("example.txt").unwrap();
```

Use only in tests or throwaway code. Panics with a generic message on `Err`.

---

### 5. `expect(msg)` — like `unwrap` with a custom panic message

```rust
let file = File::open("example.txt").expect("Failed to open file!");
```

Preferred over `unwrap` when you want a meaningful panic message.

---

### 6. `unwrap_or(default)` — fallback value on `Err`

Both `unwrap_or` and `unwrap_or_else` do the same thing:
- If `Ok(v)` → return `v`
- If `Err` → return a fallback value instead of panicking

They are a safe alternative to `unwrap()` — instead of crashing on error, you get a default value.

`unwrap_or` takes a **fixed fallback value** directly:

```rust
let value: Result<i32, &str> = Err("oops");
let n = value.unwrap_or(0);
// value is Err → n gets the fallback: 0

let value: Result<i32, &str> = Ok(42);
let n = value.unwrap_or(0);
// value is Ok → n gets the inner value: 42
```

---

### 7. `unwrap_or_else(|e| ...)` — compute fallback lazily via closure

`unwrap_or_else` takes a **closure** instead of a fixed value. The closure only runs **if there is an error** — making it lazy.

```rust
let value: Result<i32, &str> = Err("oops");
let n = value.unwrap_or_else(|e| {
    println!("Got error: {e}"); // only runs on Err
    -1                          // this becomes the fallback value
});
// n == -1
```

> **Why lazy?** When you write `unwrap_or(heavy_computation())`, Rust evaluates
> **all arguments before calling the function**. So `heavy_computation()` runs first
> and its result is computed — then `unwrap_or` is called with that result.
> This happens regardless of whether the value is `Ok` or `Err`. It's the same as:
>
> ```rust
> let fallback = heavy_computation(); // runs no matter what
> value.unwrap_or(fallback);          // now decides Ok or Err
> ```
>
> `unwrap_or_else` fixes this — the closure is not pre-evaluated as an argument.
> It only executes **inside** `unwrap_or_else` when there actually is an error.

- Use `unwrap_or` for cheap fallbacks like `0`, `""`, `false`
- Use `unwrap_or_else` for expensive fallbacks like DB calls, file reads, or logging

---

### 8. `map(|v| ...)` — transform `Ok` value, pass `Err` through

**"Transform"** = change the value inside `Ok` to something else.
**"Pass through"** = don't touch it, return it as-is.

`get_name()` returns `Result<String, io::Error>`. Sometimes you want to transform the success value — for example convert it to uppercase or parse it — **without touching the `Err`**. `map` lets you do that.

> Think of a box that is either green (Ok) or red (Err).
> `map` only opens the **green box**, swaps what's inside, and reseals it.
> The red box is never touched.

```
Result<String, io::Error>
         ↓  map(|s| s.to_uppercase())
Result<String, io::Error>  ← Ok value changed, Err untouched
```

```rust
let result = get_name().map(|s| s.to_uppercase());

// If Ok("alice")  → returned as Ok("ALICE")
// If Err(e)       → returned unchanged as Err(e)
```

---

### 9. `map_err(|e| ...)` — transform `Err` value, pass `Ok` through

The vice versa of `map` — only opens the **red box (Err)**, transforms what's inside, and leaves the **green box (Ok) untouched**.

`File::open` returns `Result<File, io::Error>`. Sometimes you want the error to be a `String` instead — for example to display it or match your function's return type. `map_err` lets you convert the error type **without touching the `Ok` value**.

```
Result<File, io::Error>
         ↓  map_err(|e| format!("IO error: {e}"))
Result<File, String>     ← error type changed, File untouched
```

```rust
let result: Result<File, String> = File::open("example.txt")
    .map_err(|e| format!("IO error: {e}"));

// If Ok(file)  → returned unchanged as Ok(file)
// If Err(e)    → e is converted to a String, returned as Err("IO error: ...")
```

---

### 10. `and_then(|v| ...)` — chain fallible operations (flatMap)

The problem it solves: when you use `map` and the closure itself returns a `Result`,
you end up with a nested `Result<Result<T, E>, E>` — which is awkward to work with.

**With `map` — gets nested:**

```rust
let result = File::open("example.txt")
    .map(|file| read_to_string(file)); // read_to_string also returns Result

// result type: Result<Result<String, io::Error>, io::Error>  ← nested, ugly
```

**With `and_then` — stays flat:**

```rust
let result = File::open("example.txt")
    .and_then(|file| read_to_string(file)); // flattens the nested Result

// result type: Result<String, io::Error>  ← clean
```

Think of it as **"if the first step succeeded, try the next step"**:

```
File::open succeeds  →  run read_to_string  →  return its Result
File::open fails     →  skip read_to_string →  return the Err immediately
```

**Rule of thumb:**
- Use `map` when your closure returns a **plain value** (`String`, `i32`, ...)
- Use `and_then` when your closure returns a **`Result`** (another fallible operation)

---

### 11. `?` operator — pass the error to the caller

`?` means: **"I'm not handling this error here — give it to whoever called me."**

It is shorthand for this `match`:

```rust
match result {
    Ok(val) => val,           // unwrap and continue
    Err(e)  => return Err(e), // stop this function, give error to caller
}
```

> `return Err(e)` does **not** mean this function is handling the error.
> It means this function is **refusing to handle it** — it exits immediately
> and hands the error upward to whoever called it.
>
> ```
> File::open fails
>   → returns Err to open_and_read
>       → open_and_read does `return Err(e)`  ← refusing, passing it up
>           → Err travels up to main()
>               → main() finally handles it
> ```

**Example — without `?`:**

```rust
fn open_and_read() -> Result<String, io::Error> {
    let file = match File::open("example.txt") {
        Ok(f)  => f,
        Err(e) => return Err(e), // stop, give error to caller
    };
    let content = match read_to_string(file) {
        Ok(s)  => s,
        Err(e) => return Err(e), // stop, give error to caller
    };
    Ok(content)
}
```

**Same thing — with `?`:**

```rust
fn open_and_read() -> Result<String, io::Error> {
    let file    = File::open("example.txt")?; // Err → return to caller immediately
    let content = read_to_string(file)?;      // Err → return to caller immediately
    Ok(content)
}
```

**Who should handle the error?**

`?` moves the error *up the call stack* until it reaches a function that can decide what to do with it:

```
main()               ← must handle it here (no one above)
  └─ open_and_read() ← uses ? to pass it up
       └─ File::open ← uses ? to pass it up
```

- Middle functions use `?` — they don't have enough context to react.
- The outermost caller (usually `main`) uses `match` or `unwrap_or_else` — it knows whether to log, retry, show a message, or exit.

**From the Rust Reference:**

The official name is the **"try propagation expression"**. It is based on the `Try` trait and works on more than just `Result`:

| Type | On success | On failure |
|---|---|---|
| `Result<T, E>` | evaluates to `T` | returns `Err(From::from(e))` |
| `Option<T>` | evaluates to `T` | returns `None` |
| `ControlFlow<B, C>` | evaluates to `C` | returns `Break(b)` |

> Note: when propagating a `Result`, `?` calls `From::from(e)` on the error —
> meaning it **automatically converts** the error type if a conversion exists.
> This is why you can use `?` across functions with different (but compatible) error types.

**Example — `?` converting error types automatically:**

```rust
use std::fs::File;
use std::io;
use std::num::ParseIntError;

// Our custom error type that can represent both kinds of errors
enum AppError {
    Io(io::Error),
    Parse(ParseIntError),
}

// Tell Rust: io::Error can be converted into AppError
impl From<io::Error> for AppError {
    fn from(e: io::Error) -> AppError {
        AppError::Io(e)
    }
}

// Tell Rust: ParseIntError can be converted into AppError
impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> AppError {
        AppError::Parse(e)
    }
}

fn run() -> Result<i32, AppError> {
    let _file = File::open("example.txt")?; // io::Error → automatically converted to AppError::Io
    let n: i32 = "42".parse()?;             // ParseIntError → automatically converted to AppError::Parse
    Ok(n)
}
```

Without `From::from`, these two `?` calls would fail to compile because `io::Error`
and `ParseIntError` are different types from `AppError`. The automatic conversion
is what makes `?` so powerful when combining multiple fallible operations.

Reference: [operator-expr.html#the-try-propagation-expression](https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#the-try-propagation-expression)

> Only works inside functions that return `Result`, `Option`, or another `Try`-compatible type.

---

## Python vs Rust Error Handling

### Python — exceptions bubble up automatically

```python
def open_and_read():
    file = open("example.txt")  # raises FileNotFoundError automatically
    return file.read()

def main():
    try:
        content = open_and_read()  # exception bubbles up here
    except FileNotFoundError as e:
        print(f"Error: {e}")       # main handles it
```

Python exceptions travel up **silently and automatically** — middle functions don't need to do anything.

**Two failure modes:**

1. **No try/except** — exception bubbles all the way up and crashes the program at runtime
2. **With try/except** — you might forget to handle a specific case, program still crashes at runtime

Both failures are discovered **at runtime** — possibly in production, by your users.

---

### Rust — errors are explicit values

```rust
fn open_and_read() -> Result<String, io::Error> {
    let file    = File::open("example.txt")?; // must explicitly pass it up with ?
    let content = read_to_string(file)?;
    Ok(content)
}

fn main() {
    match open_and_read() {
        Ok(content) => println!("{content}"),
        Err(e)      => println!("Error: {e}"),
    }
}
```

Errors travel up **only if you explicitly write `?`**. The compiler forces you to deal with every `Result`.

**Rust eliminates both Python failure modes:**

- You **cannot ignore** a `Result` — compiler warns you
- You **cannot forget** a case in `match` — compiler enforces exhaustive matching:

```rust
match result {
    Ok(val) => val,
    // forgot Err arm → compile ERROR, won't build
}
```

```
error[E0004]: non-exhaustive patterns: `Err(_)` not covered
```

---

### Side by side

| | Python | Rust |
|---|---|---|
| How errors travel up | Automatically (invisible) | Explicitly with `?` |
| Can you ignore an error? | Yes, accidentally | No — compiler forces you |
| Forget to handle an error | Crashes at **runtime** | Won't **compile** |
| Miss a case in handler | Crashes at **runtime** | Won't **compile** |
| When you find out | In production | While writing code |

> Rust moves the problem from **"your users discover it"** to **"you discover it before shipping"**.

---

## Quick Comparison

| Method          | Panics on Err? | Returns value? | Good for                        |
|-----------------|---------------|----------------|---------------------------------|
| `match`         | No            | Yes            | Full control, complex logic     |
| `if let`        | No            | No             | One-arm handling                |
| `while let`     | No            | No             | Loops over results              |
| `unwrap()`      | Yes           | Yes            | Tests / quick prototyping       |
| `expect()`      | Yes           | Yes            | Tests with clear panic message  |
| `unwrap_or()`   | No            | Yes            | Simple fallback value           |
| `unwrap_or_else`| No            | Yes            | Computed/logged fallback        |
| `map()`         | No            | Yes (Result)   | Transform success value         |
| `map_err()`     | No            | Yes (Result)   | Transform error value           |
| `and_then()`    | No            | Yes (Result)   | Chain fallible operations       |
| `?`             | No            | Yes            | Propagate errors up the call stack |

---

## دسته‌بندی متدها بر اساس چهار کار اصلی

به‌جای اینکه بپرسیم «`Ok` مهم است یا `Err`؟»، بهتر است بپرسیم **با مقدار داخل `Result` یا `Option` چه کاری می‌خواهیم بکنیم؟** چهار کار اصلی داریم:

1. **استخراج (Extract):** مقدار را بیرون بکش (خروجی دیگر `Result`/`Option` نیست).
2. **تبدیل (Transform):** محتوا را عوض کن ولی بسته‌بندی بماند.
3. **زنجیره (Chain):** عملیات بعدی را که خودش ممکن است شکست بخورد، پشت سر هم وصل کن.
4. **ارسال به بالا (Propagate):** خطا را به تابع صدازننده برگردان.

> تقریباً همه‌ی این متدها هم روی `Result` و هم روی `Option` کار می‌کنند (به‌جز `map_err` که فقط مال `Result` است).

---

### ۱. استخراج (Extract) — بیرون کشیدن مقدار

خروجی این‌ها **مقدار خام** است (مثلاً `i32`)، نه `Result`. سؤال اصلی این است: **اگر شکست خورد، چه کنم؟**

#### الف) اگر شکست خورد → panic

متدهای `unwrap` و `expect` مقدار را برمی‌گردانند و در حالت `Err`/`None` برنامه را متوقف می‌کنند. فرقشان فقط پیام سفارشی در `expect` است.

```rust
let n: i32 = "42".parse().unwrap();                 // 42
let n: i32 = "42".parse().expect("should be a number");  // 42
// "abc".parse::<i32>().unwrap();                   // panic!
```

#### ب) اگر شکست خورد → مقدار پیش‌فرض

در این حالت **خطا برایمان مهم نیست** و فقط یک جایگزین می‌خواهیم:

- متد `unwrap_or(x)`: خودت مقدار پیش‌فرض را می‌دهی (همیشه محاسبه می‌شود، حتی اگر لازم نباشد).
- متد `unwrap_or_default()`: مقدار پیش‌فرض نوع را می‌دهد (مثلاً `0` برای `i32` یا `""` برای `String`).
- متد `unwrap_or_else(closure)`: مقدار پیش‌فرض را **فقط در صورت شکست** با یک closure می‌سازد. در `Result` خطا هم به closure داده می‌شود (`|e|`).

```rust
let a = "abc".parse::<i32>().unwrap_or(0);          // 0
let b = "abc".parse::<i32>().unwrap_or_default();   // 0
let c = "abc".parse::<i32>().unwrap_or_else(|e| {
    println!("error: {e}");                         // error: invalid digit found in string
    -1
});                                                 // -1
let name = None::<&str>.unwrap_or("unknown");       // "unknown"
```

#### ج) کنترل کامل یا فقط یک حالت

ساختار `match` همه‌ی حالت‌ها را پوشش می‌دهد. ساختارهای `if let`، `while let` و `let else` فقط حالتی را که برایت مهم است می‌گیرند. در `let else` بلوک else **باید** از جریان خارج شود (`return`، `break` یا `panic!`).

```rust
fn parse_age(s: &str) -> i32 {
    let Ok(age) = s.parse::<i32>() else {
        return 0;          // must exit here
    };
    age                    // from here on, age is a plain i32
}
// parse_age("x") → 0
```

---

### ۲. تبدیل (Transform) — تغییر محتوا، بسته‌بندی می‌ماند

خروجی این‌ها **هنوز `Result` یا `Option`** است. فقط محتوای داخل یا نوع بسته‌بندی عوض می‌شود.

#### الف) تغییر مقدار موفق → `map`

متد `map` فقط روی `Ok`/`Some` اجرا می‌شود. `Err`/`None` دست‌نخورده عبور می‌کند.

```rust
let r: Result<i32, String> = Ok(2);
r.map(|x| x * 10);                  // Ok(20)

let e: Result<i32, String> = Err("bad".to_string());
e.map(|x| x * 10);                  // Err("bad") — untouched
```

#### ب) تغییر خطا → `map_err`

برعکس `map` است: فقط روی `Err` اجرا می‌شود و `Ok` دست‌نخورده عبور می‌کند. معمولاً برای تبدیل خطا به نوع خطای تابع خودمان به کار می‌رود.

```rust
"abc".parse::<i32>().map_err(|e| format!("invalid input: {e}"));
// Err("invalid input: invalid digit found in string")
```

#### ج) تبدیل بین `Option` و `Result`

- متد `ok_or(err)` یا `ok_or_else(|| err)`: تبدیل `Option` به `Result` (به `None` یک خطا می‌دهد).
- متد `.ok()`: تبدیل `Result` به `Option` (خطا دور ریخته می‌شود).

```rust
let v = vec![1, 2, 3];
v.get(10).ok_or("index not found");  // Err("index not found")

"abc".parse::<i32>().ok();             // None
```

#### 💡 نکته‌هایی برای به یاد ماندن

**۱. قانون اطلاعات:** هر تبدیلی که اطلاعات را دور می‌ریزد، آرگومان نمی‌خواهد. هر تبدیلی که اطلاعات اضافه می‌کند، آرگومان می‌خواهد.

- از `Result` به `Option`: خطا دور ریخته می‌شود و چیزی کم نمی‌آید، پس `.ok()` آرگومان ندارد.
- از `Option` به `Result`: در حالت `None` خطایی وجود ندارد، پس خودت باید یکی بدهی: `ok_or(e)`.

**۲. اسم‌ها را مثل جمله بخوان:**

- متد `.ok()` یعنی «فقط قسمت ok را بده». روی `Result` صدا زده می‌شود. (همزادش `.err()` است که فقط قسمت خطا را نگه می‌دارد.)
- متد `ok_or(e)` یعنی «Ok باشد، **وگرنه** e». روی `Option` صدا زده می‌شود: `Some(v)` می‌شود `Ok(v)` و `None` می‌شود `Err(e)`.

**۳. کلمه‌ی `or` در اسم متدها** همیشه یعنی «اگر نبود، این را بگذار»، مثل `unwrap_or`، `ok_or` و `or`.

**۴. پسوند `_else`** همه‌جا یعنی به‌جای خود مقدار یک **closure** بده که فقط در صورت نیاز اجرا شود:

```rust
opt.ok_or(make_error());          // make_error ALWAYS runs
opt.ok_or_else(|| make_error());  // runs ONLY if None
```

| اسم | روی چه نوعی؟ | تبدیل | آرگومان |
|---|---|---|---|
| متد `.ok()` | روی `Result` | به `Option` | ندارد (خطا دور ریخته می‌شود) |
| متد `ok_or(e)` | روی `Option` | به `Result` | خطای آماده |
| متد `ok_or_else(f)` | روی `Option` | به `Result` | یک closure که خطا را می‌سازد |

> خلاصه در یک جمله: **«دور ریختن مجانی است، ساختن خطا آرگومان می‌خواهد.»**

---

### ۳. زنجیره (Chain) — وصل کردن عملیات‌هایی که ممکن است شکست بخورند

#### الف) اگر موفق بود → عملیات بعدی: `and_then`

وقتی عملیات بعدی **خودش ممکن است شکست بخورد** (یعنی `Result` برمی‌گرداند)، از `and_then` استفاده کن.

| closure چه برمی‌گرداند؟ | با `map` | با `and_then` |
|---|---|---|
| مقدار ساده: `20` | `Ok(20)` ✅ | خطای کامپایل |
| مقدار `Result`: `Ok(5)` | `Ok(Ok(5))` ❌ تودرتو | `Ok(5)` ✅ |

اگر ورودی `Err` باشد، هیچ‌کدام closure را اجرا نمی‌کنند و `Err` همان‌طور عبور می‌کند. پس در زنجیره‌ی `and_then`، **اولین شکست کل زنجیره را متوقف می‌کند**.

```rust
fn parse(s: &str) -> Result<i32, String> {
    s.parse::<i32>().map_err(|e| e.to_string())
}

fn check_positive(n: i32) -> Result<i32, String> {
    if n > 0 { Ok(n) } else { Err("must be positive".to_string()) }
}

parse("5").and_then(check_positive);   // Ok(5)
parse("-5").and_then(check_positive);  // Err("must be positive")
parse("x").and_then(check_positive);   // Err("invalid digit found in string") — check_positive never ran
```

#### 🔬 مقایسه‌ی دقیق `map` و `and_then`

فرق اصلی این است که **چه کسی مقدار را داخل `Ok` می‌گذارد**:

- متد `map` هر چیزی را که closure برگرداند، **خودش** داخل یک `Ok` می‌گذارد.
- متد `and_then` خروجی closure را **همان‌طور که هست** برمی‌گرداند و چیزی اضافه نمی‌کند.

پیاده‌سازی ساده‌شده‌ی هر دو:

```rust
fn map(self, f) -> Result<U, E> {
    match self {
        Ok(v) => Ok(f(v)),   // wraps f's output in Ok
        Err(e) => Err(e),
    }
}

fn and_then(self, f) -> Result<U, E> {
    match self {
        Ok(v) => f(v),       // returns f's output as-is
        Err(e) => Err(e),
    }
}
```

تنها فرق، همان `Ok(f(v))` در برابر `f(v)` است.

> 💡 **راه به خاطر سپردن:**
>
> - **در `map`:** خروجی `Ok(f(v))` است. خودِ `map` دور خروجی `Ok` می‌گذارد، پس `f(v)` می‌تواند **هر چیزی** باشد و نتیجه همیشه `Result` می‌شود.
> - **در `and_then`:** خروجی خودِ `f(v)` است و کسی `Ok` اضافه نمی‌کند، پس `f(v)` **باید ذاتاً `Result` باشد**.

**چرا نوع خطا هم باید یکی باشد؟** در `match` همه‌ی شاخه‌ها باید هم‌نوع باشند:

```rust
match self {
    Ok(v) => f(v),       // this arm's type...
    Err(e) => Err(e),    // ...must match this arm's type: Result<_, E>
}
```

شاخه‌ی `Err(e)` یک `Result` با نوع خطای `E` است. پس `f(v)` هم باید `Result` باشد، **و با همان نوع خطای `E`**. اگر نوع خطاها فرق داشته باشد (مثلاً یکی `String` و دیگری `ParseIntError`)، اول باید با `map_err` آن‌ها را یکی کنی. متد `map` این شرط را ندارد، چون closure آن اصلاً خطایی برنمی‌گرداند.

**متد `map` همیشه تودرتو نمی‌سازد.** اگر closure یک مقدار ساده برگرداند، `map` کاملاً درست کار می‌کند:

```rust
Ok(2).map(|x| x * 10)   // Ok(20)
```

تودرتو شدن فقط **وقتی** پیش می‌آید که closure خودش `Result` برگرداند، مثل `check_positive`.

##### باز کردن هر دو با `match`

این همان دو مثال قبلی (با تابع‌های `parse` و `check_positive` که بالاتر تعریف شدند) است که با `match` باز شده تا ببینی پشت صحنه چه اتفاقی می‌افتد.

این کد:

```rust
let r = parse("5").and_then(check_positive);
```

با `match` می‌شود:

```rust
let r: Result<i32, String> = match parse("5") {
    Ok(v) => check_positive(v),   // return its Result as-is
    Err(e) => Err(e),             // parse failed → pass the error through
};
```

و این کد:

```rust
let r = parse("5").map(check_positive);
```

با `match` می‌شود:

```rust
let r: Result<Result<i32, String>, String> = match parse("5") {
    Ok(v) => Ok(check_positive(v)),   // wraps the Result in another Ok
    Err(e) => Err(e),
};
```

فرق فقط در شاخه‌ی `Ok` است. همان یک `Ok` اضافه، نوع را تودرتو می‌کند.

##### نتیجه برای سه ورودی

| ورودی | قدم اول: `parse` | با `and_then` | با `map` |
|---|---|---|---|
| مقدار `"5"` | `Ok(5)` | `Ok(5)` | `Ok(Ok(5))` |
| مقدار `"-5"` | `Ok(-5)` | `Err("must be positive")` | `Ok(Err("must be positive"))` |
| مقدار `"x"` | `Err("invalid digit...")` | `Err("invalid digit...")` | `Err("invalid digit...")` |

- **ردیف `"x"`:** هر دو یکسان‌اند، چون `parse` شکست خورد و رفت در شاخه‌ی `Err`. تابع `check_positive` اصلاً صدا زده نشد.
- **ردیف `"-5"` با `map`:** نتیجه `Ok(Err(...))` است. یعنی بیرونش می‌گوید «موفق» ولی داخلش خطاست. این گیج‌کننده است، و به همین دلیل اینجا باید از `and_then` استفاده کرد.
- **متد `map` نمی‌تواند `Ok` را به `Err` تبدیل کند،** چون وقتی ورودی `Ok` است، خروجی closure حتماً داخل یک `Ok` گذاشته می‌شود. ولی `and_then` می‌تواند: در ردیف `"-5"`، مقدار `Ok(-5)` به `Err` تبدیل شد.

اگر بخواهی با `map` به همان نتیجه برسی، باید یک قدم اضافه بزنی تا لایه‌ی بیرونی را باز کنی:

```rust
let r = parse("5").map(check_positive).and_then(|inner| inner);  // Ok(5)
```

متد `and_then` همین دو کار را یک‌جا انجام می‌دهد.

##### قانون انتخاب

| خروجی closure | متد مناسب |
|---|---|
| مقدار ساده، مثل `i32` یا `String` | متد `map` |
| خودش `Result` یا `Option` است (یعنی ممکن است شکست بخورد) | متد `and_then` |

> همین قانون برای `Option` هم برقرار است: `Some(...)` جای `Ok(...)` را می‌گیرد.

#### ب) اگر شکست خورد → امتحان یک جایگزین: `or` و `or_else`

برعکس `and_then` است: فقط در حالت `Err`/`None` اجرا می‌شود و یک `Result`/`Option` دیگر را امتحان می‌کند. فرقش با `unwrap_or` این است که خروجی هنوز بسته‌بندی شده است.

هر دو متد **هم روی `Option` و هم روی `Result`** کار می‌کنند. تنها فرق در closure متد `or_else` است:

- روی `Option`: closure ورودی ندارد (`|| ...`)، چون `None` چیزی برای دادن ندارد.
- روی `Result`: closure **خطا را می‌گیرد** (`|e| ...`)، پس می‌توانی بر اساس نوع خطا تصمیم بگیری که جایگزین بدهی یا خطا را نگه داری.

```rust
// on Option
None.or(Some(1));                   // Some(1)
None::<i32>.or_else(|| Some(8080)); // Some(8080)

// on Result
let a: Result<i32, &str> = Err("first one failed");
let r: Result<i32, &str> = a.or(Ok(0));  // Ok(0)

let b: Result<i32, &str> = Err("timeout");
b.or_else(|e| if e == "timeout" { Ok(-1) } else { Err(e) });  // Ok(-1)

let c: Result<i32, &str> = Ok(5);
let r: Result<i32, &str> = c.or(Ok(0));  // Ok(5) — it succeeded, fallback not used
```

---

### ۴. ارسال به بالا (Propagate) — عملگر `?`

اگر `Ok`/`Some` باشد، مقدار داخل را می‌دهد. اگر `Err`/`None` باشد، **همان لحظه از تابع return می‌کند** و خطا را به صدازننده می‌دهد.

```rust
use std::num::ParseIntError;

fn sum(a: &str, b: &str) -> Result<i32, ParseIntError> {
    let x = a.parse::<i32>()?;   // on error, returns right here
    let y = b.parse::<i32>()?;
    Ok(x + y)
}
// sum("2", "3") → Ok(5)
// sum("2", "x") → Err(ParseIntError { kind: InvalidDigit })

fn first_upper(s: &str) -> Option<char> {
    let c = s.chars().next()?;   // empty string → returns None
    Some(c.to_ascii_uppercase())
}
// first_upper("rust") → Some('R')
// first_upper("")     → None
```

نکته‌ها:

- فقط داخل تابعی کار می‌کند که خودش `Result` یا `Option` برمی‌گرداند.
- در `Result` خطا را عیناً برنمی‌گرداند، بلکه اول با `From` آن را به نوع خطای تابع تبدیل می‌کند.

#### 🧠 «بالا» یعنی کجا؟

وقتی تابع A تابع B را صدا می‌زند، A «بالای» B است. «ارسال به بالا» یعنی B به خطا خورده ولی **خودش تصمیم نمی‌گیرد** با آن چه کند. خطا را به A تحویل می‌دهد تا A تصمیم بگیرد.

یک تشبیه: مدیر از کارمند گزارش می‌خواهد و کارمند از کارآموز داده می‌خواهد. کارآموز می‌بیند فایل داده‌ها وجود ندارد. سه راه دارد:

| کار کارآموز | معادل در Rust |
|---|---|
| همه‌چیز را رها کند و برود | متدهای `unwrap` و `expect` (panic) |
| خودش یک عدد الکی بگذارد | متدهای `unwrap_or` و `unwrap_or_else` |
| به کارمند بگوید «فایل نبود» | عملگر `?` (ارسال به بالا) |

کارمند هم می‌تواند خودش مشکل را حل کند یا دوباره به مدیر بگوید. در نهایت کسی تصمیم می‌گیرد که **دید کلی دارد**.

#### 🔍 عملگر `?` پشت صحنه چه می‌نویسد؟

عملگر `?` یک **خلاصه‌نویسی** است (به انگلیسی: syntactic sugar). یعنی تو کوتاه می‌نویسی و کامپایلر آن را به کد کامل باز می‌کند:

```rust
// what you write:
let x = a.parse::<i32>()?;

// what the compiler sees:
let x = match a.parse::<i32>() {
    Ok(v) => v,                          // success → give the value, keep going
    Err(e) => return Err(From::from(e)), // failure → exit the function NOW
};
```

پس داخل `?` یک **`return` پنهان** وجود دارد. عملگر `?` خطا را حل نمی‌کند، فقط تابع را زودتر تمام می‌کند و خطا را تحویل صدازننده می‌دهد.

#### ⚠️ چرا `return` لازم است؟ فرق `Err(e) => Err(e)` با `Err(e) => return Err(e)`

کلمه‌ی `return` یعنی **از کل تابع خارج شو**. بدون آن، `Err(e)` فقط **نتیجه‌ی همان `match`** می‌شود و تابع به کارش ادامه می‌دهد.

```rust
// ❌ without return
let x = match a.parse::<i32>() {
    Ok(v) => v,          // this arm gives i32
    Err(e) => Err(e),    // this arm gives Result
};
// does not compile: all match arms must have the same type
// (match arms have incompatible types)
```

```rust
// ✅ with return
let x = match a.parse::<i32>() {
    Ok(v) => v,
    Err(e) => return Err(e),   // exits the function, never reaches x
};
// x is always a clean i32
```

شاخه‌ی دارای `return` هیچ‌وقت مقداری به `x` نمی‌دهد، چون قبلش از تابع خارج شده‌ایم. برای همین کامپایلر با آن مشکلی ندارد.

| کد | نتیجه |
|---|---|
| حالت `Err(e) => Err(e)` | مقدار `match` می‌شود `Err` و تابع ادامه می‌دهد |
| حالت `Err(e) => return Err(e)` | تابع همین‌جا تمام می‌شود و `Err` به صدازننده می‌رسد |

#### 🪜 یک مثال کامل: خطا پله‌پله بالا می‌رود

```rust
use std::num::ParseIntError;

// bottom step: only converts
fn parse_number(s: &str) -> Result<i32, ParseIntError> {
    s.parse::<i32>()
}

// middle step: adds, but makes no decision about errors
fn sum(a: &str, b: &str) -> Result<i32, ParseIntError> {
    let x = parse_number(a)?;
    let y = parse_number(b)?;
    Ok(x + y)               // reached only if both succeeded
}

// top step: the final decision is made here
fn main() {
    match sum("2", "x") {
        Ok(total) => println!("sum: {total}"),
        Err(e) => println!("bad input: {e}"),
    }
}
```

مسیر خطا:

```
parse_number("x")  ← error is created here
      ↑ Err
sum                ← sends it up with ? (Ok(x + y) never runs)
      ↑ Err
main               ← decides with match: print a message
```

**چرا این کار خوب است؟** تابع `sum` نمی‌داند در چه برنامه‌ای استفاده می‌شود. شاید یک برنامه بخواهد پیام چاپ کند، یکی مقدار پیش‌فرض بگذارد و دیگری دوباره از کاربر ورودی بگیرد. پس بهتر است `sum` تصمیم نگیرد.

#### 🚫 اگر صدازننده هم خطا را مدیریت نکند چه؟

**در Rust خطا نمی‌تواند بی‌صدا گم شود.** صدازننده یک `Result` دریافت می‌کند، نه یک عدد. پس نمی‌تواند مستقیم از عدد استفاده کند و باید یکی از این کارها را بکند:

**۱. مدیریتش کند،** با `match` یا `unwrap_or` یا روش‌های دیگر.

**۲. خودش هم با `?` بالاتر بفرستد.** خطا یک پله‌ی دیگر بالا می‌رود.

**۳. نادیده‌اش بگیرد.** اگر کلاً به نتیجه کاری نداشته باشد، کامپایلر هشدار می‌دهد:

```rust
sum("2", "x");          // ⚠️ warning: unused `Result` that must be used

let _ = sum("2", "x");  // ✅ explicit: I know, I discard it on purpose
```

**۴. اگر خطا تا `main` بالا برود چه؟** تابع `main` هم می‌تواند `Result` برگرداند و در این صورت داخلش `?` مجاز است:

```rust
fn main() -> Result<(), ParseIntError> {
    let total = sum("2", "x")?;
    println!("{total}");
    Ok(())
}
```

اگر خطا به اینجا برسد، Rust خودش آن را چاپ می‌کند و برنامه با **کد خروج ۱** (یعنی ناموفق) تمام می‌شود:

```
Error: ParseIntError { kind: InvalidDigit }
```

> خلاصه: خطا یا جایی مدیریت می‌شود، یا تا `main` بالا می‌رود و برنامه با پیام خطا تمام می‌شود. **گم شدن بی‌صدا ممکن نیست**، در حالی که در Python یک exception ممکن است بی‌خبر برنامه را در production از کار بیندازد.

#### 📝 خلاصه در یک جمله

**«اگر موفق بود مقدار را بده و ادامه بده؛ اگر شکست خورد از تابع خارج شو و خطا را به صدازننده تحویل بده.»**

---

### جمع‌بندی

| کار اصلی | سؤال | متدها | خروجی |
|---|---|---|---|
| استخراج | اگر شکست خورد panic کنم؟ | متدهای `unwrap` و `expect` | مقدار خام |
| استخراج | اگر شکست خورد مقدار پیش‌فرض بدهم؟ | متدهای `unwrap_or`، `unwrap_or_default` و `unwrap_or_else` | مقدار خام |
| استخراج | همه‌ی حالت‌ها یا فقط یک حالت؟ | ساختارهای `match`، `if let`، `while let` و `let else` | دست خودت |
| تبدیل | مقدار موفق را عوض کنم؟ | متد `map` | بسته‌بندی‌شده |
| تبدیل | خطا را عوض کنم؟ | متد `map_err` | بسته‌بندی‌شده |
| تبدیل | بین `Option` و `Result` جابه‌جا کنم؟ | متدهای `ok_or`، `ok_or_else` و `.ok()` | بسته‌بندی‌شده |
| زنجیره | اگر موفق بود عملیات بعدی؟ | متد `and_then` | بسته‌بندی‌شده |
| زنجیره | اگر شکست خورد جایگزین امتحان کنم؟ | متدهای `or` و `or_else` | بسته‌بندی‌شده |
| ارسال به بالا | خطا را به صدازننده بدهم؟ | عملگر `?` | مقدار خام (یا return زودهنگام) |
