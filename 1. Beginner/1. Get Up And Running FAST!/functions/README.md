# Functions are minimal

Rust functions are intentionally simple compared to languages like Python. Here's what that means:

## 1. Input -> Output

Functions take typed inputs and return typed outputs. No magic.

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

## 2. No default values

**Python** lets you set defaults:
```python
def greet(name="World"):
    print(f"Hello, {name}!")

greet()        # Hello, World!
greet("Rust")  # Hello, Rust!
```

**Rust** doesn't have default parameters. You can use `Option` or the builder pattern instead:
```rust
fn greet(name: Option<&str>) {
    let name = name.unwrap_or("World");
    println!("Hello, {}!", name);
}

greet(None);          // Hello, World!
greet(Some("Rust"));  // Hello, Rust!
```

## 3. No keyword arguments

### Keyword / named arguments

**Python** lets you pass arguments by name in any order:
```python
def create_user(name, age, active=True):
    ...

create_user("Alice", 30)                      # positional
create_user(age=30, name="Alice", active=False) # by name, any order
```

**Rust** only has positional arguments — order matters and you can't skip or name them:
```rust
fn create_user(name: &str, age: u32, active: bool) {
    println!("{}, {}, {}", name, age, active);
}

create_user("Alice", 30, false); // must match the exact order
// create_user(age: 30, name: "Alice"); ❌ not valid Rust
```

For many parameters, use a struct to get named-field clarity:
```rust
struct UserConfig {
    name: String,
    age: u32,
    active: bool,
}

fn create_user(config: UserConfig) { ... }

create_user(UserConfig {
    name: "Alice".to_string(),
    age: 30,
    active: false,
});
```

### Variadic arguments (`*args` and `**kwargs`)

**Python** accepts a variable number of arguments:
```python
# *args — variable positional arguments
def sum_all(*args):
    return sum(args)

sum_all(1, 2, 3)       # 6
sum_all(1, 2, 3, 4, 5) # 15

# **kwargs — variable keyword arguments
def print_info(**kwargs):
    for key, value in kwargs.items():
        print(f"{key}: {value}")

print_info(name="Alice", age=30, city="Tokyo")
```

**Rust** has no variadic functions. Here are the alternatives:

**Slices** — for a variable number of same-type values:
```rust
fn sum_all(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}

sum_all(&[1, 2, 3]);       // 6
sum_all(&[1, 2, 3, 4, 5]); // 15
```

**Vecs** — when you build the list dynamically:
```rust
fn sum_all(numbers: Vec<i32>) -> i32 {
    numbers.iter().sum()
}

let nums = vec![1, 2, 3, 4, 5];
sum_all(nums); // 15
```

**Macros** — for true variadic syntax (this is how `println!`, `vec!` work):
```rust
macro_rules! sum_all {
    ($($x:expr),*) => {{
        let mut total = 0;
        $(total += $x;)*
        total
    }};
}

sum_all!(1, 2, 3);       // 6
sum_all!(1, 2, 3, 4, 5); // 15
```

**HashMap** — the closest equivalent to `**kwargs`:
```rust
use std::collections::HashMap;

fn print_info(kwargs: HashMap<&str, &str>) {
    for (key, value) in &kwargs {
        println!("{}: {}", key, value);
    }
}

let mut info = HashMap::new();
info.insert("name", "Alice");
info.insert("age", "30");
info.insert("city", "Tokyo");
print_info(info);
```

## 4. No exceptions

**Python** uses try/except:
```python
def divide(a, b):
    if b == 0:
        raise ValueError("division by zero")
    return a / b

try:
    result = divide(10, 0)
except ValueError as e:
    print(e)
```

**Rust** uses `Result` — errors are values, not control flow:
```rust
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        return Err("division by zero".to_string());
    }
    Ok(a / b)
}

match divide(10.0, 0.0) {
    Ok(val) => println!("{}", val),
    Err(e) => println!("{}", e),
}
```

## 5. No null/none

**Python** uses `None` freely:
```python
def find_user(id):
    if id == 1:
        return "Alice"
    return None  # could cause AttributeError later

user = find_user(2)
print(user.upper())  # 💥 AttributeError at runtime
```

**Rust** uses `Option` — you must handle the absence explicitly:
```rust
fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some("Alice".to_string())
    } else {
        None
    }
}

match find_user(2) {
    Some(user) => println!("{}", user.to_uppercase()),
    None => println!("User not found"),  // compiler forces you to handle this
}
```

## 6. No overloading

**Python** can fake overloading with default args or `*args`:
```python
def area(length, width=None):
    if width is None:
        return length * length  # square
    return length * width       # rectangle
```

**Rust** has no function overloading. Use traits or different function names:
```rust
fn square_area(side: f64) -> f64 {
    side * side
}

fn rect_area(length: f64, width: f64) -> f64 {
    length * width
}
```

Or use traits for polymorphism:
```rust
trait Area {
    fn area(&self) -> f64;
}

struct Square { side: f64 }
struct Rectangle { length: f64, width: f64 }

impl Area for Square {
    fn area(&self) -> f64 { self.side * self.side }
}

impl Area for Rectangle {
    fn area(&self) -> f64 { self.length * self.width }
}
```
