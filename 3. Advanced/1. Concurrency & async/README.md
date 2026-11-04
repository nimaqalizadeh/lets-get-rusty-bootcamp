# Concurrency & async
Yes, `i32` implements `Send`, and it also implements `Sync`. You never see an `impl Send for i32` line because of how these traits work.

## They are auto traits

`Send` and `Sync` are **auto traits**: the compiler implements them for you. The rule is structural:

- A type is `Send` if all of its fields are `Send`.
- A type is `Sync` if all of its fields are `Sync`.

Primitives like `i32`, `bool` and `f64` are `Send + Sync`, so anything built from them is too:

```rust
struct Point { x: i32, y: i32 }   // automatically Send + Sync
```

One type that isn't `Send` "infects" everything that contains it:

```rust
use std::rc::Rc;
struct Holder { data: Rc<i32> }   // NOT Send, because Rc isn't
```

## What "marker trait" means

A marker trait has **no methods**. Its definition is basically empty:

```rust
pub unsafe auto trait Send {}
```

It doesn't give the type any behavior. It only labels the type with a property, here "safe to move to another thread." The compiler then checks that label wherever it matters. For example, `std::thread::spawn` requires the closure to be `Send`:

```rust
let rc = Rc::new(5);
std::thread::spawn(move || println!("{rc}")); // compile error: Rc<i32> cannot be sent between threads
```

## A correction to your phrasing

It's not quite "properties that aren't forced by the developer." It's closer to this:

> Marker traits describe properties the compiler **can't see by looking at the type's methods**, so it relies on the label to enforce thread safety at compile time.

Most of the time the compiler infers the label itself. The developer only steps in when the compiler can't work it out. This usually happens with raw pointers (`*const T`, `*mut T`), which are deliberately neither `Send` nor `Sync`. If you build something like your own `Mutex` on top of raw pointers, you have to promise it's safe:

```rust
unsafe impl Send for MyMutex {}
unsafe impl Sync for MyMutex {}
```

The `unsafe` there means "compiler, you can't verify this, so I take responsibility." If the promise is wrong, you get data races.

## Send vs Sync

- **`Send`**: ownership of the value can move to another thread.
- **`Sync`**: the value can be *shared* between threads by reference. Formally, `T` is `Sync` exactly when `&T` is `Send`.

A few classic examples:

| Type | Send | Sync | Why |
|---|---|---|---|
| `i32`, `String`, `Vec<T>` | ✅ | ✅ | plain data |
| `Rc<T>` | ❌ | ❌ | non-atomic reference count, so two threads could corrupt it |
| `Arc<T>` (T: Send+Sync) | ✅ | ✅ | atomic reference count |
| `RefCell<T>` | ✅ | ❌ | can move to one thread, but its borrow tracking isn't thread-safe to share |
| `Mutex<T>` (T: Send) | ✅ | ✅ | locking makes shared access safe |
| `*mut T` | ❌ | ❌ | the compiler can't reason about it |

Thanks to these two labels, Rust can catch data races at compile time instead of at runtime.

## توضیح ساده: Send و Sync یعنی چه؟

فرض کنید هر thread یک آدم است و یک مقدار، مثلاً یک دفترچه، بین آن‌ها جابه‌جا می‌شود. هر کدام از این دو trait جواب یک سؤال ساده است.

### سؤال اول، یعنی `Send`

آیا می‌شود این دفترچه را **کامل به یک نفر دیگر داد** و دیگر خودمان نگهش نداریم؟

در این حالت مالکیت (ownership) به thread دیگر منتقل می‌شود و در هر لحظه فقط **یک نفر** دفترچه را دارد.

```rust
let s = String::from("hi");
std::thread::spawn(move || println!("{s}")); // OK: String is Send
```

اینجا `s` کامل به thread جدید منتقل شده و thread اصلی دیگر به آن دسترسی ندارد.

### سؤال دوم، یعنی `Sync`

آیا **چند نفر می‌توانند هم‌زمان** همین دفترچه را نگاه کنند؟

