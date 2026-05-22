# Control flow

## `loop`

If a loop is just doing work (like printing, modifying external variables, or waiting for a timer), you don't need to bind it to anything. You just use break; to stop it.

If the loop is actively calculating or retrieving a value, you need to "catch" that value by binding it to a variable with let — otherwise, the value just vanishes into the void.

Here are the three ways this plays out in real code:

### 1. The "Action" Loop (No binding)

Used when the loop just _does_ things and doesn't calculate a final answer.

```rust
fn main() {
    let mut counter = 0;

    // No 'let' binding needed
    loop {
        counter += 1;
        println!("Counting: {counter}");

        if counter == 3 {
            break; // Just stops the loop, returns nothing
        }
    }
}

```

### 2. The "Calculation" Loop (Binding to a variable)

Used when the loop's entire purpose is to find a specific value.

```rust
fn main() {
    let mut counter = 0;

    // We bind the loop to 'result' to catch the value
    let result = loop {
        counter += 1;

        if counter == 3 {
            break counter * 10; // Stops the loop AND throws out the value 30
        }
    }; // Semicolon required here!

    println!("Result is {result}");
}

```

### 3. The "Direct Return" Loop (No binding, but returning)

There is one exception where you return a value from a loop _without_ using let. If the loop is the last expression in a function, you can use it to return a value directly out of the function itself!

```rust
fn find_magic_number() -> i32 {
    let mut counter = 0;

    // No 'let', no semicolon at the end!
    // The loop evaluates to an i32, which becomes the function's return value.
    loop {
        counter += 1;

        if counter == 3 {
            break counter * 10;
        }
    }
}

```
