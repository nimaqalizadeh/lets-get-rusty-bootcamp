# Additional notes

You can define the unit tuple as a type and use it as a function's return type — useful when you're in the early stages of a project and designing the overall structure
```rust
// This is a type-alias, but for now I've just set it to
// "nothing". Note: () is the Rust "unit" type. It's kind of
// like None in Python.
pub type Source = ();

pub fn read_source(filename: &str) -> Source {
    println!("Reading source");
}
```