# Builder pattern

The Builder pattern is useful when constructing an object with many fields, especially if many are optional or require complex setup.

Instead of a single complex constructor with a long list of parameters, the builder provides a clean, step-by-step, self-documenting process. Each optional parameter is set with a descriptive method call (e.g., `.width()`, `.resizable()`).

## How it works

1. Define the **target struct** with all fields (required and optional).
2. Define a **builder struct** where optional fields are wrapped in `Option<T>`.
3. The builder's `new()` takes only required fields, setting optional ones to `None`.
4. Each optional field gets a setter method that takes `mut self`, sets the value, and returns `self` — enabling method chaining (fluent interface).
5. A `build()` method consumes the builder and produces the final struct, applying defaults for any unset optional fields via `unwrap_or()`.

## Example

```rust
#[derive(Debug)]
pub struct WindowConfig {
    title: String,
    width: u32,
    height: u32,
    is_resizable: bool,
    has_decorations: bool,
}

pub struct WindowConfigBuilder {
    title: String,              // Required field
    width: Option<u32>,
    height: Option<u32>,
    is_resizable: Option<bool>,
    has_decorations: Option<bool>,
}

impl WindowConfigBuilder {
    // Start building with the required field(s)
    pub fn new(title: String) -> Self {
        WindowConfigBuilder {
            title,
            width: None,
            height: None,
            is_resizable: None,
            has_decorations: None,
        }
    }

    // Methods to set optional fields, consuming and returning self (fluent interface)
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.is_resizable = Some(resizable);
        self
    }

    pub fn decorations(mut self, decorations: bool) -> Self {
        self.has_decorations = Some(decorations);
        self
    }

    // Finalize the build, providing defaults for unset options
    pub fn build(self) -> WindowConfig {
        WindowConfig {
            title: self.title,
            width: self.width.unwrap_or(800),
            height: self.height.unwrap_or(600),
            is_resizable: self.is_resizable.unwrap_or(true),
            has_decorations: self.has_decorations.unwrap_or(true),
        }
    }
}

fn main() {
    let basic_window = WindowConfigBuilder::new("My App".to_string())
        .build(); // Uses all defaults

    let custom_window = WindowConfigBuilder::new("Game Window".to_string())
        .width(1024)
        .height(768)
        .resizable(false)
        .build(); // Sets some fields, uses defaults for others

    let fullscreen_window = WindowConfigBuilder::new("Fullscreen".to_string())
        .decorations(false) // Only set decorations
        .build();

    println!("Basic Window: {:?}", basic_window);
    println!("Custom Window: {:?}", custom_window);
    println!("Fullscreen Window: {:?}", fullscreen_window);
}
```

## Key takeaways

- **Required vs optional**: Required fields go in `new()`, optional fields use setter methods.
- **Fluent interface**: Each setter takes `mut self` and returns `Self`, allowing chained calls like `.width(1024).height(768)`.
- **Defaults**: The `build()` method uses `unwrap_or()` to provide sensible defaults for any fields that weren't explicitly set.
- **Ownership**: The builder consumes itself on `build()`, preventing reuse of a partially-configured builder (enforced at compile time).

## Real-world use cases

- **HTTP request/response construction**: Libraries like `reqwest` use builders to construct HTTP requests — you chain `.method()`, `.header()`, `.body()`, `.timeout()` etc. Only the URL is required; everything else has sensible defaults.
- **Database query builders**: ORMs and query libraries (e.g., Diesel, SQLx) let you build queries step by step: `Query::select("users").where_("age > 18").order_by("name").limit(10)`. Each call is optional and adds a clause.
- **GUI/window configuration**: Exactly like the example above — creating windows, dialogs, or widgets with many optional properties (size, position, title, decorations, fullscreen mode).
- **Configuration objects**: Server configs, logging configs, CLI argument parsers — anything with many optional settings and reasonable defaults. For example, building a `ServerConfig` with `.port(8080).max_connections(100).tls(true)`.
- **Test data/fixtures**: Building test objects with specific fields set while relying on defaults for everything else: `UserBuilder::new("alice").role(Admin).build()`.
- **Document/report generation**: Constructing complex documents piece by piece — adding headers, footers, sections, styles — before calling `.build()` to produce the final output.

