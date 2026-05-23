# Rust Mock Interview: Insights from Alice Ryhl

_Alice Ryhl — Software Engineer at Google (Android Rust Team) · Core maintainer of Tokio · Rust Language Team Adviser_

> Source: ["Why Rust is different, with Alice Ryhl" — The Pragmatic Engineer Podcast](https://www.youtube.com/watch?v=q9xD36NCtZ8)

---

## Tokio: An Overview _(04:09)_

**Q: What is Tokio and what role does it play in the Rust ecosystem?**

**A:** Tokio is an asynchronous runtime for Rust. You can think of it as the standard library for Rust when you're writing async code. It provides a queue of tasks that are ready to run and executes them one after another — and unlike JavaScript, Tokio can be multi-threaded, so you can have multiple queues running in parallel.

---

**Q: How does Tokio compare to JavaScript's event loop?**

**A:** If you compare with JavaScript in the browser, you might compare Tokio with the browser itself. In JavaScript, you have an event loop with all the tasks that can run, and they get executed one after the other. If you use `await`, tasks can pause and another task starts running on the same thread. Tokio does something similar — but the key difference is that Tokio can be multi-threaded, whereas JavaScript's event loop is single-threaded.

---

## What Alice Likes About Rust _(05:11)_

**Q: Why do people say "when it compiles, it works" about Rust?**

**A:** This has to be in quotes because obviously bugs are still possible, but there's a reason people say it. It comes down to Rust's type system and how aggressively it catches mistakes at compile time. Even compared to other languages with type systems, Rust does a better job. For example, Java's `null` — Tony Hoare invented it and called it his "billion-dollar mistake" because every time you call a function, your program might crash due to a null pointer. In Rust, that problem simply doesn't exist. The compiler forces you to handle the case where a value might be absent, so you can't forget like you can in Java.

---

**Q: How does Rust handle null differently from Java?**

**A:** In Rust, you have to explicitly say "this value might be absent" using an `Option` type. The compiler will then force you to check for the absence before you use it. So you can't forget — if you try to use the value without checking, it's a compiler error. In Java, you might call a function and have no idea it might return null until your program crashes in production.

---

**Q: How does Rust handle errors differently from languages that use exceptions?**

**A:** Rust doesn't use exceptions. Instead, it returns errors as values — specifically using a `Result<T, E>` type, which is an enum that is either the result or the error. There's a `?` operator which makes this ergonomic: you write `my_function()?` and it means "if this fails, return the error." So error handling is explicit — not zero characters like with exceptions — but you can't forget to handle it either, because ignoring it is a compiler error. It's the best of both worlds: explicit but not verbose.

---

**Q: How does Rust prevent documentation examples from going out of date?**

**A:** In Rust, you write documentation comments with three slashes (`///`) instead of two. The special thing is that any code examples you write inside your documentation are automatically compiled and run as tests. So if you change the underlying code and your example no longer works, your test suite fails. You literally cannot forget to update your examples — the build will break if they go stale. It's the first language where there's an actual enforced solution to the eternal problem of documentation drifting away from code.

---

**Q: What is exhaustive pattern matching and why does it matter for reliability?**

**A:** In Rust, it's called `match` instead of `switch`, but the key difference is that `match` is exhaustive — if you have an enum, you must handle every possible variant. If you forget one, it's a compiler error. You can have a catch-all case if you want, but most of the time you list all the cases explicitly. The real benefit shows up when you add a new variant to an enum in the future: the compiler will tell you every single place in the codebase you need to update. This is how Rust actively helps with refactoring — change the type, then just fix the compiler errors until it stops complaining.

---

**Q: How does Rust make refactoring safer compared to other languages?**

**A:** If you're refactoring something — say you change a return type or restructure a type — you just make the change and fix the compiler errors. Once the compiler is happy, you've updated every place that needed updating. This is really powerful because you can't accidentally miss a call site. Rust is very good at telling you all the places you need to update. That's the deeper pattern: the language designers thought hard about what mistakes programmers actually keep making, and they try to eliminate those mistakes through the compiler.

---

## Rust for TypeScript Engineers _(12:48)_

**Q: Where does Rust fit in the stack for a TypeScript engineer?**

**A:** Rust fits in as the backend language. I wouldn't use it on the frontend — it's a pretty good fit for backends and API servers. The pitch is: you don't want to be woken up at night because there are problems with your web server. You need a language that's reliable, that's going to have as few bugs as possible. That's the idea of Rust.

---

**Q: What specific Rust features would a TypeScript developer appreciate?**

**A:** A few things stand out. First, null safety — TypeScript is actually reasonably good here, but Rust is strict about it at the type level. Second, error handling with the `?` operator: errors are values you can't ignore, but it's still convenient to propagate them. Third, exhaustive `match` — TypeScript doesn't enforce this nearly as strictly. Fourth, documentation tests: examples in docs that automatically become tests. And fifth, the refactoring story — the compiler guides you through every change that needs to be made. There's also `serde`, a library that generates highly efficient, type-safe JSON parsers for your structs at compile time, so you get both safety and performance.

---

## Moving from C++ to Rust _(13:51)_

**Q: What is the pitch for Rust to a C++ developer?**

**A:** From C++, the pitch is even stronger. In JavaScript or TypeScript, if you make a mistake you might take down your server — which is already bad. But in C++, when you make a mistake, it's usually a security vulnerability. Something as trivial as an off-by-one error in an array index can be a critical security vulnerability. That just keeps happening. In Rust, you get memory safety: a whole class of bugs — the kind that typically lead to security vulnerabilities — simply cannot occur in safe Rust code.

---

## Memory Safety _(14:34)_

**Q: What is memory safety in Rust?**

**A:** Memory safety means that no matter how "stupid" the code you write is, you will never have a certain class of bugs. Specifically, bugs like reading past the end of an array, using an object after it's been freed (use-after-free), or writing to the wrong memory — the kinds of bugs that almost always lead to security vulnerabilities. Rust guarantees at compile time that these bugs cannot happen in safe code.

---

**Q: Can you give a real-world example of how a memory safety bug becomes a security exploit?**

**A:** A classic example in the Linux kernel: suppose you have some object and you manage to free it, but then the memory gets reused by the kernel for a `task_struct` — which represents a running process and contains a field called `user_id`. It's very common for code to write zeros to memory. If an attacker can craft a situation where a zero is written to that `user_id` field, the process now runs as root. That's a full privilege escalation from a relatively trivial memory bug. Once an attacker achieves that, they can take over the entire system. Rust's memory safety eliminates this entire class of exploits.

---

## Garbage Collection Tradeoffs _(18:12)_

**Q: What are the tradeoffs of using a garbage collector?**

**A:** A garbage collector (GC) is a piece of code that periodically checks all your objects, figures out which ones are no longer used, and cleans them up. The tradeoff is that this checking has overhead and happens at unpredictable times. For embedded use cases — like firmware or the Linux kernel — a GC might simply be impossible or unacceptable, because you need fine-grained control over memory and timing. Even for backend use, a GC can cause latency spikes: if a request comes in right when the GC runs, it takes much longer to reply than usual.

---

**Q: How does Rust manage memory without a garbage collector?**

**A:** In Rust (and C++), a variable is cleaned up at the end of its scope — when it goes out of scope. There's no "later" cleanup. This is made safe by the ownership and borrow checker system, which ensures at compile time that memory is always cleaned up exactly once, at the right time, without needing runtime tracking.

---

## Ownership, References, and Borrowing _(21:46)_

**Q: What is the ownership model in Rust?**

**A:** Ownership is the idea that if you have a variable containing some object, that object has exactly one owner — it's exclusive. For example, if you have `let a = some_string` and then write `let b = a`, that's a _move_. The contents of `a` are moved to `b`, and using `a` afterwards is a compiler error. This matters because without a GC, when `b` goes out of scope, it cleans up the string. If `a` were also valid, both would try to clean it up when they go out of scope, which would be a double-free bug. Ownership prevents that at compile time.

---

**Q: What is reference counting (`Arc`) and when would you use it?**

**A:** `Arc` stands for Atomically Reference Counted, and it's one of Rust's pointer types. The idea is you have some object in memory, and `Arc` gives you a pointer to it along with a counter. When you clone an `Arc`, you increment the counter — now two `Arc`s point to the same underlying memory without copying the data. When one goes out of scope, the counter decrements. When the counter reaches zero, the object is cleaned up. You use it when there's no single place that "owns" the data — multiple parts of your code need to share it. It's also the way to create cyclic data structures in Rust.

---

## **Q: What is borrowing and the borrow checker?**

## **A:** A _reference_ in Rust is a pointer to an object that does no runtime checking — it's purely a compile-time construct. Creating a reference is called _borrowing_. The borrow checker is the part of the compiler that ensures two things: (1) a reference can never outlive the object it points to, and (2) you can have either one mutable borrow or any number of immutable borrows at a time — never both simultaneously. For example, if you have an immutable reference to element 5 of a vector and then call `.clear()` on that vector, the borrow checker will prevent you from using that reference afterwards, because the element it pointed to no longer exists.

---

**Q: What do newcomers to Rust most commonly struggle with?**

**A:** The biggest stumbling block is how to design data structures. With code logic you can mostly do the same things as in other languages, but data structures are different. In TypeScript you might have a `Book` that contains an array of `Page`s, and each `Page` has a back-reference to its `Book` — a cyclic relationship. In Rust, you generally have to design your objects so they form a tree (no cycles). When newcomers try to reproduce cyclic structures without tools like `Arc`, they get a wall of compiler errors and keep trying to change the code — when the real solution is to change the struct design itself.

---

**Q: What does "fighting the borrow checker" mean, and how do you resolve it?**

**A:** "Fighting the borrow checker" is when you can't get your code to compile because it keeps throwing borrow-related errors. A common cause is having a reference inside a struct and trying to pass that struct across multiple functions — the compiler can't easily verify the reference's lifetime across function boundaries. The recurring lesson is: the solution is usually to change the data structure, not to keep tweaking the code. For instance, switching from a plain reference to `Arc` often resolves these situations, because `Arc` makes shared ownership explicit.

---

## Unsafe in Rust _(26:59)_

**Q: What does the `unsafe` keyword do in Rust?**

**A:** `unsafe` is the escape hatch. Without any use of `unsafe`, Rust guarantees that no matter how bad your code is, you will never have a memory safety bug. When you write an `unsafe` block, you unlock a small set of additional operations — like calling functions marked `unsafe fn`, dereferencing raw pointers, or performing unchecked array access. Each of these operations comes with rules you must follow yourself. If you break those rules, you might introduce a memory safety bug. But critically, `unsafe` does **not** disable the borrow checker or other compiler checks — it just gives you a few more things you can do.

---

**Q: Does `unsafe` disable the borrow checker?**

**A:** No. The borrow checker still runs in full inside an `unsafe` block. `unsafe` only unlocks a narrow set of additional operations. Everything the compiler normally checks still applies.

---

**Q: When is it appropriate to use `unsafe`?**

**A:** In most backend servers you'd have zero uses of `unsafe`. It generally shows up when you're implementing something that adds a new capability to the language. The classic example is: imagine Rust didn't have `Vec`. It does have functions to allocate memory, read from a pointer, and free memory. You could use those — all `unsafe` operations — to build your own `Vec`, wrapping them in a safe public API. Another common use is calling into a C library (FFI), where you wrap the C API safely so callers never have to touch `unsafe` themselves.

---

**Q: How can you wrap `unsafe` code behind a safe public API?**

**A:** The key is field privacy and API design. When you implement something like `Vec`, the raw pointer and length fields are private — nobody outside the module can touch them directly. You expose only safe methods that enforce the invariants internally. As long as your API doesn't permit callers to violate the rules, users of your type can't cause memory safety bugs no matter what they do. This is how Rust lets you add new "language features" via libraries without compromising the overall safety guarantee.

---

## Crates and Cargo _(31:21)_

**Q: What is a crate in Rust?**

**A:** "Crate" is just the Rust word for a package — a unit of compiled code that you can share and depend on.

---

**Q: Where is the Rust ecosystem most mature, and where is it weakest?**

**A:** Most mature: backend services, CLI tools, embedded systems, and increasingly the Linux kernel. These are all areas where Rust is a very natural fit. Least mature: the frontend. There have been attempts to compile Rust to WebAssembly and use it as a TypeScript replacement in the browser, but that ecosystem isn't ready. If building a web product today, Rust makes sense for the backend; TypeScript still makes sense for the frontend.

---

## Language Design and RFCs _(35:55)_

**Q: How does Rust make language decisions without a single "benevolent dictator"?**

**A:** Rust has a set of teams — the language team, the library API team, the compiler team, dev tools, and others — that collectively govern the project. Each team is responsible for its domain. When something is contentious or cross-cutting, it gets discussed at events like Rust Week's "All Hands," where all the Rust developers come together. If a team doesn't sign off, it doesn't happen. The teams have also been good at delegating — for example, saying "this is a library API decision, we'll go with whatever the library team decides."

---

**Q: What is an RFC in Rust and how is it structured?**

**A:** RFC stands for Request for Comments. It's a structured document used for big language or ecosystem decisions. It has a template with these key sections:

- **Summary** — a brief description
- **Motivation** — why this feature is needed
- **Guide-level explanation** — explain the feature as if writing a tutorial, as though it already exists
- **Reference-level explanation** — explain it as if it were in the language reference, more technically
- **Rationale and alternatives** — why this design vs. others? Answering questions before they're asked
- **Prior art** — what did C++ or other languages do?
- **Future possibilities** — what could come next?

The guide-level and reference-level explanations force you to think from the user's perspective, not just the implementer's perspective.

---

## Building New Features _(43:02)_

**Q: How does a language feature go from RFC to stable Rust?**

**A:** Once an RFC is accepted, the implementation begins. The feature is put behind a _feature flag_ in nightly Rust — you can only use it if you're on the nightly compiler and opt in to the unstable feature. People experiment with it, bugs are found and fixed. When the feature feels ready, a _stabilization report_ is written: it documents how the feature has been used, identifies dangerous edge cases, and lists tests for those edges. The team then runs an FCP on the stabilization PR. Once that FCP passes, the feature flag is removed and the feature is in the main branch. Within 0–6 weeks it enters a beta build, and 6 weeks after that it ships in a stable release.

---

**Q: What is the role of nightly builds and feature flags in Rust's release process?**

**A:** Nightly builds are the way experimental features are made available without being "official" stable Rust. A feature flag (enabled with `#![feature(some_feature)]`) is required to use an unstable feature, signaling that it might change. This lets the community experiment and give feedback before a feature is locked in. Feature flags also protect users on stable Rust — they can't accidentally use an API that might break.

---

**Q: How does Rust's 6-week release cycle work?**

**A:** Rust ships a new stable release every 6 weeks like clockwork. The cycle is: changes land in `main`, a beta branch is cut every 6 weeks from whatever state `main` is in, and the previous beta branch becomes the new stable release. There's no concept of "beta features" — you're either on nightly (all flags available) or stable (only stabilized features). The 6-week beta window exists purely to catch any regressions before they reach stable users.

---

## Editions vs. Versions _(46:30)_

**Q: What are editions in Rust?**

**A:** Editions are a way for Rust to make breaking changes to the language syntax without breaking existing code. So far there have been editions in 2015, 2018, 2021, and 2024. Each crate can declare which edition it's written in. The crucial difference from traditional versioning: crates from different editions can be mixed together freely and they interoperate. A library written in the 2021 edition and a binary using the 2024 edition can depend on each other without any issues.

---

**Q: How do editions allow breaking changes without breaking existing code?**

**A:** The key example is the `async` keyword. Before the edition that introduced it, you could name a variable `async` — it was just an identifier. After the edition, `async` became a reserved keyword. If Rust had applied that change globally, all existing code using `async` as a variable name would break. With editions, code on the older edition keeps working as before, while new code on the newer edition can use `async`/`await`. Old and new code can still be compiled and linked together.

---

**Q: Why did Rust choose editions rather than traditional major version bumps (like Python 2 → 3)?**

**A:** The core idea is: we want old code to keep working forever, and we want to keep evolving the language. The Python 2→3 split is the cautionary tale — it fractured the ecosystem for over a decade. Rust's edition system solves this by allowing the mix-and-match of code from different editions in the same compiled binary. You never have a hard split where old code can't be used with new code. Rust has an extremely strong backwards compatibility guarantee: code you write today should keep compiling indefinitely.
