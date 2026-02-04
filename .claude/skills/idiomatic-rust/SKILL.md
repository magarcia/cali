---
name: idiomatic-rust
description: Write idiomatic, high-quality Rust code following community best practices. Use when writing, reviewing, or refactoring Rust code (.rs files, Cargo.toml), converting code to Rust, designing error types with Result/Option, implementing traits, or when discussing ownership, borrowing, lifetimes, or Rust API design patterns.
---

# Idiomatic Rust

This skill provides guidance for writing idiomatic Rust code that follows community conventions, leverages the type system effectively, and produces maintainable, performant software.

## Core Principles

### 1. Leverage the Type System

Rust's type system is your greatest ally. Use it to:

- **Make invalid states unrepresentable** - Design types so illegal combinations cannot compile
- **Encode invariants at compile time** - Use newtypes, enums, and const generics
- **Prefer `From`/`TryFrom` over `as` casts** - Explicit, safe conversions

```rust
// BAD: Primitive obsession
fn process_user(user_id: u64, account_id: u64) { ... }

// GOOD: Newtype pattern prevents mixing up IDs
struct UserId(u64);
struct AccountId(u64);
fn process_user(user_id: UserId, account_id: AccountId) { ... }
```

### 2. Embrace Ownership and Borrowing

- **Accept borrowed data when possible** - Use `&str` over `String`, `&[T]` over `Vec<T>` in function parameters
- **Return owned data when creating** - Let callers decide how to store results
- **Use `Cow<'_, T>` for flexibility** - When you might or might not need to clone
- **Prefer `&self` methods** - Only use `&mut self` or `self` when necessary

```rust
// GOOD: Accept borrowed, return owned
fn normalize_path(path: &str) -> PathBuf { ... }

// GOOD: Cow for conditional allocation
fn escape_html(input: &str) -> Cow<'_, str> {
    if needs_escaping(input) {
        Cow::Owned(do_escape(input))
    } else {
        Cow::Borrowed(input)
    }
}
```

### 3. Handle Errors Properly

- **Use `Result` for recoverable errors, `panic!` for bugs**
- **Create meaningful error types** - Not just strings
- **Don't expose internal error types** - Abstract over dependencies
- **Use `?` liberally** - Propagate errors cleanly

```rust
// BAD: Leaking internal errors
pub fn read_config(path: &Path) -> Result<Config, std::io::Error> { ... }

// GOOD: Custom error type
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file: {path}")]
    Read { path: PathBuf, source: std::io::Error },
    #[error("invalid config format")]
    Parse(#[from] toml::de::Error),
}
```

### 4. Write Expressive Code

- **Use iterators over explicit loops** - More declarative, often faster
- **Leverage pattern matching** - Exhaustive, self-documenting
- **Prefer standard library types** - `Option`, `Result`, `Vec`, `HashMap`
- **Use `#[must_use]` for important return values**

```rust
// BAD: Imperative loop
let mut results = Vec::new();
for item in items {
    if item.is_valid() {
        results.push(item.process());
    }
}

// GOOD: Iterator chain
let results: Vec<_> = items
    .iter()
    .filter(|item| item.is_valid())
    .map(|item| item.process())
    .collect();
```

### 5. Document with Purpose

- **Document the "why", not the "what"** - Code shows what, comments explain why
- **Include examples in doc comments** - They're tested by `cargo test`
- **Document panics, errors, and safety** - Required sections for public APIs
- **Use `#[doc(hidden)]` for internal public items**

````rust
/// Parses a configuration file from the given path.
///
/// # Errors
///
/// Returns an error if the file cannot be read or contains invalid TOML.
///
/// # Examples
///
/// ```
/// let config = parse_config("config.toml")?;
/// assert_eq!(config.name, "my-app");
/// ```
pub fn parse_config(path: &str) -> Result<Config, ConfigError> { ... }
````

## Quick Reference

### Naming Conventions (RFC 430)

| Item             | Convention              | Example             |
| ---------------- | ----------------------- | ------------------- |
| Crates           | `snake_case`            | `my_crate`          |
| Modules          | `snake_case`            | `my_module`         |
| Types            | `UpperCamelCase`        | `MyStruct`          |
| Traits           | `UpperCamelCase`        | `MyTrait`           |
| Enum variants    | `UpperCamelCase`        | `MyVariant`         |
| Functions        | `snake_case`            | `my_function`       |
| Methods          | `snake_case`            | `my_method`         |
| Local variables  | `snake_case`            | `my_variable`       |
| Static variables | `SCREAMING_SNAKE_CASE`  | `MY_STATIC`         |
| Constants        | `SCREAMING_SNAKE_CASE`  | `MY_CONST`          |
| Type parameters  | `UpperCamelCase`, short | `T`, `E`, `K`, `V`  |
| Lifetimes        | `lowercase`, short      | `'a`, `'de`, `'src` |