یعنی چند thread هم‌زمان یک reference به آن داشته باشند (`&T`). تعریف رسمی‌اش همین است: اگر `&T` قابل Send باشد، خود `T` یعنی Sync است.

### چرا بعضی نوع‌ها این دو را ندارند؟

بهترین مثال `Rc` است. این نوع یک شمارنده دارد که تعداد صاحب‌ها را نگه می‌دارد، و این شمارنده **اتمیک نیست**:

```rust
let a = Rc::new(5);
let b = Rc::clone(&a); // count: 1 -> 2
```

اگر دو thread هم‌زمان clone کنند، هر دو عدد ۱ را می‌خوانند و هر دو ۲ می‌نویسند. شمارنده خراب می‌شود و حافظه ممکن است زودتر از موقع آزاد شود. به همین دلیل `Rc` نه Send است نه Sync. نسخهٔ امنش `Arc` است که شمارنده‌اش اتمیک است.

مثال جالب‌تر `RefCell` است که **Send هست ولی Sync نیست**:

- دادنش به یک thread دیگر مشکلی ندارد، چون باز هم فقط یک نفر از آن استفاده می‌کند.
- اما اگر دو thread هم‌زمان به آن reference داشته باشند، ممکن است هر دو هم‌زمان `borrow_mut` بگیرند. سیستم شمارش borrow در `RefCell` برای این وضعیت ساخته نشده.

## استفاده در generic ها: `T: Send + Sync`

کامپایلر این برچسب‌ها را خودش روی نوع‌ها می‌گذارد. جایی که آن‌ها را می‌بینیم **trait bound** های generic است: توابعی که با thread کار می‌کنند شرط می‌گذارند که نوع ورودی حتماً این برچسب‌ها را داشته باشد.

### مثال از کتابخانهٔ استاندارد

امضای `thread::spawn` تقریباً این شکلی است:

```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
```

معنی‌اش این است:

- **شرط روی F:** closure باید Send باشد، چون قرار است به thread جدید منتقل شود.
- **شرط روی T:** خروجی هم باید Send باشد، چون از thread جدید به thread اصلی برمی‌گردد.
- **شرط static:** closure نباید به متغیرهای محلی reference داشته باشد، چون ممکن است thread بیشتر از تابع فعلی زنده بماند.

برای همین وقتی یک `Rc` را داخل closure می‌برید، خطا از همین `F: Send` می‌آید.

### در کد خودمان کی لازم می‌شود؟

وقتی تابع یا struct ما مقدار generic را به thread دیگری بفرستد:

```rust
use std::thread;

fn process_in_background<T: Send + 'static>(value: T) {
    thread::spawn(move || {
        drop(value);
    });
}
```

اگر `T: Send` را ننویسیم، کامپایلر داخل تابع خطا می‌دهد. دلیلش این است که از کجا معلوم `T` قابل فرستادن باشد؟

### کی Send + Sync با هم؟

وقتی مقدار قرار است بین چند thread **به اشتراک گذاشته شود**، مثلاً داخل `Arc`:

```rust
use std::sync::Arc;
use std::thread;

fn share<T: Send + Sync + 'static>(value: T) {
    let shared = Arc::new(value);
    for _ in 0..3 {
        let s = Arc::clone(&shared);
        thread::spawn(move || {
            let _r = &*s; // many threads read the same value
        });
    }
}
```

در این مثال Sync لازم است چون چند thread هم‌زمان به **همان** مقدار دسترسی دارند. Send هم لازم است چون ممکن است آخرین `Arc` در یک thread دیگر drop شود و مقدار همان‌جا آزاد شود.

### یک الگوی خیلی رایج

در کد async و مخصوصاً Axum و Tokio، این را زیاد می‌بینیم:

```rust
Box<dyn std::error::Error + Send + Sync>
```

این یعنی خطایی که می‌شود بین threadها جابه‌جا و share کرد. دلیلش این است که Tokio ممکن است یک task را وسط کار به thread دیگری منتقل کند.

