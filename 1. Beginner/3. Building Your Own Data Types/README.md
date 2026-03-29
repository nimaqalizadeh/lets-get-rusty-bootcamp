# `match` expression, `if let`, `let else` and `while let`

## How `match` Works Under the Hood

`match` takes a value (the **scrutinee**) and tries each arm top-to-bottom. Each arm has two steps:

1. **Pattern check** — does the value fit the structural shape?
2. **Guard check** (optional) — does the `if` condition pass?

First arm where both pass wins. The body executes and match is done.

```rust
match value {
    pattern if guard => body,
//  ^^^^^^^ step 1: does the shape fit?
//          ^^^^^^^^ step 2: is condition true?
//                    ^^^^^ step 3: execute
}
```

---

## Exhaustiveness

The compiler requires that **every possible value** is covered. If you miss a case, it won't compile.

```rust
enum Color { Red, Green, Blue }

match color {
    Color::Red => "red",
    Color::Green => "green",
    // ERROR: Color::Blue not covered
}
```

The wildcard `_` catches everything remaining:

```rust
match color {
    Color::Red => "red",
    _ => "other",
}
```

**Important:** Guards are invisible to the exhaustiveness checker. Even if your guards logically cover all cases, the compiler can't prove it:

```rust
match x {
    n if n > 0 => "positive",
    n if n < 0 => "negative",
    n if n == 0 => "zero",
    // ERROR: compiler still wants a catch-all `_`
}
```

---

## Pattern Types

### Literal and Range Patterns

```rust
match age {
    1 => "baby",
    2..=12 => "child",       // inclusive range
    13..=19 => "teenager",
    _ => "adult",
}
```

### Variable Binding

A name in a pattern **binds** the matched value to that name for use in the arm. This is not destructuring — it's giving the whole value a name.

```rust
match amount {
    a if a > 0.0 => println!("positive: {a}"),
//  ^ binds `amount` to `a` in this arm
    _ => println!("non-positive"),
}
```

### `@` Bindings

`@` lets you bind a value to a name **while also** testing it against a pattern:

```rust
match count {
    n @ 1..=9 => println!("{n} is single digit"),
    n @ 10..=99 => println!("{n} is double digit"),
    n @ 100.. => println!("{n} is large"),
    _ => println!("non-positive"),
}
```

Without `@`, you'd need a range pattern and a separate variable. `@` combines both: "match this range AND give me the value."

### Destructuring Enums

Pull apart enum variants to access inner data:

```rust
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
}

match shape {
    Shape::Circle(radius) => 2.0 * PI * radius,
    Shape::Rectangle(w, h) => 2.0 * (w + h),
}
```

### Destructuring Struct Variants

```rust
enum Vehicle {
    Car { brand: String, horsepower: u32 },
    Bicycle { gear_count: u8 },
}

match vehicle {
    Vehicle::Car { brand, horsepower } => format!("{brand} with {horsepower}hp"),
    Vehicle::Bicycle { gear_count } => format!("{gear_count} gears"),
}
```

### Tuple Matching

Match on multiple values at once:

```rust
match (a, b) {
    (Some(x), Some(y)) => Some(x + y),
    (Some(v), None) | (None, Some(v)) => Some(v),
    (None, None) => None,
}
```

### Or Patterns (`|`)

Combine multiple patterns in one arm:

```rust
match direction {
    Direction::North | Direction::South => "vertical",
    Direction::East | Direction::West => "horizontal",
}
```

---

## Guards (`if` after pattern)

A guard adds a boolean condition after the pattern. The arm only fires if the pattern matches **and** the guard is true:

```rust
match temp {
    t if t < 0 => "freezing",
    0..=15 => "cold",
    16..=25 => "pleasant",
    _ => "hot",
}
```

Guards are useful when patterns alone can't express the condition (e.g., comparing against another variable or using floats):

```rust
match amount {
    a if a > self.balance => Err(InsufficientFunds),
    a if a <= 0.0 => Err(InvalidAmount),
    _ => Ok(self.balance - amount),
}
```

---

## Matching on `Option<T>`

`Option` is an enum: `Some(T)` or `None`.

```rust
match val {
    Some(x) => println!("got {x}"),
    None => println!("nothing"),
}
```

---

## Matching on `Result<T, E>`

`Result` is an enum: `Ok(T)` or `Err(E)`.

```rust
match safe_divide(a, b) {
    Ok(value) => println!("result: {value}"),
    Err(MathError::DivisionByZero) => println!("can't divide by zero"),
    Err(e) => println!("error: {e:?}"),
}
```

### The `?` Operator

`?` is shorthand for "unwrap `Ok`, or `return Err` from the function":

```rust
// These are equivalent:
let value = match result {
    Ok(v) => v,
    Err(e) => return Err(e),
};

let value = result?;
```

---

## Nested and Combined `Result` / `Option`

### Chaining Results

When one function's output feeds into another:

```rust
fn sqrt_of_division(a: f64, b: f64) -> Result<f64, MathError> {
    match safe_divide(a, b) {
        Ok(quotient) => safe_sqrt(quotient),  // returns Result
        Err(e) => Err(e),
    }
}

// Or with `?`:
fn sqrt_of_division(a: f64, b: f64) -> Result<f64, MathError> {
    let quotient = safe_divide(a, b)?;
    safe_sqrt(quotient)
}
```

### Nested Options (`Option<Option<T>>`)

Common with `Vec::pop()` on a `Vec<Option<T>>`:

```rust
// stack.pop() returns Option<Option<i32>>
//   None           -> vec is empty
//   Some(None)     -> element was None
//   Some(Some(5))  -> element was Some(5)

while let Some(Some(num)) = stack.pop() {
    sum += num;
}
```

### Result inside a loop

Process a series of operations, stopping at first error:

```rust
for &amount in transactions {
    match amount {
        a if a > 0.0 => { account.deposit(a)?; },
        a if a < 0.0 => { account.withdraw(a.abs())?; },
        _ => return Err(BankError::InvalidAmount),
    }
}
Ok(account.balance)
```

---

## `if let` — Match a Single Pattern

When you only care about one variant:

```rust
// Instead of:
match shape {
    Shape::Circle(r) => println!("radius: {r}"),
    _ => {},
}

// Use:
if let Shape::Circle(r) = shape {
    println!("radius: {r}");
}
```

---

## `let-else` — Match or Return Early

Unwrap a pattern or bail out:

```rust
let Vehicle::Car { horsepower, .. } = vehicle else {
    return 0;  // must diverge: return, break, continue, or panic
};
// `horsepower` is available here
println!("{horsepower}hp");
```

---

## `while let` — Loop While Pattern Matches

Keep looping as long as the pattern matches:

```rust
while let Some(value) = stack.pop() {
    println!("{value}");
}
// loop ends when pop() returns None
```

---

## `matches!` Macro

Returns `true`/`false` for a pattern check. Great for boolean expressions:

```rust
let is_vertical = matches!(dir, Direction::North | Direction::South);

// With a guard:
let is_big_circle = matches!(shape, Shape::Circle(r) if r > 5.0);
```
