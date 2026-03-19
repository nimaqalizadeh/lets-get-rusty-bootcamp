fn main() {

use anyhow::{bail, Context, Result};

// Here's how to use `with_context` to add more context to an error
let home = std::env::var("HOME")
    .with_context(|| "Could not read HOME environment variable")?;

// ...alternatively, use `bail` to return an error immediately 
let Ok(home) = std::env::var("HOME") else {
    bail!("Could not read HOME environment variable");
};

}