### قاعدهٔ سرانگشتی

- **فقط Send:** وقتی مقدار را به یک thread دیگر **می‌دهیم**.
- **ترکیب Send + Sync:** وقتی چند thread هم‌زمان از **یک** مقدار استفاده می‌کنند.

اگر مطمئن نیستیم، bound را ننویسیم. کامپایلر دقیقاً می‌گوید کدام شرط کم است.

---
# Async syntax sugar
## پشت پردهٔ `async fn`

کلمهٔ `async` در واقع یک خلاصه‌نویسی (syntactic sugar) است. این دو تابع از نظر مفهومی یکی هستند:

```rust
use std::future::Future;

// What we write:
async fn my_function() {
    println!("I'm an async function!");
}

// What the compiler turns it into (roughly):
fn my_function() -> impl Future<Output = ()> {
    async {
        println!("I'm an async function!");
    }
}
```

نکته‌های مهم:

- **خروجی واقعی** یک `async fn` خود مقدار نیست، بلکه یک **Future** است. اینجا `Output = ()` یعنی وقتی Future تمام شود، مقدار `()` می‌دهد. نوع برگشتی که در امضای `async fn` نوشته می‌شود همان `Output` است.
- **بدون بلوک `async`** نسخهٔ دوم کامپایل نمی‌شود. اگر بدنه فقط `println!` باشد، تابع `()` برمی‌گرداند و نه یک Future.
- **صدا زدن تابع** هیچ کدی را اجرا نمی‌کند و فقط Future را می‌سازد. چاپ پیام فقط وقتی انجام می‌شود که Future را `.await` کنیم یا یک runtime مثل Tokio آن را اجرا کند. به همین دلیل می‌گویند Future ها در Rust **تنبل (lazy)** هستند.

```rust
#[tokio::main]
async fn main() {
    let fut = my_function(); // nothing is printed yet
    fut.await;               // now it prints
}
```

## معنی `impl Future<Output = ()>`

### نوشتن `impl Trait` در جای نوع برگشتی

این دقیقاً trait bound نیست، ولی خیلی به آن نزدیک است. معنی‌اش این است: «این تابع **یک نوع مشخص** برمی‌گرداند که Future را پیاده کرده، ولی اسمش گفته نمی‌شود.»

دلیلش این است که نوعی که کامپایلر از بلوک `async` می‌سازد اسم ندارد و نمی‌شود آن را نوشت. برای همین فقط می‌گوییم چه trait ای را پیاده کرده است. اما سینتکس `<Output = ()>` دقیقاً همان چیزی است که در trait bound ها هم استفاده می‌شود.

### نیازهای trait به نام Future

تعریفش در کتابخانهٔ استاندارد این است:

```rust
pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

هر نوعی که بخواهد Future باشد باید دو چیز را مشخص کند:

1. **نوع Output:** وقتی کار تمام شد، چه مقداری تحویل می‌دهد.
2. **متد poll:** runtime (مثلاً Tokio) مدام این متد را صدا می‌زند و می‌پرسد «تمام شدی؟». جواب یکی از این دو است:
   - **حالت Ready:** بله، و مقدار همراهش است (`Poll::Ready(value)`).
   - **حالت Pending:** هنوز نه، بعداً دوباره بپرس (`Poll::Pending`).

معمولاً لازم نیست این را دستی پیاده کنیم. وقتی `async fn` یا بلوک `async` می‌نویسیم، کامپایلر خودش یک نوع می‌سازد و `poll` را برایش پیاده می‌کند.

### نسخهٔ ساده‌شدهٔ Future و Poll

برای فهمیدن ایدهٔ اصلی، می‌شود Future را این‌طور ساده تصور کرد:

```rust
trait Future {
    type Output;
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output>;
}

