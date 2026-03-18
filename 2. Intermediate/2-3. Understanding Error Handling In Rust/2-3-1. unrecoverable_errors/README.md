# Panic

A good insight from Davi Beazley about error handling: 
I you know that something is going to fail, coding will be a lot easier if you account for it early on as supposed to later on. 

```rust
fn main() {
    panic!("Something went wrong!")
}
```

```bash
thread 'main' (46794) panicked at src/main.rs:2:5:
Something went wrong!
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

```bash
RUST_BACKTRACE=1 cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/panic_test`

thread 'main' (47158) panicked at src/main.rs:2:5:
Something went wrong!
stack backtrace:
   0: __rustc::rust_begin_unwind
   1: core::panicking::panic_fmt
   2: panic_test::main
             at ./src/main.rs:2:5
   3: core::ops::function::FnOnce::call_once
             at /usr/src/debug/rust/rustc-1.93.1-src/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.
```

Backtrace shows the chain of function calls causing our program to panic. To read it, look for your crate name (`panic_test::`) — that frame points to the exact location in your code. Frames above it are Rust internals and can be ignored.

The top two frames (`rust_begin_unwind` and `panic_fmt`) are the internals of the `panic!` macro itself. `panic_fmt` runs first — it formats your panic message into a structured payload. Then `rust_begin_unwind` takes over — it starts unwinding the stack, running `Drop` implementations, and printing the panic message and backtrace to stderr. Both are always present in any panic and can be safely ignored.

**Unwinding the stack** means Rust walks back up the call stack frame by frame, and for each frame runs the `Drop` implementation of any local variables (freeing memory, closing files, releasing locks, etc.), then removes the frame and moves to the caller. This ensures resources are cleaned up properly even during a crash. The alternative is `abort` — configured via `panic = "abort"` in `Cargo.toml` — which kills the process immediately without any cleanup, producing faster and smaller binaries.

**Stack frames and call order**

The call stack is a region of memory that tracks which functions are currently running. Every time a function is called, a stack frame is pushed onto it containing the function's local variables, return address, and arguments. Every time a function returns, its frame is popped off.

Given this code:

```rust
fn main() { a(); }
fn a()    { b(); }
fn b()    { panic!("oh no"); }
```

The stack at the moment of panic (top = most recent):

```
b       ← panicked here (frame 0 in backtrace)
a       ← called b  (frame 1)
main    ← called a  (frame 2)
```

Unwinding walks this in reverse — starting from `b` down to `main`, dropping locals and popping each frame along the way. This is why backtrace frame numbers start at `0` for the most recent call and increase as you go back toward `main`.

The last frame (`core::ops::function::FnOnce::call_once`) is the runtime glue between Rust's entry point and your `main`. Rust doesn't call `main` like a plain C function — it wraps it in `FnOnce::call_once` so the runtime can set up panic handling, flush stdout, and return the exit code properly before and after your code runs. The full call chain is:

```
_start                      ← OS hands control to the binary
  └─ lang_start             ← Rust runtime sets up the environment
       └─ FnOnce::call_once ← runtime calls main through the trait
            └─ main         ← your code runs
```

This frame is always present and can be safely ignored when debugging.

For more information can use `RUST_BACKTRACE=full`. Compared to the short backtrace, the full one shows:

- **Raw memory addresses** (e.g. `0x5606eefb24e2`) — the actual location of each function in memory at runtime
- **Mangled symbol hashes** (e.g. `::h10482faeba4fc5f9`) — unique identifiers Rust appends to function names to avoid naming collisions
- **More frames** — the short backtrace hides many internal frames; the full one exposes everything

```bash
RUST_BACKTRACE=full cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/panic_test`

thread 'main' (48932) panicked at src/main.rs:2:5:
Something went wrong!
stack backtrace:
   0:     0x5606eefb24e2 - <std::sys::backtrace::BacktraceLock::print::DisplayBacktrace as core::fmt::Display>::fmt::h10482faeba4fc5f9
   1:     0x5606eefc1027 - core::fmt::write::h4aabe3a6a7b4f5b9
   2:     0x5606eef91ac6 - std::io::Write::write_fmt::h26713ae0cad9bf58
   3:     0x5606eef98166 - std::panicking::default_hook::{{closure}}::hab4c9297d5e5052f
   4:     0x5606eef97fc6 - std::panicking::default_hook::h5673ad5b77d18f9b
   5:     0x5606eef983ab - std::panicking::panic_with_hook::h30c4d524ceba2905
   6:     0x5606eef9825a - std::panicking::panic_handler::{{closure}}::hdf3d031ac08113f6
   7:     0x5606eef96969 - std::sys::backtrace::__rust_end_short_backtrace::hc254b29afcfa9315
   8:     0x5606eef86a5d - __rustc[ee045879db8e283c]::rust_begin_unwind
   9:     0x5606eefc3c5c - core::panicking::panic_fmt::h332dc4998dcc78fa
  10:     0x5606eef86615 - panic_test::main::ha5bcaf8d061b4fe0
                               at /tmp/panic_test/src/main.rs:2:5
  11:     0x5606eef866ab - core::ops::function::FnOnce::call_once::h9a0ab970d3db4dfc
                               at /usr/src/debug/rust/rustc-1.93.1-src/library/core/src/ops/function.rs:250:5
  12:     0x5606eef866be - std::sys::backtrace::__rust_begin_short_backtrace::hf3b29ed2fd33046a
                               at /usr/src/debug/rust/rustc-1.93.1-src/library/std/src/sys/backtrace.rs:160:18
  13:     0x5606eef86721 - std::rt::lang_start::{{closure}}::hebac77c6fb2607ec
                               at /usr/src/debug/rust/rustc-1.93.1-src/library/std/src/rt.rs:206:18
  14:     0x5606eef929f6 - std::rt::lang_start_internal::h0bf50c01c8a654de
  15:     0x5606eef86707 - std::rt::lang_start::h5ce2d4beb5225420
                               at /usr/src/debug/rust/rustc-1.93.1-src/library/std/src/rt.rs:205:5
  16:     0x5606eef8663e - main
  17:     0x7fa0ce77a6c1 - <unknown>
  18:     0x7fa0ce77a7f9 - __libc_start_main
  19:     0x5606eef864e5 - _start
  20:                0x0 - <unknown>
```

| Frames | What it is |
|--------|-----------|
| `0-2`  | Printing the backtrace itself to stderr |
| `3-5`  | The default panic hook — formats and prints the panic message |
| `6`    | The panic handler closure |
| `7`    | `__rust_end_short_backtrace` — a marker Rust uses to know where to stop the short backtrace |
| `8-9`  | Same as short backtrace: `rust_begin_unwind` and `panic_fmt` |
| `10`   | Your code (`panic_test::main`) |
| `11`   | Same as short backtrace: `FnOnce::call_once` |
| `12`   | `__rust_begin_short_backtrace` — a marker for where the short backtrace starts |
| `13-15`| `lang_start` — the Rust runtime startup |
| `16`   | The C `main` entry point (before Rust takes over) |
| `17-18`| `__libc_start_main` — the C runtime handing control to `main` |
| `19`   | `_start` — the very first instruction the OS executes in the binary |
| `20`   | Null terminator, always ignored |

The full backtrace is useful when the short one doesn't show enough context, particularly when debugging panics inside dependencies.