# State pattern

The State pattern handles objects whose behavior depends on their internal state. In many languages, this is done with flags or string fields (e.g., `if post.status == "draft"`), which is fragile and can lead to runtime errors if the state isn't checked before calling a method.

### Why string-based state is fragile (Python example)

Consider the same `Post` example in Python using string-based state:

```python
class Post:
    def __init__(self):
        self.status = "draft"
        self.content = ""

    def add_text(self, text):
        if self.status == "draft":
            self.content += text

    def request_review(self):
        if self.status == "draft":
            self.status = "pending_review"

    def approve(self):
        if self.status == "pending_review":
            self.status = "published"
```

This approach breaks down in several ways:

- **Typos are silent bugs**: `self.status = "pubilshed"` or `post.status == "daft"` — Python won't complain, the code just runs incorrectly.
- **No exhaustiveness checking**: Adding a new state (e.g., `"archived"`) gives you no warning about all the `if/elif` chains you forgot to update. In Rust, adding a new variant to an enum causes every `match` on that enum to fail to compile until you handle the new case — the compiler forces you to cover every variant. Python's `if/elif` chains have no such guarantee, so forgotten cases are silently ignored.
- **Anyone can set any state at any time**: Nothing prevents `post.status = "published"` (skipping review) or `post.status = "banana"` (not even a real state). There's no enforcement of valid transitions.
- **State-specific data is unstructured**: If only `pending_review` posts need a `reviewer` field, you'd have `self.reviewer = None` on all posts and hope you check before accessing it.

Python relies entirely on programmer discipline. Rust's enum approach makes invalid states **unrepresentable in the type system**.

Rust's enums and `match` system offer a powerful and safe way to implement the State pattern. Instead of relying on simple flags, you encode the various states directly within the type system, making the code both robust and clear. The compiler can verify state transitions and ensure certain methods are only called when the object is in the correct state — catching bugs at compile time instead of at runtime.

## How it works

1. Define a **separate struct for each state** (e.g., `DraftPost`, `PendingReviewPost`, `PublishedPost`), each holding the data relevant to that state.
2. Define an **enum** that wraps all possible states as variants.
3. The **main struct** holds the current state as an instance of this enum.
4. Methods on the main struct use `if let` or `match` to check the current state and only allow valid actions or transitions.
5. State transitions use `std::mem::replace` to temporarily take ownership of the state from `&mut self.state`, allowing you to move data out of one variant and into another while upholding Rust's ownership rules.

## Example