enum Poll<T> {
    Ready(T),
    Pending,
}
```

#### نقش هر قسمت

- **خط `type Output;`** جای خالی نوع خروجی است. هر Future خودش تعیین می‌کند وقتی تمام شد چه چیزی تحویل می‌دهد.
- **متد poll** یعنی «تمام شدی؟». runtime این متد را صدا می‌زند تا Future را یک قدم جلو ببرد.
- **نوع Poll** یک enum ساده با دو حالت است:
  - **حالت Ready(T):** کار تمام شده و مقدار نهایی داخلش است.
  - **حالت Pending:** هنوز کار تمام نشده.
- **عبارت `Poll<Self::Output>`** یعنی جای `T` در Poll همان Output ای قرار می‌گیرد که این Future انتخاب کرده. مثلاً اگر `type Output = String` باشد، خروجی poll می‌شود `Poll<String>`.

#### نقش پارامتر wake

اگر Future جواب Pending بدهد، runtime باید بداند **کی** دوباره poll کند. دو راه دارد:

1. **پرسیدن مداوم:** مدام در یک حلقه poll کند. این کار CPU را بیهوده مصرف می‌کند.
2. **صبر برای خبر:** Future تابع `wake` را نگه دارد و هر وقت آماده شد (مثلاً جواب شبکه رسید) آن را صدا بزند. runtime فقط بعد از آن دوباره poll می‌کند.

حالت دوم همان چیزی است که Rust انجام می‌دهد. به همین دلیل یک thread می‌تواند هزاران Future را مدیریت کند: فقط سراغ Future هایی می‌رود که خبر داده‌اند آماده‌اند.

```
runtime            future
   | --- poll ---->  |
   | <-- Pending --  |   (future keeps `wake`)
   |                 |
   |   ...waiting... |   (data arrives)
   | <--- wake() --- |
   | --- poll ---->  |
   | <-- Ready(v) -- |
```

#### تفاوت با تعریف واقعی

تعریف واقعی که بالاتر آمد همان ایده را دارد، فقط دقیق‌تر است:

```rust
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
```

- **به جای `wake: fn()`** یک `Context` می‌گیرد که داخلش یک `Waker` است. با `cx.waker().wake_by_ref()` همان کار wake انجام می‌شود.
- **به جای `&mut self`** از `Pin<&mut Self>` استفاده می‌کند. این تضمین می‌کند Future بعد از شروع poll شدن در حافظه جابه‌جا نشود، چون Future هایی که از `async` ساخته می‌شوند ممکن است به داده‌های داخل خودشان reference داشته باشند.

### عبارت `Output = ()` و فرقش با generic

اینجا `Output` یک generic نیست، بلکه یک **associated type** است، یعنی همان خط `type Output;` که داخل trait تعریف شده. سینتکس `Trait<Name = Type>` یعنی «این trait را پیاده کرده، **و** associated type اش برابر این نوع است».

### نمونه‌های دیگر این سینتکس

رایج‌ترین نمونه Iterator است:

```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

fn evens() -> impl Iterator<Item = u32> {
    (0..10).filter(|n| n % 2 == 0)
}

fn sum_all<I: Iterator<Item = i32>>(iter: I) -> i32 {
    iter.sum()
}

