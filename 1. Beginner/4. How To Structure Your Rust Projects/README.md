# Project Structure Overview

## Table of Contents

- [Basic Components](#basic-components)
- [Comprehensive Example: `auth_service`](#comprehensive-example-auth_service)
  - [Project Tree](#project-tree)
  - [Code](#code)
  - [Running the Binaries](#running-the-binaries)
  - [How the Module Tree Maps to `use` Paths](#how-the-module-tree-maps-to-use-paths)
- [Cargo Workspaces](#cargo-workspaces)
  - [When to Use a Workspace vs `src/bin/`](#when-to-use-a-workspace-vs-srcbin)
  - [Workspace Project Tree](#workspace-project-tree)
  - [Workspace Code](#workspace-code)
  - [Running Workspace Packages](#running-workspace-packages)
  - [Key Benefits of Workspaces](#key-benefits-of-workspaces)
- [Combining Both Approaches](#combining-both-approaches)
- [Summary: How to Choose](#summary-how-to-choose)
- [`cargo install` vs `cargo add`](#cargo-install-vs-cargo-add)
  - [Where Does the Source Code Go?](#where-does-the-source-code-go)
- [Module Keywords & Path Syntax](#module-keywords--path-syntax)
  - [Examples in Context](#examples-in-context)
- [درسنامه فارسی](#درسنامه-فارسی)

## Basic Components

- **Packages**: Contain one or more crates that provide a set of functionality. Packages allow you to build, test, and share crates.
  - **Cargo.toml** - Describes the package and defines how to build crates.
  - **Rules**
    - Must have at least 1 crate
    - At most 1 library crate
    - Any number of binary crates
  - This means the `src` directory must have at least one of:
    1. `main.rs` only — a single binary crate
    2. `lib.rs` only — a single library crate (cannot be run with `cargo run` — exists only to be used as a dependency by other crates that have a `main.rs`, or tested with `cargo test`)
    3. `lib.rs` + `main.rs` — one library crate and one binary crate
    4. `lib.rs` and/or `main.rs` + additional binaries in `src/bin/` — any number of binary crates
    5. Any combination of the above + module files (e.g. `utils.rs`) in `src/` without a `main` function — these are modules, not crates
- **Crates**: A tree of modules that produces a library or executable.
- **Modules**: Let you control the organization, scope, and privacy.
  - Organize code for readability and reuse
  - Control scope and privacy
  - Contain items (functions, structs, enums, traits, etc.)
  - Explicitly defined (using the `mod` keyword)
    - Not mapped to the file system
    - Flexibility & straightforward conditional compilation

  > **Note on `mod` vs `import`:** Coming from Python, `mod` declarations might look similar to an `import` statement. However, something else is really going on. Instead of importing code, `mod` is merely declaring the existence of a submodule and is required so that the Rust compiler can actually find code in other files. You only need to specify `mod` in one place (usually `main.rs` or `lib.rs`) and nowhere else in the program. If you have a C background, `mod` declarations are closer to declarations that one would put in a Makefile.

  > **`mod` means the file is part of the build:** The Rust compiler only compiles files that are part of the module tree. If a file is never declared with `mod`, the compiler completely ignores it — even if it contains errors. For example, if you create a file `example.rs` with a type error:
  >
  > ```rust
  > fn bad_fn(x: i32) -> i32 {
  >     x * 2.5  // type error: can't multiply i32 by f64
  > }
  > ```
  >
  > and you never add `mod example;` in `main.rs` or `lib.rs`, the entire project will compile without any errors. The compiler doesn't know that file exists. Only once you add `mod example;` does the compiler include `example.rs` in the build, and then you'll get the type error. This is what makes `mod` different from Python's `import` — it's not about using code, it's about telling the compiler "this file is part of the build." Without the `mod` declaration, the `.rs` file is just a text file sitting in your directory as far as `rustc` is concerned.

  **Module resolution order** — when you write `mod foo;`, Rust looks for the module in these locations (mutually exclusive, not a fallback chain):

  1. **Inline** — `mod foo { ... }` block in the current file
  2. **`foo.rs`** — sibling file (new style, Rust 2018+)
  3. **`foo/mod.rs`** — directory with `mod.rs` (old style)

  If both `foo.rs` and `foo/mod.rs` exist, the compiler **errors** — you must use one or the other.

  Modules can be defined in five ways:

  1. **Inline in `main.rs` or `lib.rs`** — define the module directly in the file:

     ```rust
     mod auth {
         pub fn login() {
             println!("Logging in...");
         }
     }

     fn main() {
         auth::login();
     }
     ```

  2. **As a separate file in `src/`** — create `src/auth.rs` and declare it with `mod auth;`:

     ```
     src/
     ├── main.rs
     └── auth.rs
     ```

     ```rust
     // src/auth.rs
     pub fn login() {
         println!("Logging in...");
     }
     ```

     ```rust
     // src/main.rs
     mod auth;

     fn main() {
         auth::login();
     }
     ```

  3. **As a directory with `mod.rs`** — create `src/auth/mod.rs` (**old style, pre-2018 edition**):

     ```
     src/
     ├── main.rs
     └── auth/
         └── mod.rs
     ```

     ```rust
     // src/auth/mod.rs
     pub fn login() {
         println!("Logging in...");
     }
     ```

     ```rust
     // src/main.rs
     mod auth;

     fn main() {
         auth::login();
     }
     ```

  4. **A file in `src/` + a directory with the same name for submodules** (**new style, Rust 2018+, recommended**) — create `src/auth.rs` and `src/auth/` directory containing submodule files:

     ```
     src/
     ├── main.rs
     ├── auth.rs
     └── auth/
         └── auth_utils.rs
     ```

     ```rust
     // src/auth/auth_utils.rs
     pub fn validate_token(token: &str) -> bool {
         !token.is_empty()
     }
     ```

     ```rust
     // src/auth.rs
     pub mod auth_utils;

     pub fn login() {
         let valid = auth_utils::validate_token("my_token");
         println!("Token valid: {}", valid);
     }
     ```

     ```rust
     // src/main.rs
     mod auth;

     fn main() {
         auth::login();
         let valid = auth::auth_utils::validate_token("test");
         println!("Direct check: {}", valid);
     }
     ```

  > **Option 3 vs Option 4:** Both approaches work, but option 4 is the preferred modern style. The old `mod.rs` approach (option 3) leads to multiple files all named `mod.rs` in your editor, making it hard to tell them apart. The new style (option 4) gives each module file a unique name (e.g. `auth.rs` instead of `auth/mod.rs`), which is much easier to navigate. **Do not mix both styles for the same module** — use one or the other.

  5. **Using the `#[path]` attribute** — map any file to any module name, ignoring the default naming conventions (**rarely used, not recommended**):

     ```
     src/
     ├── main.rs
     └── my_custom_folder/
         └── authentication.rs
     ```

     ```rust
     // src/my_custom_folder/authentication.rs
     pub fn login() {
         println!("Logging in...");
     }
     ```

     ```rust
     // src/main.rs
     #[path = "my_custom_folder/authentication.rs"]
     mod auth;

     fn main() {
         auth::login();
     }
     ```

  > **Why avoid `#[path]`?** It breaks the convention that file names reflect module names, making the project harder to navigate. Other developers won't be able to find modules by looking at the file structure alone.

## Comprehensive Example: `auth_service`

A realistic project that demonstrates all concepts — packages, crates, modules (with nested submodules), and multiple binaries.

### Project Tree

```
auth_service/
├── Cargo.toml
└── src/
    ├── lib.rs               # library crate root — exposes all modules
    ├── main.rs              # default binary — the main server
    ├── config.rs            # module: app configuration
    ├── db.rs                # module: database connection
    ├── auth.rs              # module: auth logic (has submodules)
    ├── auth/                # directory for auth submodules (new style)
    │   ├── jwt.rs           # submodule: JWT token handling
    │   └── password.rs      # submodule: password hashingg
    └── bin/
        ├── migrate.rs       # separate binary: database migrations
        └── generate_key.rs  # separate binary: generate secret keys
```

### Code

**`Cargo.toml`** — package definition:

```toml
[package]
name = "auth_service"
version = "0.1.0"
edition = "2021"
```

---

**`src/config.rs`** — a simple module with a struct:

```rust
pub struct AppConfig {
    pub db_url: String,
    pub port: u16,
    pub jwt_secret: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        AppConfig {
            db_url: String::from("postgres://localhost/auth_db"),
            port: 8080,
            jwt_secret: String::from("super_secret_key"),
        }
    }
}
```

**`src/db.rs`** — another simple module:

```rust
use crate::config::AppConfig;

pub struct DbPool {
    pub url: String,,
}

impl DbPool {
    pub fn connect(config: &AppConfig) -> Self {
        println!("Connecting to database: {}", config.db_url);
        DbPool {
            url: config.db_url.clone(),
        }
    }

    pub fn run_migrations(&self) {
        println!("Running migrations on: {}", self.url);
    }
}
```

---

**`src/auth/jwt.rs`** — a submodule for JWT handling:

```rust
pub fn create_token(user_id: u64, secret: &str) -> String {
    format!("jwt_token_{}_{}", user_id, secret)
}

pub fn verify_token(token: &str, secret: &str) -> bool {
    token.contains(secret)
}
```

**`src/auth/password.rs`** — a submodule for password hashing:

```rust
pub fn hash(password: &str) -> String {
    format!("hashed_{}", password)
}

pub fn verify(password: &str, hashed: &str) -> bool {
    hashed == format!("hashed_{}", password)
}
```

**`src/auth.rs`** — the parent module, declaring its submodules and providing a public API:

```rust
pub mod jwt;
pub mod password;

use crate::config::AppConfig;

pub struct AuthService {
    jwt_secret: String,
}

impl AuthService {
    pub fn new(config: &AppConfig) -> Self {
        AuthService {
            jwt_secret: config.jwt_secret.clone(),
        }
    }

    pub fn register(&self, email: &str, raw_password: &str) -> String {
        let hashed = password::hash(raw_password);
        println!("Registered {} with password hash: {}", email, hashed);
        jwt::create_token(1, &self.jwt_secret)
    }

    pub fn login(&self, email: &str, raw_password: &str) -> Option<String> {
        let stored_hash = password::hash(raw_password); // simulated lookup
        if password::verify(raw_password, &stored_hash) {
            println!("Login successful for {}", email);
            Some(jwt::create_token(1, &self.jwt_secret))
        } else {
            println!("Login failed for {}", email);
            None
        }
    }
}
```

---

**`src/lib.rs`** — the library crate root, exposing all modules:

```rust
pub mod config;
pub mod db;
pub mod auth;
```

**`src/main.rs`** — the default binary, using the library to start a server:

```rust
use auth_service::config::AppConfig;
use auth_service::db::DbPool;
use auth_service::auth::AuthService;

fn main() {
    let config = AppConfig::from_env();
    let _pool = DbPool::connect(&config);
    let auth = AuthService::new(&config);

    println!("Server starting on port {}...", config.port);

    // simulate a registration and login
    let token = auth.register("user@example.com", "my_password");
    println!("Registration token: {}", token);

    if let Some(token) = auth.login("user@example.com", "my_password") {
        println!("Login token: {}", token);
    }
}
```

---

**`src/bin/migrate.rs`** — a separate binary for running database migrations:

```rust
use auth_service::config::AppConfig;
use auth_service::db::DbPool;

fn main() {
    println!("=== Database Migration Tool ===");
    let config = AppConfig::from_env();
    let pool = DbPool::connect(&config);
    pool.run_migrations();
    println!("Migrations complete!");
}
```

**`src/bin/generate_key.rs`** — a separate binary for generating secret keys:

```rust
fn main() {
    println!("=== Secret Key Generator ===");
    let key: String = (0..32)
        .map(|_| format!("{:x}", rand::random::<u8>()))
        .collect();
    println!("Generated key: {}", key);
    println!("Add this to your environment variables as JWT_SECRET");
}
```

### Running the Binaries

```bash
cargo run                        # runs src/main.rs (the server)
cargo run --bin migrate          # runs src/bin/migrate.rs
cargo run --bin generate_key     # runs src/bin/generate_key.rs
```

### How the Module Tree Maps to `use` Paths

```
auth_service                     # crate root (lib.rs)
├── auth_service::config         # config.rs
├── auth_service::db             # db.rs
├── auth_service::auth           # auth.rs
│   ├── auth_service::auth::jwt      # auth/jwt.rs
│   └── auth_service::auth::password # auth/password.rs
```

## Cargo Workspaces

When a project grows beyond a single package, you can use a **workspace** to manage multiple packages in one repository. Unlike `src/bin/` (multiple binaries in one package), a workspace holds **multiple independent packages**, each with their own `Cargo.toml`.

### When to Use a Workspace vs `src/bin/`

| | `src/bin/` | Workspace |
|---|---|---|
| **Scope** | Multiple binaries in one package | Multiple independent packages |
| **`Cargo.toml`** | Single `Cargo.toml` | Each package has its own `Cargo.toml` |
| **Shared code** | Via the package's `lib.rs` | Via a shared library package as a dependency |
| **Use case** | Small CLI tools related to the same crate | Large projects with distinct services or libraries |

### Workspace Project Tree

```
my_workspace/
├── Cargo.toml               # workspace root (no [package], just [workspace])
├── Cargo.lock               # shared lock file for all packages
├── target/                  # shared build directory
├── auth_service/
│   ├── Cargo.toml           # independent package (binary)
│   └── src/
│       └── main.rs
├── api_gateway/
│   ├── Cargo.toml           # independent package (binary)
│   └── src/
│       └── main.rs
└── shared_utils/
    ├── Cargo.toml           # independent package (library)
    └── src/
        └── lib.rs
```

### Workspace Code

**Root `Cargo.toml`** — defines the workspace members (no `[package]` section):

```toml
[workspace]
members = [
    "auth_service",
    "api_gateway",
    "shared_utils",
]
```

**`shared_utils/Cargo.toml`** — a shared library package:

```toml
[package]
name = "shared_utils"
version = "0.1.0"
edition = "2021"
```

**`shared_utils/src/lib.rs`** — shared code used by other packages:

```rust
pub fn log(message: &str) {
    println!("[LOG] {}", message);
}

pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}
```

**`auth_service/Cargo.toml`** — depends on the shared library:

```toml
[package]
name = "auth_service"
version = "0.1.0"
edition = "2021"

[dependencies]
shared_utils = { path = "../shared_utils" }
```

**`auth_service/src/main.rs`** — uses the shared library:

```rust
use shared_utils::{log, validate_email};

fn main() {
    log("Auth service starting...");

    let email = "user@example.com";
    if validate_email(email) {
        log(&format!("Valid email: {}", email));
    } else {
        log(&format!("Invalid email: {}", email));
    }
}
```

**`api_gateway/Cargo.toml`** — also depends on the shared library:

```toml
[package]
name = "api_gateway"
version = "0.1.0"
edition = "2021"

[dependencies]
shared_utils = { path = "../shared_utils" }
```

**`api_gateway/src/main.rs`** — uses the same shared library:

```rust
use shared_utils::log;

fn main() {
    log("API gateway starting on port 3000...");
    log("Routing requests to auth_service...");
}
```

### Running Workspace Packages

```bash
cargo run -p auth_service    # runs auth_service/src/main.rs
cargo run -p api_gateway     # runs api_gateway/src/main.rs
cargo build                  # builds all packages in the workspace
cargo test                   # tests all packages in the workspace
```

### Key Benefits of Workspaces

- **Shared `Cargo.lock`** — all packages use the same dependency versions, avoiding conflicts
- **Shared `target/` directory** — dependencies compile once, not once per package
- **Cross-package dependencies** — packages can depend on each other using `{ path = "..." }`
- **Unified commands** — `cargo build` and `cargo test` run across all packages at once

## Combining Both Approaches

Both approaches can coexist — a workspace member can itself have multiple binaries via `src/bin/`:

```
my_workspace/
├── Cargo.toml                # [workspace] root
├── auth_service/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs            # library crate
│       ├── main.rs           # default binary (the server)
│       └── bin/
│           └── migrate.rs    # additional binary within this package
└── shared_utils/
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

```bash
cargo run -p auth_service                # runs auth_service/src/main.rs
cargo run -p auth_service --bin migrate  # runs auth_service/src/bin/migrate.rs
```

## Summary: How to Choose

There are only two built-in ways to organize Rust projects, and everything else is a combination of them:

| Approach | When to Use |
|---|---|
| **Single package** | All code is closely related and shares the same dependencies |
| **Single package + `src/bin/`** | You need a few extra CLI tools or utilities alongside your main binary |
| **Workspace** | You have distinct components (services, libraries) that should be compiled and versioned independently |
| **Workspace + `src/bin/`** | Distinct components where some members also have multiple binaries |

There are no other alternatives — these cover all standard Rust project structures.

## `cargo install` vs `cargo add`

| Command | What it does | Works with |
|---|---|---|
| `cargo install <crate>` | Installs a CLI tool to `~/.cargo/bin/` | Binary crates only |
| `cargo add <crate>` | Adds a dependency to `Cargo.toml` | Libraries (and binary crates with a lib) |

- **`cargo install`** — only works with crates that have a `main.rs`. For example, `cargo install ripgrep` installs the `rg` command. You **cannot** `cargo install` a library-only crate.
- **`cargo add`** — adds a library as a dependency to your project. For example, `cargo add serde` adds serde to your `Cargo.toml`.

### Where Does the Source Code Go?

When you `cargo add` a library, Cargo downloads its full source code locally to:

```
~/.cargo/registry/src/index.crates.io-*/
```

For example, after `cargo add serde`:

```
~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/serde-1.0.200/src/
```

The compiled output goes to your project's `target/` directory.

You can browse the source code directly in that directory, or in your IDE — **Ctrl+click** (or Cmd+click) on any function or type from a dependency to jump into the library's source code.

## Module Keywords & Path Syntax

| Keyword/Syntax | Meaning | Example |
|---|---|---|
| `mod` | Declare a module | `mod auth;` |
| `use` | Bring items into scope | `use crate::auth::login;` |
| `::` | Path separator | `auth::jwt::create_token()` |
| `crate` | Root of the current crate | `use crate::config::AppConfig;` |
| `self` | Current module | `use self::helpers::format;` |
| `super` | Parent module | `use super::login;` |
| `as` | Rename/alias an import | `use crate::auth as authentication;` |
| `*` | Glob import (all public items) | `use crate::utils::*;` |
| `pub use` | Re-export (make available to outside) | `pub use self::jwt::create_token;` |
| `pub` | Make item public | `pub fn login() {}` |
| `pub(crate)` | Public only within the crate | `pub(crate) fn internal() {}` |
| `pub(super)` | Public only to parent module | `pub(super) fn helper() {}` |
| `{}` | Import multiple items | `use crate::auth::{login, logout};` |

### Examples in Context

Given this structure:

```
src/
├── lib.rs
├── auth.rs
└── auth/
    ├── jwt.rs
    └── password.rs
```

**Using `crate` — absolute path from the crate root:**

```rust
// src/auth/jwt.rs
use crate::auth::password::hash;  // start from the crate root, go to auth::password::hash

pub fn create_token(password: &str) -> String {
    let hashed = hash(password);
    format!("token_{}", hashed)
}
```

**Using `super` — relative path to the parent module:**

```rust
// src/auth/jwt.rs
use super::password::hash;  // super = auth (parent module), then password::hash

pub fn create_token(password: &str) -> String {
    let hashed = hash(password);
    format!("token_{}", hashed)
}
```

**Using `self` — current module:**

```rust
// src/auth.rs
pub mod jwt;
pub mod password;

use self::jwt::create_token;  // self = auth (current module)
use self::password::hash;
```

**Using `as` — rename an import:**

```rust
use crate::auth::jwt::create_token as generate_jwt;
use crate::auth::password::hash as hash_password;

fn main() {
    let token = generate_jwt("secret");
    let hashed = hash_password("password123");
}
```

**Using `{}` — import multiple items at once:**

```rust
use crate::auth::{jwt, password};
use crate::auth::jwt::{create_token, verify_token};
```

**Using `*` — glob import (import everything public):**

```rust
use crate::auth::jwt::*;  // imports create_token, verify_token, and all other pub items
```

**Using `pub use` — re-export items so users don't need to know the internal structure:**

```rust
// src/auth.rs
pub mod jwt;
pub mod password;

// re-export so users can do `auth::create_token` instead of `auth::jwt::create_token`
pub use self::jwt::create_token;
pub use self::password::hash;
```

**Visibility modifiers:**

```rust
pub fn public_to_everyone() {}          // accessible from anywhere
pub(crate) fn public_in_crate() {}      // accessible only within this crate
pub(super) fn public_to_parent() {}     // accessible only to the parent module
fn private() {}                         // accessible only within this module
```
https://matklad.github.io/2021/09/05/Rust100k.html

https://corrode.dev/blog/prototyping/#fr-3-1:~:text=main.rs%20is%20your%20best%20friend%20while%20prototyping.

---

## درسنامه فارسی

### ۱. پکیج (package)

هر پوشه‌ای که فایل `Cargo.toml` داشته باشد یک **package** است. هر package حداقل یک crate دارد.

### ۲. کریت (crate)

کریت واحدی است که کامپایل می‌شود. طبق قرارداد پیش‌فرض Cargo، این فایل‌ها **ریشه‌ی یک crate** هستند:

| فایل | نوع crate |
|---|---|
| فایل `src/lib.rs` | کتابخانه (library)، حداکثر یکی در هر package |
| فایل `src/main.rs` | اجرایی (binary) |
| هر فایل در `src/bin/` | اجرایی، هر فایل یک crate جدا |

یک crate فقط همان فایل ریشه نیست. **فایل ریشه و همه‌ی moduleهایی که به آن وصل شده‌اند** روی هم یک crate هستند.

وقتی package فایل `lib.rs` دارد، یعنی **یک library crate دارد**. اگر `main.rs` یا `src/bin/` هم داشته باشد، binary crate هم دارد. مثلاً پروژه‌ی `auth_service` در این داکیومنت چهار crate دارد:

```
src/lib.rs               ← library crate
src/main.rs              ← binary crate
src/bin/migrate.rs       ← binary crate
src/bin/generate_key.rs  ← binary crate
```

**چرا library crate می‌سازیم؟**

کدی که فقط در `main.rs` باشد به یک برنامه‌ی اجرایی تبدیل می‌شود. دیگران می‌توانند آن را **اجرا کنند** (مثلاً با `cargo install`)، ولی نمی‌توانند در کد خودشان از توابع و structهای آن استفاده کنند. برای این کار library crate لازم است.

دقت کن که کتابخانه **به‌صورت سورس کد** منتشر می‌شود، نه به‌صورت باینری. وقتی کسی `cargo add serde` می‌زند، Cargo سورس را دانلود می‌کند و روی سیستم خود او کامپایل می‌کند (در `~/.cargo/registry/src/`). دلیلش این است که Rust یک ABI پایدار ندارد و خروجی کامپایل‌شده‌ی یک نسخه از کامپایلر لزوماً با نسخه‌ی دیگر سازگار نیست.

حتی اگر هیچ‌وقت کد را به کسی ندهی، کتابخانه به کار می‌آید. «استفاده‌کننده» فقط آدم‌های بیرون نیستند، **crateهای دیگر خود پروژه‌ات** هم هستند:

۱. **چند باینری در یک package:** هر کدام از `main.rs` و `src/bin/migrate.rs` یک crate جداست. تنها راه تمیز برای کد مشترک بین آن‌ها `lib.rs` است. پروژه‌ی `auth_service` در همین داکیومنت همین کار را می‌کند.

۲. **تست‌های یکپارچه (integration tests):** فایل‌های پوشه‌ی `tests/` فقط به library crate دسترسی دارند. چیزی که فقط در `main.rs` باشد را نمی‌شود از آنجا تست کرد.

۳. **تست‌های داخل مستندات (doc tests):** مثال‌هایی که در کامنت‌های `///` می‌نویسی و `cargo test` اجرایشان می‌کند، فقط برای library crate اجرا می‌شوند.

۴. **ورک‌اسپیس (workspace):** وقتی چند package داری (مثل `shared_utils` در بخش Cargo Workspaces)، فقط از راه library crate می‌توانند از کد هم استفاده کنند.

۵. **جدا کردن منطق از ورودی/خروجی:** منطق اصلی در `lib.rs` می‌ماند و `main.rs` فقط ورودی را می‌خواند و کتابخانه را صدا می‌زند. کد مرتب‌تر و تست‌پذیرتر می‌شود. کتاب رسمی Rust هم همین الگو را پیشنهاد می‌کند:

```rust
// src/main.rs  (thin wrapper: no logic here, only calls into the library)
use auth_service::auth::AuthService;
use auth_service::config::AppConfig;
use auth_service::db::DbPool;

fn main() {
    let config = AppConfig::from_env();
    let _pool = DbPool::connect(&config);
    let auth = AuthService::new(&config);

    if let Some(token) = auth.login("user@example.com", "my_password") {
        println!("Login token: {}", token);
    }
}
```

همه‌ی منطق (ساختن تنظیمات، اتصال به دیتابیس، هش کردن رمز و ساختن توکن) در کتابخانه است. پس می‌شود آن را از `tests/` تست کرد و `src/bin/migrate.rs` هم می‌تواند از همان `AppConfig` و `DbPool` استفاده کند.

خلاصه: library crate را می‌سازیم تا کد **از crateهای دیگر** قابل استفاده باشد. آن crate دیگر می‌تواند پروژه‌ی یک نفر دیگر باشد، یک باینری دیگر در همین package، یا تست‌های پوشه‌ی `tests/`.

### ۳. ماژول (module)

هر فایل `.rs` دیگری در `src` (به‌جز `src/bin/`) یک **module** است، مثل `config.rs`، `db.rs`، `auth.rs` یا `auth/jwt.rs`.

برای اینکه یک module کامپایل شود و جزو crate باشد، **والدش** باید آن را با `mod` وصل کند:

```rust
// src/lib.rs   ← والد
pub mod auth;
```

```rust
// src/auth.rs  ← خود module، اینجا هیچ mod auth نوشته نمی‌شود
pub struct AuthService { /* ... */ }
```

نکته‌ها:

- کلمه‌ی `mod` را والد می‌نویسد، نه خود فایل module.
- والد همیشه `lib.rs` یا `main.rs` نیست. یک module هم می‌تواند زیرmodule داشته باشد، مثلاً `auth.rs` با `pub mod jwt;` فایل `auth/jwt.rs` را وصل می‌کند.
- فایلی که به هیچ والدی وصل نشده اصلاً کامپایل نمی‌شود.
- عبارت `mod auth;` بدون `pub` هم کافی است تا module **داخل همان crate** قابل استفاده باشد. کلمه‌ی `pub` فقط برای دسترسی از **بیرون** crate لازم است.

**فایل را به کدام والد وصل کنم، `lib.rs` یا `main.rs`؟**

هر دو درست است و کامپایل می‌شود. ولی **جایی که `mod` را می‌نویسی تعیین می‌کند فایل جزو کدام crate باشد**، و همین، مسیر استفاده از آن را هم عوض می‌کند:

| جای نوشتن `mod auth;` | فایل جزو کدام crate است | مسیر استفاده در `main.rs` |
|---|---|---|
| فقط در `main.rs` | باینری | مسیر `crate::auth::login` |
| فقط در `lib.rs` | کتابخانه | مسیر `auth_service::auth::login` |
| در هر دو | ❌ دو نسخه‌ی جدا از یک فایل ساخته می‌شود ([بخش ۶](#۶-دام-وصل-کردن-یک-فایل-به-دو-crate)) | — |

پس سؤال این نیست که کدام درست است. سؤال این است که **چه کسی از این module استفاده می‌کند**:

- اگر فقط `main.rs` لازمش دارد (مثلاً `cli.rs`)، در `main.rs` وصلش کن.
- اگر کتابخانه یا باینری‌های دیگر هم لازمش دارند، در `lib.rs` وصلش کن. مثلاً `src/bin/migrate.rs` یک crate جداست و فقط از راه کتابخانه به module می‌رسد، نه از راه `main.rs`.
- هیچ‌وقت یک فایل را در هر دو وصل نکن.

### ۴. پنج روش ساختن module

**روش ۱: درون‌خطی (inline)**، یعنی module را داخل همان فایل با آکولاد تعریف می‌کنی:

```rust
// src/main.rs
mod auth {
    pub fn login() { println!("Logging in..."); }
}

fn main() {
    auth::login();   // ✅ با اینکه mod auth خصوصی است
}
```

ماژول خصوصی فقط «داخل خودش» قابل دسترسی نیست. **والدش** (فایلی که `mod` را نوشته) و moduleهای دیگر داخل همان والد هم به آن دسترسی دارند. ولی چیزهای داخلش (مثل `login`) باید `pub` باشند.

**روش ۲: یک فایل جدا.** ساده‌ترین روش است:

```
src/
├── main.rs      ← mod auth;
└── auth.rs      ← pub fn login() {...}
```

**روش ۳: پوشه به‌اضافه‌ی `mod.rs` (روش قدیمی، قبل از ۲۰۱۸).** فایل `auth/mod.rs` دقیقاً همان نقش `auth.rs` را دارد، فقط داخل پوشه است:

```
src/
├── main.rs        ← mod auth;
└── auth/
    ├── mod.rs     ← pub mod jwt;
    └── jwt.rs
```

ایرادش این است که در پروژه‌ی بزرگ ده‌ها فایل با اسم `mod.rs` داری و در editor نمی‌شود آن‌ها را از هم تشخیص داد.

**روش ۴: فایل و پوشه‌ی هم‌نام (روش جدید و پیشنهادی).** فایل `auth.rs` والد است و زیرmoduleها در پوشه‌ی `auth/` قرار می‌گیرند:

```
src/
├── main.rs        ← mod auth;
├── auth.rs        ← pub mod jwt;
└── auth/
    └── jwt.rs
```

روش ۳ و ۴ نتیجه‌ی یکسانی دارند. فقط جای فایل والد فرق دارد:

| روش | فایل والد module به نام `auth` |
|---|---|
| روش قدیمی | فایل `src/auth/mod.rs` |
| روش جدید | فایل `src/auth.rs` |

**روش ۵: ویژگی `#[path]`.** هر فایلی را با هر اسمی به یک module وصل می‌کند، حتی بیرون از مسیر معمول:

```rust
#[path = "my_custom_folder/authentication.rs"]
mod auth;
```

این روش کار می‌کند ولی توصیه نمی‌شود، چون دیگر از روی اسم فایل نمی‌شود فهمید module کجاست.

**ترتیب جست‌وجو وجود ندارد.** کامپایلر این روش‌ها را یکی پس از دیگری امتحان نمی‌کند. **شکل نوشتن تو** تعیین می‌کند کدام روش است:

| چیزی که می‌نویسی | کامپایلر چه می‌کند |
|---|---|
| عبارت `mod auth { ... }` | کد را از همین آکولاد برمی‌دارد و دنبال فایل نمی‌گردد |
| عبارت `mod auth;` | هم‌زمان دنبال `auth.rs` و `auth/mod.rs` می‌گردد؛ دقیقاً یکی باید باشد |
| عبارت `#[path = "..."] mod auth;` | فقط همان مسیر داده‌شده را باز می‌کند |

اگر هم `auth.rs` و هم `auth/mod.rs` وجود داشته باشند، هیچ‌کدام برنده نمی‌شود و خطا می‌گیری:

```
error[E0761]: file for module `auth` found at both "auth.rs" and "auth/mod.rs"
```

اگر هم inline و هم فایل را با یک اسم بنویسی، باز خطا می‌گیری، چون اسم دو بار تعریف شده:

```rust
mod auth { pub fn login() {} }
mod auth;   // ❌ error[E0428]: the name `auth` is defined multiple times
```

### ۵. شروع مسیر: `crate::` یا نام package؟

معنی `crate::` به این بستگی دارد که **کد را در کدام crate می‌نویسی**:

- عبارت `crate::` یعنی «ریشه‌ی **همین** crate» (خودم).
- نام package (مثلاً `auth_service::`) یعنی «library crate این پروژه» (آن یکی).

| کد را کجا می‌نویسی | عبارت `crate::` اشاره می‌کند به |
|---|---|
| در `lib.rs` یا moduleهایش | فایل `lib.rs` |
| در `main.rs` یا moduleهایی که خودش وصل کرده | فایل `main.rs` |
| در `src/bin/migrate.rs` | فایل `src/bin/migrate.rs` |

در این مثال، `config` و `auth` هر دو در `lib.rs` وصل شده‌اند. پس داخل `auth.rs` می‌نویسیم:

```rust
// src/auth.rs  (جزو library)
use crate::config::AppConfig;
```

ولی `main.rs` یک crate جداست. برای رسیدن به library باید از نام package شروع کند:

```rust
// src/main.rs  (binary)
use auth_service::config::AppConfig;   // ✅
use crate::config::AppConfig;          // ❌ چون config در main.rs وصل نشده
```

اگر نام package خط تیره داشته باشد (مثلاً `auth-service`)، در کد به `_` تبدیل می‌شود: `auth_service::`.

### ۶. دام: وصل کردن یک فایل به دو crate

اگر در `main.rs` هم `mod config;` بنویسی، عبارت `crate::config::AppConfig` در `main.rs` کار می‌کند. ولی چون `lib.rs` هم همین فایل را وصل کرده، فایل **دو بار** کامپایل می‌شود و **دو نوع جدا** با اسم یکسان ساخته می‌شود:

```rust
// src/main.rs
mod config;
use auth_service::db::DbPool;

fn main() {
    let c = crate::config::AppConfig::from_env();
    let pool = DbPool::connect(&c);   // ❌ mismatched types
}
```

قانون عملی:

- ماژولی که فقط مال `main.rs` است (مثلاً `cli.rs`) را در `main.rs` با `mod cli;` وصل کن و با `crate::cli` از آن استفاده کن.
- ماژولی که در `lib.rs` وصل شده را در `main.rs` دوباره وصل نکن و با `auth_service::...` از آن استفاده کن.
- هر فایل فقط باید به **یک** crate وصل باشد.

### ۷. سؤال: آیا `pub mod auth;` کافی است تا در `main.rs` از آن استفاده کنم؟

نه. دو کار مستقل اینجا هست:

- کلمه‌ی `mod` باعث می‌شود فایل **کامپایل شود** و جزو library باشد.
- کلمه‌ی `pub` باعث می‌شود از **بیرون** دیده شود.

عبارت `pub mod auth;` فقط «در» module را باز می‌کند. **در Rust همه‌چیز به‌طور پیش‌فرض خصوصی است.** پس هر struct، تابع، فیلد و enumی که می‌خواهی از `main.rs` به آن برسی هم باید جداگانه `pub` باشد:

```rust
// src/lib.rs
pub mod auth;              // ✅ قدم ۱: module عمومی است
```

```rust
// src/auth.rs
pub struct AuthService {   // ✅ قدم ۲: struct عمومی است
    jwt_secret: String,    // ⚠️ فیلد خصوصی است
}

impl AuthService {
    pub fn new() -> Self { /* ... */ }   // ✅ قدم ۳: تابع عمومی است
    fn helper() { /* ... */ }            // ❌ خصوصی است
}
```

```rust
// src/main.rs
use auth_service::auth::AuthService;   // ✅

fn main() {
    let a = AuthService::new();        // ✅ چون new عمومی است
    AuthService::helper();             // ❌ خطا: private function
    a.jwt_secret;                      // ❌ خطا: private field
}
```

### ۸. صادر کردن دوباره (re-export) با `pub use`

عبارت `pub use` یک چیز را از جای دیگری به module فعلی می‌آورد و **از اینجا هم عمومی می‌کند**. یعنی آن چیز یک آدرس دوم و کوتاه‌تر پیدا می‌کند.

- عبارت `use` اسم را فقط برای **همین فایل** در دسترس می‌کند (خصوصی).
- عبارت `pub use` اسم را برای **بیرونی‌ها** هم از این آدرس در دسترس می‌کند.

مثال از پوشه‌ی [auth_service](auth_service/src/lib.rs):

```
src/lib.rs
 └── mod auth_utils;          ← خصوصی
      └── pub mod models;
           └── pub struct Credentials
```

چون `auth_utils` خصوصی است، `main.rs` نمی‌تواند از مسیر کامل به `Credentials` برسد:

```rust
use auth_service::auth_utils::models::Credentials;   // ❌ خطا: module خصوصی است
```

برای همین در `lib.rs` نوشته شده:

```rust
// src/lib.rs
mod auth_utils;                              // ← لازم است تا فایل کامپایل شود
pub use auth_utils::models::Credentials;     // ← فقط Credentials را بیرون می‌دهد
```

و حالا `main.rs` با آدرس کوتاه به آن می‌رسد:

```rust
// src/main.rs
use auth_service::Credentials;   // ✅
```

دقت کن که `pub use` **جای `mod` را نمی‌گیرد**، بلکه جای `pub` روی `mod` را می‌گیرد. خط `mod auth_utils;` همچنان لازم است.

| روش در `lib.rs` | چه چیزی از بیرون دیده می‌شود؟ |
|---|---|
| فقط `mod auth_utils;` | هیچ‌چیز |
| عبارت `pub mod auth_utils;` | همه‌ی چیزهای `pub` داخلش، با آدرس کامل |
| عبارت `mod auth_utils;` به‌اضافه‌ی `pub use ...` | فقط چیزهایی که دوباره صادر شده‌اند، با آدرس کوتاه |
| عبارت `pub mod auth_utils;` به‌اضافه‌ی `pub use ...` | همه‌چیز با آدرس کامل، و چیزهای صادرشده با آدرس کوتاه هم |

چرا مفید است؟

- **پنهان کردن ساختار داخلی:** کاربر library لازم نیست بداند `Credentials` در کدام فایل است.
- **آزادی در تغییر:** اگر بعداً `models.rs` را جابه‌جا کنی، فقط خط `pub use` را عوض می‌کنی و کد کاربران دست نمی‌خورد.
- **آدرس کوتاه‌تر:** به‌جای `auth_service::auth_utils::models::Credentials` فقط `auth_service::Credentials` می‌نویسی.

خیلی از کتابخانه‌های معروف همین کار را می‌کنند. مثلاً `serde::Serialize` در یک module داخلی تعریف شده و از ریشه دوباره صادر شده است.

### ۹. خلاصه

| مفهوم | تعریف |
|---|---|
| پکیج (package) | پوشه‌ای که `Cargo.toml` دارد |
| کریت (crate) | فایل ریشه (`lib.rs`، `main.rs` یا فایلی در `src/bin/`) به‌اضافه‌ی moduleهایش |
| ماژول (module) | فایل `.rs` دیگری که والدش آن را با `mod` وصل کرده |
| عبارت `mod name;` | فایل را جزو crate می‌کند (والد می‌نویسد) |
| کلمه‌ی `pub` | دسترسی از بیرون را باز می‌کند و باید در هر مرحله‌ی مسیر باشد |
| مسیر `crate::` | ریشه‌ی crate فعلی |
| مسیر `auth_service::` | کتابخانه‌ی پروژه، از دید crateهای دیگر |
| عبارت `pub use` | یک چیز را با آدرس کوتاه‌تر دوباره عمومی می‌کند (re-export) |
