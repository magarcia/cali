# Naming Conventions

This reference covers Rust naming conventions based on [RFC 430](https://github.com/rust-lang/rfcs/blob/master/text/0430-finalizing-naming-conventions.md) and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/naming.html).

## Casing Conventions (RFC 430)

### General Rules

| Item | Convention | Example |
|------|------------|---------|
| Crates | `snake_case` | `my_crate`, `serde_json` |
| Modules | `snake_case` | `my_module`, `error_handling` |
| Types (structs, enums, unions) | `UpperCamelCase` | `MyStruct`, `HttpRequest` |
| Traits | `UpperCamelCase` | `Iterator`, `Display`, `MyTrait` |
| Enum variants | `UpperCamelCase` | `Some`, `None`, `IoError` |
| Functions | `snake_case` | `my_function`, `read_to_string` |
| Methods | `snake_case` | `push_back`, `into_inner` |
| Local variables | `snake_case` | `my_variable`, `user_count` |
| Static variables | `SCREAMING_SNAKE_CASE` | `MAX_SIZE`, `DEFAULT_PORT` |
| Constants | `SCREAMING_SNAKE_CASE` | `PI`, `HTTP_OK` |
| Type parameters | `UpperCamelCase`, single letter preferred | `T`, `E`, `K`, `V` |
| Lifetimes | lowercase, single letter preferred | `'a`, `'b`, `'static` |

### Crate Naming

```rust
// GOOD: Descriptive, snake_case
serde_json
tokio_util
my_awesome_lib

// BAD: CamelCase or mixed
SerdeJson
tokioUtil
```

### Acronyms and Initialisms

Treat acronyms as single words in `UpperCamelCase`:

```rust
// GOOD
struct HttpRequest { ... }
struct JsonParser { ... }
struct Uuid { ... }
type IoResult<T> = Result<T, IoError>;

// BAD
struct HTTPRequest { ... }  // All caps for acronym
struct JSONParser { ... }
struct UUID { ... }
```

## Conversion Methods

Rust has strong conventions for method names based on their behavior:

### `as_` Prefix - Borrowed to Borrowed (Free)

Returns a view of the data without allocation. Cost: O(1).

```rust
impl String {
    fn as_str(&self) -> &str { ... }
    fn as_bytes(&self) -> &[u8] { ... }
}

impl Path {
    fn as_os_str(&self) -> &OsStr { ... }
}
```

### `to_` Prefix - Borrowed to Owned (Expensive)

Creates a new owned value, typically involving allocation. Cost: O(n).

```rust
impl str {
    fn to_string(&self) -> String { ... }
    fn to_lowercase(&self) -> String { ... }
    fn to_ascii_uppercase(&self) -> String { ... }
}

impl [T] {
    fn to_vec(&self) -> Vec<T> where T: Clone { ... }
}
```

### `into_` Prefix - Owned to Owned (Variable)

Consumes self, transforms into another type. May or may not allocate.

```rust
impl String {
    fn into_bytes(self) -> Vec<u8> { ... }
    fn into_boxed_str(self) -> Box<str> { ... }
}

impl Vec<T> {
    fn into_iter(self) -> IntoIter<T> { ... }
    fn into_boxed_slice(self) -> Box<[T]> { ... }
}
```

### Summary Table

| Prefix | Cost | Ownership Change | Example |
|--------|------|------------------|---------|
| `as_` | Free | `&self → &T` | `as_str()`, `as_bytes()` |
| `to_` | Expensive | `&self → T` | `to_string()`, `to_vec()` |
| `into_` | Variable | `self → T` | `into_inner()`, `into_bytes()` |

## Getter and Setter Methods

### Getter Naming

**Omit the `get_` prefix** for simple getters:

```rust
// GOOD
impl Config {
    fn name(&self) -> &str { &self.name }
    fn port(&self) -> u16 { self.port }
    fn is_enabled(&self) -> bool { self.enabled }
}

// BAD
impl Config {
    fn get_name(&self) -> &str { &self.name }  // Unnecessary prefix
    fn get_port(&self) -> u16 { self.port }
}
```

### Mutable Getters

Use `_mut` suffix for mutable access:

```rust
impl Container {
    fn items(&self) -> &[Item] { &self.items }
    fn items_mut(&mut self) -> &mut [Item] { &mut self.items }
}
```

### Setter Naming

Use `set_` prefix for setters:

```rust
impl Config {
    fn set_port(&mut self, port: u16) {
        self.port = port;
    }
}
```

### Consuming Getters

Use `into_` prefix when consuming self:

```rust
impl Wrapper<T> {
    fn inner(&self) -> &T { &self.inner }
    fn inner_mut(&mut self) -> &mut T { &mut self.inner }
    fn into_inner(self) -> T { self.inner }
}
```

## Iterator Methods

Standard convention for types that can be iterated:

```rust
impl MyCollection<T> {
    /// Iterates over borrowed items
    fn iter(&self) -> impl Iterator<Item = &T> { ... }

    /// Iterates over mutably borrowed items
    fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> { ... }
}

/// Implement IntoIterator for consuming iteration
impl<T> IntoIterator for MyCollection<T> {
    type Item = T;
    type IntoIter = MyCollectionIter<T>;

    fn into_iter(self) -> Self::IntoIter { ... }
}
```

## Boolean Methods

Use `is_` or `has_` prefix for methods returning `bool`:

```rust
impl Path {
    fn is_absolute(&self) -> bool { ... }
    fn is_dir(&self) -> bool { ... }
    fn has_root(&self) -> bool { ... }
}

impl str {
    fn is_empty(&self) -> bool { ... }
    fn is_ascii(&self) -> bool { ... }
}
```

## Type Parameter Naming

### Common Conventions

| Parameter | Typical Meaning |
|-----------|-----------------|
| `T` | Generic type |
| `E` | Error type |
| `K` | Key type (maps) |
| `V` | Value type (maps) |
| `S` | State type |
| `R` | Reader type |
| `W` | Writer type |
| `I` | Iterator type |
| `F` | Function/closure type |

### Descriptive Names for Clarity

When single letters are ambiguous, use descriptive names:

```rust
// Single letters when obvious
fn swap<T>(a: &mut T, b: &mut T) { ... }
fn map<T, U, F: Fn(T) -> U>(opt: Option<T>, f: F) -> Option<U> { ... }

// Descriptive when needed
fn connect<Addr: ToSocketAddrs>(addr: Addr) -> Result<Connection> { ... }
fn serialize<Output: Write>(value: &Value, output: Output) -> Result<()> { ... }
```

## Lifetime Naming

### Common Conventions

| Lifetime | Typical Meaning |
|----------|-----------------|
| `'a` | Generic lifetime |
| `'b`, `'c` | Additional generic lifetimes |
| `'de` | Deserialization lifetime (serde) |
| `'src` | Source code/text lifetime |
| `'input` | Input data lifetime |
| `'static` | Static lifetime |

### When to Use Descriptive Names

```rust
// Single letter for simple cases
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { ... }

// Descriptive for complex cases
struct Parser<'src, 'arena> {
    source: &'src str,
    allocator: &'arena Arena,
}
```

## Feature Flag Naming

Feature names should be `snake_case` and descriptive:

```toml
[features]
default = ["std"]
std = []
alloc = []
serde = ["dep:serde"]
async = ["tokio"]
full = ["std", "serde", "async"]
```

Avoid:
- Generic names like `extra` or `advanced`
- Version numbers in feature names
- Negative features (use `no-std` pattern sparingly)

## Module Organization

### File Naming

```
src/
├── lib.rs           # Crate root
├── config.rs        # Module file
├── config/          # Module directory (alternative)
│   ├── mod.rs       # Module root
│   ├── parser.rs    # Submodule
│   └── schema.rs    # Submodule
├── error.rs         # Error types
└── utils.rs         # Utility functions
```

### Re-exports

Use `pub use` to create a clean public API:

```rust
// lib.rs
mod config;
mod error;
mod parser;

pub use config::Config;
pub use error::{Error, Result};
pub use parser::Parser;
```
