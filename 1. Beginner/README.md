# Additional notes

Read source code from a file

```rust
// This is a type-alias, but for now I've just set it to
// "nothing". Note: () is the Rust "unit" type. It's kind of
// like None in Python.
pub type Source = ();

pub fn read_source(filename: &str) -> Source {
    println!("Reading source");
}
```