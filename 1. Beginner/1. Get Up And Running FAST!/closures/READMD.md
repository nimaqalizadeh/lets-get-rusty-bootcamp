# Closures

Note: Don't confuse the difference between capturing a variable and taking an argument in closures.
The way a closure captures its environment determines which of three special traits it imple-
ments: FnOnce, FnMut, or Fn.

```rust
fn main() {
    let name = String::from("Nima");
    let n = || name; // here the closure is capturing a variable by taking ownership and the trait is `impl FnOnce`
    n();
    //n() --> can not use it again because it has moved
}
```

```rust
fn main() {
    let name = String::from("Nima");
    let n = || &name; // here the closure is capturing a variable by borrowing immutably and the trait is `impl Fn`
    n();
    n();
}
```

```rust
fn main() {
    let mut name = String::from("Nima");
    let mut n = || name.pop(); // here the closure is capturing a variable borrowing it mutably so the trait is `impl FnMut`
    n();
    n();
}
```

```rust
fn main() {
    let name = String::from("Nima");
    let n = |x: String| x; // here the closure is taking an argument and the trait is `impl Fn`
    n(name);
}
```

Key insight: A closure taking a &mut argument doesn't make the closure FnMut. A closure is only FnMut when it mutably borrows a variable from the environment outside of its | | brackets so in this code:

```rust
fn main() {
    let mut name = String::from("Nima");
    let n = |x: &mut String| x.pop(); // here the closure is taking and argument and mutate it so the trait is `impl Fn`
    n(&mut name);
}
```

## All closures implement `FnOnce` Any closure that implements `Fn` also implements `FnMut`, and any closure that implements `FnMut` also implements `FnOnce`

It is all about **flexibility**. It means that if a function asks for a closure with a "lower" requirement, you are perfectly allowed to give it a closure with a "higher" capability.

Here is the easiest way to think about it: **What does the caller promise?**

### 1. Why `Fn` also implements `FnMut`

Imagine a function that asks for an `FnMut` closure. The function is essentially saying: _"I am going to call this closure, and I understand it **might** mutate some data."_

If you hand that function an `Fn` closure (which only reads data and mutates nothing), the function is perfectly happy! It allowed you to mutate, but you didn't need to.

- **The rule:** A closure that does _less_ (only reads) can always be used where a closure that does _more_ (mutates) is allowed.

### 2. Why `FnMut` also implements `FnOnce`

Imagine a function that asks for an `FnOnce` closure. The function is promising: _"I will call this closure **exactly one time**, and then never again."_

If you hand that function an `FnMut` or an `Fn` closure (which are perfectly capable of being called 100 times), the function is perfectly happy! It just calls it once and stops.

- **The rule:** A closure that can run _many times_ can obviously be run _just one time_.

---

### See it in action

Because of this rule, a function that accepts `FnOnce` is the **most accepting** function in Rust. It will take literally any closure you throw at it:

```rust
// This function promises to only call the closure one time.
fn execute_once<F>(closure: F)
where
    F: FnOnce()
{
    closure();
}

fn main() {
    let mut count = 0;
    let name = String::from("Nima");

    // 1. An Fn closure (just reads, can be called infinitely)
    let fn_closure = || println!("Reading: {name}");

    // 2. An FnMut closure (mutates 'count', can be called multiple times)
    let mut fn_mut_closure = || count += 1;

    // 3. An FnOnce closure (consumes 'name', can ONLY be called once)
    let fn_once_closure = || { let _consumed = name; };

    // Because of the rule you asked about, execute_once accepts ALL of them!
    execute_once(fn_closure);     // Fn implements FnOnce
    execute_once(fn_mut_closure); // FnMut implements FnOnce
    execute_once(fn_once_closure);
}

```

> **Key insight:** `FnOnce` is the lowest bar to clear — _every_ closure can be called at least once. `Fn` is the highest, strictest bar — it requires the closure to be perfectly safe to call anywhere, anytime, without messing up the surrounding code.

## The compiler will always infer the most permissive trait that a closure can implement. For example, a closure that only reads a variable will implement all three traits (Fn, FnMut, and FnOnce), allowing it to be used in the widest range of situations.
