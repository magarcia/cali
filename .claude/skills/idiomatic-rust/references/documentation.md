# Documentation

This reference covers documentation best practices for Rust code, including doc comments, examples, and rustdoc conventions.

## Doc Comment Basics

Use `///` for item documentation and `//!` for module/crate documentation:

```rust
//! # My Crate
//!
//! This crate provides utilities for working with widgets.
//!
//! ## Quick Start
//!
//! ```
//! use my_crate::Widget;
//!
//! let widget = Widget::new("example");
//! widget.process()?;
//! ```

/// A widget that can be processed.
///
/// Widgets represent discrete units of work that can be
/// scheduled and executed independently.
pub struct Widget {
    name: String,
}
```

## Required Documentation Sections

### For Functions and Methods

```rust
/// Parses a configuration file from the given path.
///
/// This function reads the file contents and deserializes
/// them into a `Config` struct using TOML format.
///
/// # Arguments
///
/// * `path` - Path to the configuration file
///
/// # Returns
///
/// The parsed configuration on success.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The file contains invalid TOML
/// - Required fields are missing
///
/// # Panics
///
/// Panics if the path contains invalid UTF-8 (use `parse_config_os`
/// for arbitrary paths).
///
/// # Examples
///
/// ```
/// use my_crate::parse_config;
///
/// let config = parse_config("config.toml")?;
/// assert_eq!(config.name, "my-app");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse_config(path: &str) -> Result<Config, ConfigError> {
    // ...
}
```

### For Unsafe Functions

```rust
/// Dereferences a raw pointer.
///
/// # Safety
///
/// The caller must ensure:
/// - `ptr` is valid and properly aligned
/// - `ptr` points to a properly initialized `T`
/// - The memory is not mutated while this reference exists
/// - The pointed-to value lives at least as long as `'a`
///
/// # Examples
///
/// ```
/// fn example<'a>(ptr: *const i32, _lifetime_witness: &'a i32) -> &'a i32 {
///     // SAFETY: caller guarantees ptr is valid for 'a
///     unsafe { deref_ptr(ptr) }
/// }
///
/// let value = 42;
/// let ptr = &value as *const i32;
/// let reference = example(ptr, &value);
/// assert_eq!(*reference, 42);
/// ```
pub unsafe fn deref_ptr<'a, T>(ptr: *const T) -> &'a T {
    &*ptr
}
```

## Writing Good Examples

### Make Examples Testable

Doc examples are run as tests with `cargo test`:

```rust
/// Adds two numbers together.
///
/// # Examples
///
/// ```
/// use my_crate::add;
///
/// let result = add(2, 3);
/// assert_eq!(result, 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Handling Results in Examples

```rust
/// Reads a file to string.
///
/// # Examples
///
/// ```
/// use my_crate::read_file;
///
/// let content = read_file("example.txt")?;
/// println!("Content: {}", content);
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn read_file(path: &str) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}
```

The `# Ok::<(), ...>(())` line is hidden (starts with `#`) but makes the example compile.

### Using `should_panic`

```rust
/// Divides two numbers.
///
/// # Panics
///
/// Panics if `divisor` is zero.
///
/// # Examples
///
/// ```should_panic
/// use my_crate::divide;
///
/// divide(10, 0);  // This panics
/// ```
pub fn divide(dividend: i32, divisor: i32) -> i32 {
    if divisor == 0 {
        panic!("division by zero");
    }
    dividend / divisor
}
```

### Using `no_run`

For examples that shouldn't be executed during tests:

```rust
/// Connects to a database.
///
/// # Examples
///
/// ```no_run
/// use my_crate::Database;
///
/// let db = Database::connect("postgres://localhost/mydb")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn connect(url: &str) -> Result<Database, DbError> {
    // ...
}
```

### Using `ignore`

For incomplete or platform-specific examples:

```rust
/// Platform-specific function.
///
/// # Examples
///
/// ```ignore
/// // Only works on Windows
/// use my_crate::windows_specific;
/// windows_specific();
/// ```
```

## Module-Level Documentation

```rust
//! # Parser Module
//!
//! This module provides parsing utilities for shell commands.
//!
//! ## Overview
//!
//! The parser converts shell command strings into an Abstract Syntax Tree (AST)
//! that can be analyzed for security patterns.
//!
//! ## Usage
//!
//! ```
//! use my_crate::parser::Parser;
//!
//! let ast = Parser::new().parse("ls -la")?;
//! for node in ast.walk() {
//!     println!("{:?}", node);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Architecture
//!
//! - [`Parser`] - Main parsing interface
//! - [`Ast`] - The resulting syntax tree
//! - [`Node`] - Individual AST nodes

mod ast;
mod parser;

pub use ast::{Ast, Node};
pub use parser::Parser;
```

## Crate-Level Documentation

In `lib.rs`:

```rust
//! # My Crate
//!
//! `my_crate` provides utilities for X, Y, and Z.
//!
//! ## Features
//!
//! - Feature A: Does something useful
//! - Feature B: Does something else
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! my_crate = "1.0"
//! ```
//!
//! Basic usage:
//!
//! ```
//! use my_crate::prelude::*;
//!
//! let result = do_something()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Modules
//!
//! - [`parser`] - Command parsing
//! - [`rules`] - Rule engine
//! - [`config`] - Configuration management
```

## Using `#[doc]` Attributes

### `#[doc(hidden)]`

Hide items from documentation while keeping them public:

```rust
/// Public API function.
pub fn public_function() { ... }

/// Implementation detail, not part of public API.
#[doc(hidden)]
pub fn internal_function() { ... }
```

### `#[doc(alias)]`

Add search aliases:

```rust
/// A vector of elements.
#[doc(alias = "list")]
#[doc(alias = "array")]
pub struct Vec<T> { ... }
```

### `#[doc = include_str!()]`

Include external documentation:

```rust
#[doc = include_str!("../README.md")]
pub struct Crate;
```

## Cargo.toml Metadata

```toml
[package]
name = "my_crate"
version = "1.0.0"
edition = "2021"
description = "A brief description of what the crate does"
documentation = "https://docs.rs/my_crate"
repository = "https://github.com/user/my_crate"
readme = "README.md"
license = "MIT OR Apache-2.0"
keywords = ["keyword1", "keyword2", "keyword3"]
categories = ["development-tools", "command-line-utilities"]

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

## Documentation Style Guide

### Do

- Start with a brief one-line summary
- Use active voice: "Returns the length" not "The length is returned"
- Document all public items
- Include examples for complex functions
- Link to related items with `[`backticks`]`
- Use `# Errors`, `# Panics`, `# Safety` sections when applicable

### Don't

- Document the obvious: `/// Returns self.name` for `fn name(&self) -> &str`
- Use "Gets" - just describe what it returns: `/// The name of this widget`
- Include implementation details in public docs
- Write doc comments for private items (use `//` instead)

### Linking

```rust
/// See [`Config`] for configuration options.
///
/// Use [`Self::new`] to create a new instance.
///
/// Related: [`crate::parser::Parser`]
pub struct Builder { ... }
```

## Running Documentation Tests

```bash
# Run doc tests
cargo test --doc

# Build and open documentation
cargo doc --open

# Build docs for all features
cargo doc --all-features

# Check for broken links
cargo doc --no-deps 2>&1 | grep "warning"
```
