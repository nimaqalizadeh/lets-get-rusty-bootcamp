# How think about async

The example is from [this](https://youtu.be/-m-pHRZ9iD0?si=4dDz4aSDpJLGBNu8) video:

**The Core Concept of Async**
The video introduces asynchronous programming by comparing it to everyday tasks like cooking. If you put a pizza in the oven, you don't stand at the oven door doing nothing until it's done. You move on to prepare the next pizza. Similarly, if you are making breakfast, doing everything sequentially (cook eggs, wait, fry bacon, wait, toast bread, wait) is a massive waste of time.

**Async vs. Multi-threading**
While one solution to speed up breakfast might be to hire a separate person for each task (one chef for the eggs, one for the bacon), this is equivalent to multi-threading. The video points out that dedicating an entire thread (or person) just to stand and stare at cooking eggs is a huge waste of system resources. Asynchronous programming allows a _single_ worker to initiate a task, switch to another task while waiting, and return to the first task once it requires attention again. It is concurrent, but not necessarily parallel.

**The Latency Problem**
To illustrate why async is crucial for I/O-bound tasks, the video breaks down hardware latency. A CPU accessing its L1 cache takes about 1 nanosecond. In comparison, fetching data from RAM takes 100 times longer, reading from an SSD is proportionally like waiting 4.5 hours, and waiting for a network request is like waiting 1.5 years.

Because network and database calls introduce massive wait times, spinning up a new thread for every single request is highly inefficient. When engineering backend services and APIs using frameworks like Axum or FastAPI, async programming is what allows a single thread to handle thousands of concurrent I/O-bound requests. Instead of freezing the system for "1.5 years" while waiting for a network payload or managing distributed cluster communications, the system immediately yields control to process other incoming requests, drastically improving throughput and resource management.

```
                               [CPU Registers   ]         Faster
Volatile                                v                Smaller
Random Access                  [  CPU Caches    ]         Expensive
Byte-Addressable                        v                     ^
...............................[      DRAM      ]..............|..
                                        v                     |
Non-Volatile                   [      SSD       ]              |
Sequential Access                       v                     |
Block-Addressable              [      HDD       ]              v
                                        v                  Slower
                               [Network Storage ]           Larger
                                                            Cheaper


======================================================================
                            ACCESS TIMES
            Latency Numbers Every Programmer Should Know
======================================================================

              1 ns  L1 Cache Ref        ======>  1 sec
              4 ns  L2 Cache Ref        ======>  4 sec
            100 ns  DRAM                ======>  100 sec
         16,000 ns  SSD                 ======>  4.4 hours
      2,000,000 ns  HDD                 ======>  3.3 weeks
    ~50,000,000 ns  Network Storage     ======>  1.5 years
  1,000,000,000 ns  Tape Archives       ======>  31.7 years
```

## Example

```rust
fn main() {
    let start: Instant = std::time::Instant::now();

    let _coffee: Coffee = pour_coffee();
    println!("☕ Coffee is ready");

    let _eggs: Egg = fry_eggs(2);
    println!("🍳 Eggs are ready");

    let _bacons: Bacon = fry_bacon(3);
    println!("🥓 Bacons are ready");

    let mut toast: Toast = toast_bread(2);
    apply_butter(&mut toast);
    apply_jam(&mut toast);
    println!("🥪 Toasts are ready");

    let _joice: Juice = pour_orange_joice();
    println!("🍹 Joice is ready");

    let duration: Duration = start.elapsed();
    println!("Breakfast is ready. It took {} seconds", duration.as_secs());
}
```

The same breakfast, written three ways. Each task takes a fixed amount of time so the totals are easy to compare:

| Task                | Time |
| ------------------- | ---- |
| `pour_coffee`       | 4 s  |
| `fry_eggs`          | 6 s  |
| `fry_bacon`         | 8 s  |
| `toast_bread`       | 3 s  |
| `apply_butter`      | 1 s  |
| `apply_jam`         | 1 s  |
| `pour_orange_juice` | 2 s  |

### 1. Sequential (no async, no threading)

One worker, one task at a time. Total time is the **sum** of every task: `4 + 6 + 8 + 3 + 1 + 1 + 2 = 25 s`.

```rust
use std::thread;
use std::time::{Duration, Instant};

struct Coffee;
struct Egg;
struct Bacon;
struct Toast;
struct Juice;

fn pour_coffee() -> Coffee {
    thread::sleep(Duration::from_secs(4));
    Coffee
}

fn fry_eggs(_n: u8) -> Egg {
    thread::sleep(Duration::from_secs(6));
    Egg
}

fn fry_bacon(_n: u8) -> Bacon {
    thread::sleep(Duration::from_secs(8));
    Bacon
}

fn toast_bread(_n: u8) -> Toast {
    thread::sleep(Duration::from_secs(3));
    Toast
}

fn apply_butter(_t: &mut Toast) {
    thread::sleep(Duration::from_secs(1));
}

fn apply_jam(_t: &mut Toast) {
    thread::sleep(Duration::from_secs(1));
}

fn pour_orange_juice() -> Juice {
    thread::sleep(Duration::from_secs(2));
    Juice
}

fn main() {
    let start = Instant::now();

    let _coffee = pour_coffee();
    println!("☕ Coffee is ready");

    let _eggs = fry_eggs(2);
    println!("🍳 Eggs are ready");

    let _bacons = fry_bacon(3);
    println!("🥓 Bacons are ready");

    let mut toast = toast_bread(2);
    apply_butter(&mut toast);
    apply_jam(&mut toast);
    println!("🥪 Toasts are ready");

    let _juice = pour_orange_juice();
    println!("🍹 Juice is ready");

    println!("Breakfast is ready. It took {} seconds", start.elapsed().as_secs());
}
```

### 2. Multi-threading with channels (`std::thread` + `mpsc`)

One OS thread per independent task, all sending their results back through a single `mpsc::channel`. The main thread receives them **in completion order** instead of a fixed `join` order, so we see each dish announced the moment its worker finishes. Total wall-clock is still the longest single chain — frying bacon at 8 s.

```rust
use std::sync::mpsc::channel;
use std::thread;
use std::time::Instant;

// (Coffee/Egg/Bacon/Toast/Juice structs and the blocking
//  worker functions from scenario 1 are reused as-is.)

enum Item {
    Coffee(Coffee),
    Eggs(Egg),
    Bacon(Bacon),
    Toast(Toast),
    Juice(Juice),
}

fn main() {
    let start = Instant::now();
    let (tx, rx) = channel::<Item>();

    {
        let tx = tx.clone();
        thread::spawn(move || tx.send(Item::Coffee(pour_coffee())).unwrap());
    }
    {
        let tx = tx.clone();
        thread::spawn(move || tx.send(Item::Eggs(fry_eggs(2))).unwrap());
    }
    {
        let tx = tx.clone();
        thread::spawn(move || tx.send(Item::Bacon(fry_bacon(3))).unwrap());
    }
    {
        let tx = tx.clone();
        thread::spawn(move || tx.send(Item::Juice(pour_orange_juice())).unwrap());
    }
    {
        let tx = tx.clone();
        thread::spawn(move || {
            let mut toast = toast_bread(2);
            apply_butter(&mut toast);
            apply_jam(&mut toast);
            tx.send(Item::Toast(toast)).unwrap();
        });
    }
    drop(tx); // drop the original Sender so `rx` ends after all 5 results

    for item in rx {
        match item {
            Item::Coffee(_) => println!("☕ Coffee is ready"),
            Item::Eggs(_)   => println!("🍳 Eggs are ready"),
            Item::Bacon(_)  => println!("🥓 Bacons are ready"),
            Item::Toast(_)  => println!("🥪 Toasts are ready"),
            Item::Juice(_)  => println!("🍹 Juice is ready"),
        }
    }

    println!("Breakfast is ready. It took {} seconds", start.elapsed().as_secs());
}
```

A few details worth pointing out:

- **`Sender` is `Clone`**, `Receiver` is not. Each worker gets its own clone of `tx`; the receiver stays in `main`.
- **`drop(tx)` is essential.** `for item in rx` only ends when _every_ `Sender` has been dropped. If we hold on to the original `tx`, the loop will hang forever after the 5 results.
- **The `Item` enum** lets us send heterogeneous values down a single channel — channels are typed, so without it we'd need one channel per result type (or `Box<dyn Any>`).
- **Output order is non-deterministic by design.** With the timings above, you'll see roughly: juice (2 s) → coffee (4 s) → toast (5 s) → eggs (6 s) → bacon (8 s). This is the channel demonstrating its value over `join()`: results are surfaced as they're ready instead of in a hard-coded order.

Five OS threads were spawned; four of them spend most of their life sleeping. That's the "hire one chef per dish, then have them stand there staring" problem from the video.

### 3. Async with the Tokio runtime

Here we mix two patterns:

- **Inline `.await`** for the quick drinks (`pour_coffee`, `pour_orange_juice`) — main awaits them sequentially.
- **`tokio::spawn`** for the slow dishes (`fry_eggs_async`, `fry_bacon_async`, `toast_bread_async`) — they run concurrently in the background and we hold their `JoinHandle`s, awaiting them later.

Each `sleep` inside an async fn is a yield point, so while a spawned task is waiting on its timer, the runtime drives the others forward on the same OS thread.

```rust
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use tokio::time::sleep;

struct Coffee;
struct Egg;
struct Bacon;
struct Toast;
struct Juice;

async fn pour_coffee() -> Coffee {
    sleep(Duration::from_secs(4)).await;
    Coffee
}

async fn fry_eggs_async(_n: u8) -> Egg {
    sleep(Duration::from_secs(6)).await;
    Egg
}

async fn fry_bacon_async(_n: u8) -> Bacon {
    sleep(Duration::from_secs(8)).await;
    Bacon
}

async fn toast_bread_async(_n: u8) -> Toast {
    sleep(Duration::from_secs(3)).await;
    Toast
}

fn apply_butter(_t: &mut Toast) {}
fn apply_jam(_t: &mut Toast) {}

async fn pour_orange_juice() -> Juice {
    sleep(Duration::from_secs(2)).await;
    Juice
}

#[tokio::main]
async fn main() {
    let start: Instant = std::time::Instant::now();

    let _coffee: Coffee = pour_coffee().await;
    println!("☕ Coffee is ready");

    let egg_handle: JoinHandle<Egg> = tokio::spawn(fry_eggs_async(2));
    let bacon_handle: JoinHandle<Bacon> = tokio::spawn(fry_bacon_async(3));
    let toast_handle: JoinHandle<Toast> = tokio::spawn(async {
        let mut toast: Toast = toast_bread_async(2).await;
        apply_butter(&mut toast);
        apply_jam(&mut toast);
        toast
    });

    let _juice: Juice = pour_orange_juice().await;
    println!("🍹 Juice is ready");

    let _egg: Egg = egg_handle.await.unwrap();
    println!("🍳 Eggs are ready");
    let _bacon: Bacon = bacon_handle.await.unwrap();
    println!("🥓 Bacons are ready");
    let _toast: Toast = toast_handle.await.unwrap();
    println!("🥪 Toasts are ready");

    let duration: Duration = start.elapsed();
    println!("Breakfast is ready. It took {} seconds", duration.as_secs());
}
```

**Timing walk-through:**

| Time   | What's happening                                                                 |
| ------ | -------------------------------------------------------------------------------- |
| `t=0`  | `pour_coffee().await` — main is parked here, runtime has nothing else to do.     |
| `t=4`  | Coffee done. We `tokio::spawn` eggs (6 s), bacon (8 s), toast (3 s) — all start. |
| `t=4`  | `pour_orange_juice().await` runs concurrently with the three spawned tasks.      |
| `t=6`  | Juice done. Toast finished at `t=7` is still pending; we now await `egg_handle`. |
| `t=10` | Eggs ready (started at 4, took 6).                                               |
| `t=12` | Bacon ready (started at 4, took 8). Toast handle is already complete.            |

**Total wall-clock: ~12 s.** Coffee is the cost we pay for awaiting it inline before kicking off the parallel work. If we'd spawned coffee too, we'd be back to ~8 s like scenario 2 — but on a single runtime thread.

A couple of notes:

- **`tokio::spawn` returns a `JoinHandle<T>`** and starts the future immediately on the runtime. `.await` on the handle yields the future's output (wrapped in a `Result` for cancellation/panic).
- **`apply_butter` / `apply_jam` are sync helpers** here. Inside the spawned `async` block we call them without `.await` — they're treated as instant transformations on the `Toast`.
- **Only one OS thread is required.** With `#[tokio::main]` you get the multi-thread runtime by default, but the same code works under `#[tokio::main(flavor = "current_thread")]` — proof that "concurrent" doesn't have to mean "parallel."

**Summary**

| Approach            | Wall-clock | OS threads used         |
| ------------------- | ---------- | ----------------------- |
| Sequential          | ~25 s      | 1                       |
| `std::thread`       | ~8 s       | 5 (4 mostly idle)       |
| `tokio` async/await | ~12 s      | 1 (works on any flavor) |

Based on the provided transcript, the speaker explains the concepts of asynchronous programming in Rust using a very practical, real-world analogy.

Here is what the video notes about Promises, Futures, and the examples used:

### The Examples He Uses

- **The Breakfast Analogy [00:00:00]:** The speaker uses the process of making breakfast—specifically pouring coffee/juice, frying eggs, frying bacon, and making toast—to explain how execution models work.
- **Synchronous (Blocking) Execution [00:03:19]:** He explains that in a blocking model, you start heating the pan for the eggs and just stand there watching it heat up, completely idle. You don't move on to the bacon or the toast until the eggs are 100% finished.
- **Asynchronous Execution [00:04:02]:** In contrast, the async approach involves starting the pan for the eggs, and instead of waiting idly, you move over to start the bacon or put the bread in the toaster. You only return to the eggs when they actually need your attention.
- **The "Multiple Cooks" Threading Analogy [00:07:21]:** Toward the end, he compares spawning separate OS threads to hiring three completely separate cooks just to make breakfast. While it gets the job done faster, he points out that it is a massive waste of resources compared to having a single cook multitask efficiently (which represents async).

### What He Says About "Promise"

- **Non-Blocking Guarantees [00:05:42]:** The speaker explains that in async programming, when you call a time-consuming function (like `fry_eggs`), the function does not block your progress and does not return the final result right away.
- **A Promise of an Egg [00:06:26]:** Because the eggs might not be ready for another 5 minutes, the system gives you a **"Promise"** (پرامیس / قول). It is a guarantee that the output will eventually be prepared and delivered to you later.

### What He Says About "Future"

- **Rust's Terminology [00:06:26]:** He explicitly points out that in the Rust programming language, this exact concept of a "Promise" is called a **"Future"** (فیوچر).
- **Returning a Future vs. a Value [00:07:21]:** He concludes by saying that if the synchronous, blocking version of the code returns an actual "cooked egg", the asynchronous version returns a "Future of a cooked egg"—meaning the object you currently hold is not an egg, but the promise that the egg will be completed in the future.

In this transcript, the speaker continues the "making breakfast" analogy to explain how to transition from a resource-heavy multi-threaded approach to a highly efficient **asynchronous** approach in Rust using the **Tokio** runtime.

### 1. The Problem with OS Threads (Multiple Cooks)

The speaker recaps that using standard OS threads (`std::thread`) is like hiring three separate cooks just to make one breakfast. While it gets the job done quickly, it is a massive waste of system resources because the OS has to manage all of them.

### 2. Async Functions and Futures

He explains that when you mark a function with the `async` keyword, it no longer blocks the program while it works. Instead of forcing you to wait 1 second and then handing you an egg, an async function returns immediately and hands you a **Future**—a promise that you will get an egg later.

### 3. The Role of `await` and Yielding

To actually get the result from a Future, you must use the `.await` keyword. The crucial feature of `.await` is that if a task needs to wait (e.g., waiting 300ms for a pan to heat up), it pauses _only that specific task_ and signals to the system: _"I am stuck right now; go do something else."_

### 4. Introducing the Tokio Runtime (The Head Chef)

Because the Rust standard library does not have a built-in way to execute async code, the speaker introduces **Tokio**, a popular third-party async runtime.

- He compares Tokio to a "head chef" or manager.
- When one task (like frying eggs) hits an `.await` and has to wait, Tokio instantly switches to another task (like frying bacon or making toast).
- Tokio runs on top of the OS threads and manages thousands of these lightweight tasks without the OS even knowing about them.

### 5. Practical Implementation

The speaker walks through converting the breakfast code to Tokio:

- **`tokio::spawn`:** Used to kick off the independent breakfast tasks concurrently, returning a handle for each.
- **Async Blocks (`async { ... }`):** Used to group multiple async steps together (like toasting bread, then applying butter and jam) into a single Future.
- **`#[tokio::main]` Macro:** He explains that a standard Rust `main` function cannot run async code directly. This macro sets up the Tokio runtime behind the scenes so you can `.await` inside `main`.

He brings up a fundamental rule of asynchronous programming in Rust: **You can only call an async function inside another async function**

He then explains the specific challenge this creates:

- **The `main` Function Problem:** By default, a standard Rust `main` function is synchronous. Therefore, if you try to call your async breakfast functions directly inside `main`, you will hit a roadblock. You cannot call an async function and wait for it inside a normal, synchronous function.
- **The Solution:** To solve this, you need a way to make your `main` function asynchronous. He explains that we use macros (specifically `#[tokio::main]`) to fix this. This macro sets up the Tokio runtime in the background and transforms your `main` function into an async one, creating the necessary environment to call all your other async functions.

In software engineering, "runtime" can refer to a few different levels of infrastructure, but they all share the same purpose: bridging the gap between your compiled code and the operating system.

Here are the key responsibilities a runtime typically handles:

- **Interfacing with the OS:** Your code rarely talks directly to the hardware. The runtime handles the heavy lifting of requesting memory from the operating system, writing to the file system, or opening network sockets.
- **Memory Management:** In languages like Java or Python, the runtime includes a Garbage Collector that actively runs alongside your code to clean up unused memory.
- **Concurrency and Scheduling:** As you saw with Tokio, an async runtime manages an event loop. It decides which task gets CPU time, parks tasks that are waiting on I/O (like a network request), and wakes them up when they are ready.

### How this applies to different ecosystems

Because you work across different stacks, you've likely interacted with several different types of runtimes:

**1. "Heavy" Runtimes (Python, Node.js, Java)**
These languages require a massive runtime environment. When you run a Python script, you are booting up the CPython runtime. It reads your code, interprets it, handles every single memory allocation, manages the Global Interpreter Lock (GIL), and translates your commands into OS-level system calls. Node.js does the same thing using the V8 engine and libuv for its event loop.

**2. "Minimal" Runtimes (C, C++, Rust)**
Compiled systems languages have an incredibly small standard runtime. In Rust, the standard runtime does very little: it sets up stack overflow guards, handles command-line arguments, and manages basic panic handling. It does _not_ include a garbage collector or an async event loop.

**3. "Pluggable" Async Runtimes (Tokio in Rust)**
Because Rust's standard runtime is bare-bones, it doesn't know how to handle `Future`s or async tasks. That’s why you have to bring your own "manager" to handle them. **Tokio** is an async runtime that you plug into your Rust binary. It provides the thread pool and the `epoll` reactor needed to efficiently juggle thousands of async tasks on a single OS thread.

In short, whenever your code is actively doing something, it relies on a runtime to interact with the machine and schedule its tasks.