```rust
// Define a struct for each state
struct DraftPost { content: String }
struct PendingReviewPost { content: String }
struct PublishedPost { content: String }

// Enum wrapping all possible states
enum PostState {
    Draft(DraftPost),
    PendingReview(PendingReviewPost),
    Published(PublishedPost),
}

pub struct Post {
    state: PostState,
}

impl Post {
    pub fn new() -> Post {
        Post { state: PostState::Draft(DraftPost { content: String::new() }) }
    }

    pub fn add_text(&mut self, text: &str) {
        // Only allowed in Draft state
        // Uses `ref mut` instead of `std::mem::replace` because we're just
        // modifying data inside the current state, not changing to a different state.
        // `ref mut` borrows the inner DraftPost mutably — no ownership transfer needed.
        // In contrast, transition methods like `request_review` use `std::mem::replace`
        // because they need to take ownership of the old state's data to construct
        // an entirely new state variant.
        //
        // Note: `ref mut` is pattern syntax for binding by mutable reference (left side of `=`).
        // Don't confuse it with `&mut`, which is expression syntax (right side of `=`).
        // In modern Rust (edition 2018+), you can use match ergonomics instead:
        //   if let PostState::Draft(draft) = &mut self.state { ... }
        // which automatically binds `draft` as `&mut DraftPost`.
        if let PostState::Draft(ref mut draft) = self.state {
            draft.content.push_str(text);
        } else {
            println!("Cannot add text in current state.");
        }
    }

    pub fn request_review(&mut self) {
        // Transition from Draft to PendingReview
        if let PostState::Draft(draft) = std::mem::replace(
            &mut self.state,
            PostState::Draft(DraftPost { content: String::new() }),
        ) {
            self.state = PostState::PendingReview(PendingReviewPost { content: draft.content });
        } else {
            println!("Post must be in Draft state to request review.");
        }
    }

    pub fn approve(&mut self) {
        // Transition from PendingReview to Published
        if let PostState::PendingReview(pending) = std::mem::replace(
            &mut self.state,
            PostState::Draft(DraftPost { content: String::new() }),
        ) {
            self.state = PostState::Published(PublishedPost { content: pending.content });
        } else {
            println!("Post must be Pending Review to approve.");
        }
    }

    pub fn content(&self) -> &str {
        match &self.state {
            PostState::Draft(s) => &s.content,
            PostState::PendingReview(s) => &s.content,
            PostState::Published(s) => &s.content,
        }
    }
}

fn main() {
    let mut post = Post::new();

    post.add_text("Learning about state patterns in Rust. ");
    println!("Content (Draft): {}", post.content());

    post.request_review();
    post.add_text("This won't be added."); // Tries adding text in wrong state

    post.approve();
    println!("Content (Published): {}", post.content());

    post.request_review(); // Tries invalid transition
}
```

## Why `std::mem::replace`?

In transition methods like `request_review`, we can't simply move `self.state` out because `self` is borrowed mutably — Rust won't let you leave `self.state` in an uninitialized state. `std::mem::replace` solves this by swapping in a temporary placeholder value while giving us ownership of the original. This lets us destructure the old state, extract its data, and construct the new state — all without violating ownership rules.

`std::mem::replace` signature:

```rust
pub fn replace<T>(dest: &mut T, src: T) -> T
```

It takes a mutable reference to a location (`dest`) and a new value (`src`), puts `src` into `dest`, and returns the old value that was in `dest`. Think of it like swapping something on a shelf — you put a new item on, and get the old item back.

### Step-by-step: how `request_review` works

Starting state: `self.state` is `PostState::Draft(DraftPost { content: "Learning..." })`

1. `replace` swaps in the placeholder — `self.state` is now `PostState::Draft(DraftPost { content: "" })`
2. `replace` returns the original — `PostState::Draft(DraftPost { content: "Learning..." })`
3. `if let PostState::Draft(draft)` destructures the returned value — `draft` is `DraftPost { content: "Learning..." }`
4. `self.state` is overwritten with `PostState::PendingReview(PendingReviewPost { content: draft.content })` — the text moves into the new state
5. The empty placeholder from step 1 is dropped and discarded

## Key takeaways

- **Type-safe states**: Each state is a distinct struct wrapped in an enum, so invalid states are unrepresentable.
- **Compile-time enforcement**: `match` and `if let` ensure you handle every state, and the compiler flags missing cases.
- **No runtime flags**: Unlike string/boolean-based approaches, there are no forgotten checks or invalid state combinations.
- **Ownership-friendly transitions**: `std::mem::replace` enables moving data between states while satisfying Rust's borrow checker.
- **More robust than flags**: This approach prevents entire categories of bugs that arise from unchecked state in traditional OOP languages.

## State machine

The State pattern is an implementation of a **state machine** — a model where something can only be in **one state at a time** and moves between states based on specific events or actions.

A state machine has:

1. A **finite set of states** (e.g., `Draft`, `PendingReview`, `Published`)
2. **Transitions** between states (e.g., `Draft -> PendingReview -> Published`)
3. **Rules** about which transitions are valid (e.g., can't go from `Published` back to `Draft`)
4. **Behavior** that depends on the current state (e.g., `add_text` only works in `Draft`)

Real-world examples:

```
Traffic light:  Red --timer--> Green --timer--> Yellow --timer--> Red
Door:           Locked --unlock--> Closed --open--> Open --close--> Closed --lock--> Locked
Post:           Draft --request_review--> PendingReview --approve--> Published
```

The State pattern encodes these rules in the type system so the compiler enforces them, rather than relying on the programmer to remember them.

## Real-world use cases

- **Order processing**: An e-commerce order moves through `Created -> PaymentPending -> Paid -> Shipped -> Delivered` (or `Cancelled`/`Refunded` from certain states). Each state determines what actions are valid — you can't ship an unpaid order or refund a delivered one.
- **Network connections**: A TCP connection transitions through `Closed -> Listening -> SynReceived -> Established -> Closing -> Closed`. Each state has different valid operations — you can only send data in the `Established` state.
- **Authentication flows**: A user session moves through `Anonymous -> Authenticating -> Authenticated -> Expired`. Only authenticated sessions can access protected resources; expired sessions must re-authenticate.
- **Document/content workflows**: Blog posts, legal contracts, or any content with an approval pipeline — `Draft -> InReview -> Approved -> Published` (or `Rejected -> Draft`). Different roles have permissions tied to each state.
- **Game entity states**: A character can be `Idle -> Walking -> Running -> Jumping -> Falling -> Idle`. Each state determines which animations play, what input is accepted, and which transitions are possible.
- **Payment processing**: A payment goes through `Initiated -> Processing -> Succeeded`/`Failed`/`Refunded`. Each state carries different data (e.g., only `Succeeded` has a transaction ID, only `Failed` has an error reason).

# Observer pattern

The Observer pattern creates a one-to-many relationship between objects: when the main object (the **subject**) changes state, all its dependents (the **observers**) are automatically notified and updated. This is perfect for building event-driven systems where different parts of an app can respond to changes without being tightly coupled to the object that changes.

In Rust, this pattern is implemented using traits and smart pointers.

## How it works

1. Define an **`Observer` trait** with an `update()` method that the subject will call on each observer.
2. Define a **`Subject` struct** that holds its own state and a collection of observers.
3. The observer collection uses **`RefCell<Vec<Rc<dyn Observer>>>`** — a combination of smart pointers that solves several ownership challenges at once.
4. The subject provides an `attach()` method to register observers and a `set_state()` method that updates the state and notifies all observers.
5. **Concrete observers** (e.g., `Logger`, `Notifier`) implement the `Observer` trait, each providing their own reaction logic in `update()`.

## Why `RefCell<Vec<Rc<dyn Observer>>>`?

This type looks complex, but each layer serves a purpose:

- **`Rc`** (Reference Counted): Enables shared ownership — multiple parts of the application can hold a reference to the same observer. Without `Rc`, the subject would take exclusive ownership of observers, and no one else could use them.
- **`RefCell`**: Provides interior mutability — it allows the observer list to be modified (e.g., adding a new observer via `attach()`) even through an immutable reference to the `Subject`. Rust's borrow checker normally prevents mutation through `&self`, but `RefCell` moves the borrow check to runtime.
- **`dyn Observer`**: A trait object that allows the `Vec` to store different concrete types of observers (such as `Logger` and `Notifier`) in the same collection, as long as they all implement the `Observer` trait.

For multi-threaded environments, you would use `Mutex<Vec<Arc<dyn Observer>>>` instead — `Mutex` replaces `RefCell` (wrapping the `Vec` for thread-safe interior mutability), and `Arc` replaces `Rc` (wrapping each observer for thread-safe reference counting).

## Example

```rust
use std::rc::Rc;
use std::cell::RefCell;

// The trait that all observers must implement.
trait Observer {
    // The subject calls this method to notify the observer of a change.
    fn update(&self, new_state: &str);
}

// The subject holds the state and a list of observers.
struct Subject {
    state: String,
    observers: RefCell<Vec<Rc<dyn Observer>>>,
}

impl Subject {
    fn new(initial_state: &str) -> Self {
        Subject {
            state: initial_state.to_string(),
            observers: RefCell::new(Vec::new()),
        }
    }

    // Add a new observer to the list.
    fn attach(&self, observer: Rc<dyn Observer>) {
        self.observers.borrow_mut().push(observer);
    }

    // Change the state and notify all observers.
    fn set_state(&mut self, new_state: &str) {
        self.state = new_state.to_string();
        println!("\nSubject: State changed to '{}'. Notifying observers...", self.state);
        // We borrow the observers list immutably to iterate and notify.
        for observer in self.observers.borrow().iter() {
            observer.update(&self.state);
        }
    }
}

// A concrete observer that logs updates.
struct Logger {
    name: String,
}

impl Observer for Logger {
    fn update(&self, new_state: &str) {
        println!("[Logger {}]: Received update! New state is: '{}'", self.name, new_state);
    }
}

// Another concrete observer that might perform a different action.
struct Notifier {
    email: String,
}

impl Observer for Notifier {
    fn update(&self, new_state: &str) {
        println!("[Notifier]: Sending email to {}. Subject: State changed to '{}'", self.email, new_state);
    }
}

fn main() {
    let mut subject = Subject::new("Initial State");

    // Create observers. We wrap them in Rc to manage shared ownership.
    let logger = Rc::new(Logger { name: "FileLogger".to_string() });
    let notifier = Rc::new(Notifier { email: "admin@example.com".to_string() });

    // Attach the observers to the subject.
    subject.attach(Rc::clone(&logger) as Rc<dyn Observer>);
    subject.attach(Rc::clone(&notifier) as Rc<dyn Observer>);

    // Change the subject's state. This should trigger notifications.
    subject.set_state("State A");
    subject.set_state("State B");
}
```

## Execution flow

1. `main()` creates a `Subject` and two observer instances (`Logger` and `Notifier`), each wrapped in `Rc`.
2. Both observers are attached to the subject via `attach()`, which pushes cloned `Rc` pointers into the observer list.
3. When `set_state("State A")` is called, the subject updates its internal state, then iterates through the observer list calling `update()` on each one.
4. Each observer reacts in its own way — `Logger` prints a log message, `Notifier` simulates sending an email.
5. The same happens when `set_state("State B")` is called — all observers are notified again with the new state.

## Key takeaways

- **Decoupled communication**: Observers don't need to know about each other, and the subject doesn't need to know the concrete types of its observers — only that they implement the `Observer` trait.
- **Dynamic registration**: Observers can be attached (and potentially detached) at runtime, making the system flexible.
- **Smart pointer necessity**: Rust's ownership model requires explicit tools (`Rc`, `RefCell`) to achieve the shared, mutable state that the Observer pattern demands. This makes the ownership semantics visible and safe, unlike languages where shared mutable references are implicit (and error-prone).
- **Trait objects for polymorphism**: `dyn Observer` allows storing heterogeneous observer types in the same `Vec`, which is the Rust equivalent of interface-based polymorphism in OOP languages.

## Real-world use cases

- **UI/frontend frameworks**: When data changes in a model, all bound UI components (text fields, charts, labels) update automatically. This is the backbone of reactive frameworks (React's state, Android's LiveData, SwiftUI's `@Published`).
- **Event systems / message brokers**: A button click notifies all registered handlers. Kafka, RabbitMQ, and Redis Pub/Sub are built on this idea at a larger scale — publishers emit events, subscribers react independently.
- **Logging and monitoring**: A service emits events (errors, metrics, state changes), and multiple observers handle them differently: one writes to a file, another sends to Slack, a third updates a dashboard — exactly like the `Logger`/`Notifier` example above.
- **Stock/price tickers**: A price feed (subject) pushes updates to multiple displays, alert systems, and trading algorithms (observers) simultaneously.
- **Configuration hot-reloading**: A config file watcher (subject) detects changes and notifies all dependent services (observers) to reload their settings without restarting.
- **Game engines**: An entity takes damage (state change), and multiple systems respond: the health bar updates, a sound effect plays, a particle effect spawns, the AI recalculates threat. Each system is an observer of the "damage taken" event.
- **MVC/MVVM architecture**: The Model is the subject, and Views are observers. When the model changes, all views reflecting that data refresh themselves. This is one of the original motivations for the pattern.
