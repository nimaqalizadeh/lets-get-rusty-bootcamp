Scopes

The scope of a variable is the region of Rust code where that variable is valid, or alive.

The scope of a variable starts with its declaration. It ends when one of the following happens:

1. the block (i.e. the code between {}) where the variable was declared ends

```rust
    fn main() {
       // `x` is not yet in scope here
       let y = "Hello".to_string();
       let x = "World".to_string(); // <-- x's scope starts here...
       let h = "!".to_string(); //   |
    } //  <-------------- ...and ends here
```
2. ownership of the variable is transferred to someone else (e.g. a function or another variable)

```rust
    fn compute(t: String) {
       // Do something [...]
    }

    fn main() {
        let s = "Hello".to_string(); // <-- s's scope starts here...
                    //                    | 
        compute(s); // <------------------- ..and ends here
                    //   because `s` is moved into `compute`
    }
```

Destructors

When the owner of a value goes out of scope, Rust invokes its destructor.
The destructor tries to clean up the resources used by that value—in particular, whatever memory it allocated.

You can manually invoke the destructor of a value by passing it to std::mem::drop.
That’s why you’ll often hear Rust developers saying “that value has been dropped” as a way to state that a value has gone out of scope and its destructor has been invoked.
Visualizing drop points

We can insert explicit calls to drop to “spell out” what the compiler does for us. Going back to the previous example:

```rust
fn main() {
   let y = "Hello".to_string();
   let x = "World".to_string();
   let h = "!".to_string();
}
```

It’s equivalent to:

```rust
fn main() {
   let y = "Hello".to_string();
   let x = "World".to_string();
   let h = "!".to_string();
   // Variables are dropped in reverse order of declaration
   drop(h);
   drop(x);
   drop(y);
}
```

Let’s look at the second example instead, where s’s ownership is transferred to compute:

```rust
fn compute(s: String) {
   // Do something [...]
}

fn main() {
   let s = "Hello".to_string();
   compute(s);
}
```
It’s equivalent to this:

```rust
fn compute(t: String) {
    // Do something [...]
    drop(t); // <-- Assuming `t` wasn't dropped or moved 
             //     before this point, the compiler will call 
             //     `drop` here, when it goes out of scope
}

fn main() {
    let s = "Hello".to_string();
    compute(s);
}
```
Notice the difference: even though s is no longer valid after compute is called in main, there is no drop(s) in main. When you transfer ownership of a value to a function, you’re also transferring the responsibility of cleaning it up.

This ensures that the destructor for a value is called at most1 once, preventing double free bugs by design.