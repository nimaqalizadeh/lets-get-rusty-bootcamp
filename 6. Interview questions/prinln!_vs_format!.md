The core difference is where the formatted output goes: `println!` sends it directly to the console (standard output), while `format!` returns it as a new String variable in memory.

For thoroughness, the `std::fmt` formatting syntax is composed of two parts:

```bash
{<position-or-name>:<format>}
```

where:

`<position-or-name>` can be the argument position: `println!("Hello {0}!", "world");`, note that it is checked at compile-time
`<position-or-name>` can also be a name: `println!("Hello {arg}!", arg = "world");`
`<format>` is [one of the following](https://doc.rust-lang.org/stable/std/fmt/index.html#formatting-traits) formats, where each format requires the argument to implement a specific trait, checked at compile-time

The default, in the absence of position, name or format, is to pick the argument matching the index of {} and to use the Display trait. There are however various traits! From the link above:

    nothing ⇒ Display
    ? ⇒ Debug
    o ⇒ Octal
    x ⇒ LowerHex
    X ⇒ UpperHex
    p ⇒ Pointer
    b ⇒ Binary
    e ⇒ LowerExp
    E ⇒ UpperExp

and if necessary new traits could be added in the future.