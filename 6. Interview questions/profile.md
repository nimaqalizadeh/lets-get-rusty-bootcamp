Profiles

https://rust-exercises.com/100-exercises/02_basic_calculator/08_overflow

https://huonw.github.io/blog/2016/04/myths-and-legends-about-integer-overflow-in-rust/


A profile is a set of configuration options that can be used to customize the way Rust code is compiled.

Cargo provides 4 built-in profiles: dev, release, test, and bench.
The dev profile is used every time you run cargo build, cargo run or cargo test. It’s aimed at local development, therefore it sacrifices runtime performance in favor of faster compilation times and a better debugging experience.
The release profile, instead, is optimized for runtime performance but incurs longer compilation times. You need to explicitly request via the --release flag—e.g. cargo build --release or cargo run --release. The test profile is the default profile used by cargo test. The test profile inherits the settings from the dev profile. The bench profile is the default profile used by cargo bench. The bench profile inherits from the release profile. Use dev for iterative development and debugging, release for optimized production builds,
test for correctness testing, and bench for performance benchmarking.


“Have you built your project in release mode?” is almost a meme in the Rust community.
It refers to developers who are not familiar with Rust and complain about its performance on social media (e.g. Reddit, Twitter) before realizing they haven’t built their project in release mode.


You can also define custom profiles or customize the built-in ones.
overflow-check

By default, overflow-checks is set to:

    true for the dev profile
    false for the release profile

This is in line with the goals of the two profiles.
dev is aimed at local development, so it panics in order to highlight potential issues as early as possible.
release, instead, is tuned for runtime performance: checking for overflows would slow down the program, so it prefers to wrap around.

At the same time, having different behaviours for the two profiles can lead to subtle bugs.
Our recommendation is to enable overflow-checks for both profiles: it’s better to crash than to silently produce incorrect results. The runtime performance hit is negligible in most cases; if you’re working on a performance-critical application, you can run benchmarks to decide if it’s something you can afford.