let v: Box<dyn Iterator<Item = String>> = Box::new(vec![].into_iter());
```

چند مورد دیگر:

- **عملگرها:** در `Add<Output = i32>`، نوع `Output` نتیجهٔ `+` است.
- **اشاره‌گرها:** در `Deref<Target = str>`، نوع `Target` چیزی است که `*` به آن می‌رسد.
- **کلوژرها:** نوشتن `Fn() -> T` در واقع خلاصهٔ یک associated type به اسم `Output` است.

### دلیل استفاده از associated type به جای generic

فرض کنید Future به شکل generic تعریف شده بود:

```rust
trait Future<T> { /* ... */ }
```

آن‌وقت یک نوع می‌توانست هم‌زمان `Future<i32>` و `Future<String>` را پیاده کند. هر بار که آن را `.await` می‌کردیم، کامپایلر نمی‌دانست کدام را می‌خواهیم و باید خودمان مشخص می‌کردیم.

اما یک Future در واقعیت **فقط یک نوع خروجی** دارد. با associated type:

- **انتخاب نوع:** خود نوع پیاده‌کننده مشخص می‌کند خروجی چیست، و فقط یک بار.
- **استفاده کردن:** استفاده‌کننده لازم نیست چیزی بنویسد، چون کامپایلر خودش می‌داند.
- **محدود کردن:** فقط وقتی بخواهیم خروجی را محدود کنیم، `<Output = ...>` را می‌نویسیم.

### قاعدهٔ سرانگشتی

- **از associated type استفاده کنید** اگر هر نوع فقط **یک** جواب دارد، مثل `Item` در Iterator و `Output` در Future.
- **از generic استفاده کنید** اگر یک نوع می‌تواند trait را **چند بار با نوع‌های مختلف** پیاده کند. مثلاً یک نوع می‌تواند هم `From<i32>` و هم `From<String>` را پیاده کند.

نمونهٔ جالب `Add<Rhs>` است که هر دو را دارد. نوع سمت راست generic است (`Rhs`)، چون می‌شود با نوع‌های مختلف جمع کرد. اما نتیجه associated type است (`Output`)، چون برای هر ترکیب فقط یک نتیجه وجود دارد.

## بررسی دقیق‌تر `type Item` در Iterator

### خط `type Item;` نوع جدیدی نمی‌سازد

این خط فقط یک **جای خالی** تعریف می‌کند. یعنی trait می‌گوید: «هر کس من را پیاده کند، باید بگوید Item برایش چه نوعی است.»

دقیقاً مثل متد `next` است. در trait فقط امضای متد آمده و بدنه ندارد، چون بدنه را پیاده‌کننده می‌نویسد. خط `type Item;` هم `= ...` ندارد، چون مقدارش را پیاده‌کننده تعیین می‌کند.

در واقع trait مثل یک فرم با دو جای خالی است:

- **جای خالی اول:** نوع Item چیست؟
- **جای خالی دوم:** متد next چه کار می‌کند؟

### نوع Item فیلد struct نیست

وقتی می‌گوییم Self «Item دارد»، منظور این نیست که struct فیلدی به اسم Item دارد. Item یک **نوع** است، نه یک داده. این نوع داخل بلوک `impl` تعیین می‌شود، نه داخل تعریف struct.

### یک مثال کامل

```rust
struct Counter {
    count: u32,
}

impl Iterator for Counter {
    type Item = u32; // fill the blank: Item is u32

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < 5 {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}
```

اینجا قدم به قدم این اتفاق می‌افتد:

1. **معنی Self:** داخل این impl، یعنی `Counter`.
2. **معنی Self::Item:** یعنی «Item ای که Counter انتخاب کرده»، که همان `u32` است.
3. **نتیجه:** امضای `next` برای Counter در عمل این می‌شود: `fn next(&mut self) -> Option<u32>`.

علامت `::` در `Self::Item` یعنی «نوعی به اسم Item که متعلق به Self است». این دسترسی به فیلد نیست. مثل `String::new` است که چیزی را از داخل String برمی‌دارد.

### پیاده‌سازی‌های مختلف، Item های مختلف

```rust
struct Words {
    list: Vec<String>,
}

impl Iterator for Words {
    type Item = String; // here Item is String

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop()
    }
}
```

هر دو یک trait را پیاده کرده‌اند، ولی نتیجه فرق دارد:

- **برای Counter:** نوع Item یعنی `u32` و next خروجی `Option<u32>` دارد.
- **برای Words:** نوع Item یعنی `String` و next خروجی `Option<String>` دارد.

### ارتباطش با `Iterator<Item = u32>`

حالا آن سینتکس هم معنی پیدا می‌کند. وقتی می‌نویسیم:

```rust
fn sum_all(iter: impl Iterator<Item = u32>) -> u32 {
    iter.sum()
}
```

یعنی «هر iterator ای قبول است، به شرطی که جای خالی Item را با `u32` پر کرده باشد». پس `Counter` قبول می‌شود ولی `Words` رد می‌شود.