### Conversion Method Prefixes

| Prefix  | Cost      | Ownership           | Example                         |
| ------- | --------- | ------------------- | ------------------------------- |
| `as_`   | Free      | Borrowed → Borrowed | `fn as_str(&self) -> &str`      |
| `to_`   | Expensive | Borrowed → Owned    | `fn to_string(&self) -> String` |
| `into_` | Variable  | Owned → Owned       | `fn into_inner(self) -> T`      |

### Getter/Setter Conventions

```rust
// GOOD: No get_ prefix for getters
impl Foo {
    fn bar(&self) -> &Bar { &self.bar }
    fn bar_mut(&mut self) -> &mut Bar { &mut self.bar }
    fn set_bar(&mut self, bar: Bar) { self.bar = bar; }
    fn into_bar(self) -> Bar { self.bar }
}
```

### Iterator Methods

```rust
impl MyCollection<T> {
    fn iter(&self) -> impl Iterator<Item = &T>          // Borrowed iteration
    fn iter_mut(&mut self) -> impl Iterator<Item = &mut T>  // Mutable iteration
    fn into_iter(self) -> impl Iterator<Item = T>       // Consuming iteration
}
```

### Error Design Quick Guide

| Context                | Approach                             |
| ---------------------- | ------------------------------------ |
| Application binary     | `anyhow::Error` for convenience      |
| Library crate          | Custom error enum with `thiserror`   |
| Multiple error sources | Enum variants with `#[from]`         |
| Performance critical   | Boxed trait object: `Box<dyn Error>` |
| FFI boundary           | Error codes with `repr(C)` enum      |

### Common Patterns

**Builder Pattern:**

```rust
let config = ConfigBuilder::new()
    .name("my-app")
    .timeout(Duration::from_secs(30))
    .build()?;
```

**Newtype Pattern:**

```rust
struct Email(String);
impl Email {
    pub fn new(s: impl Into<String>) -> Result<Self, InvalidEmail> { ... }
}
```

**Type State Pattern:**

```rust
struct Connection<State> { ... }
struct Disconnected;
struct Connected;

impl Connection<Disconnected> {
    fn connect(self) -> Result<Connection<Connected>, Error> { ... }
}
impl Connection<Connected> {
    fn send(&self, data: &[u8]) -> Result<(), Error> { ... }
}
```

## Common Pitfalls to Avoid

1. **Integer overflow in release mode** - Use `checked_*`, `saturating_*`, or `wrapping_*`
2. **Unsafe `as` casts** - Use `From`/`TryFrom` for safe conversions
3. **Array indexing without bounds checking** - Use `.get()` or iterators
4. **Sensitive data in `Debug` output** - Implement `Debug` manually to redact
5. **TOCTOU races** - Check and act atomically when possible
6. **Ignoring `#[must_use]` warnings** - Results must be handled
7. **Blocking in async contexts** - Use `spawn_blocking` for CPU-bound work

## References

For detailed guidance on specific topics, see the reference files:

- [Naming Conventions](references/naming.md) - RFC 430, API guidelines, method prefixes
- [Error Handling](references/error-handling.md) - Error types, Result patterns, thiserror/anyhow
- [Type Safety](references/types.md) - Newtypes, builders, making invalid states unrepresentable
- [Documentation](references/documentation.md) - Doc comments, examples, rustdoc best practices
- [Common Patterns](references/patterns.md) - Iterators, pattern matching, module organization
- [Pitfalls](references/pitfalls.md) - Safe Rust gotchas and how to avoid them
- [Resources](references/resources.md) - Curated links to articles, books, and talks

## Checklist for Code Review

When reviewing Rust code, verify:

- [ ] Types encode domain constraints (no primitive obsession)
- [ ] Error types are meaningful and don't leak internals
- [ ] Public APIs have documentation with examples
- [ ] `#[must_use]` on functions returning important values
- [ ] No `unwrap()` or `expect()` in library code (except tests)
- [ ] Iterators preferred over explicit loops where appropriate
- [ ] Naming follows RFC 430 conventions
- [ ] No `unsafe` without clear justification and safety comments
- [ ] Clippy passes without warnings
- [ ] Tests cover edge cases and error conditions
