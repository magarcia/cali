# Error Handling

This reference covers error handling patterns in Rust, including error type design for libraries and applications, and effective use of `Result` and `?`.

## Core Philosophy

1. **Use `Result` for recoverable errors** - Operations that can fail in expected ways
2. **Use `panic!` for bugs** - Programming errors that shouldn't happen
3. **Make errors informative** - Include context about what went wrong
4. **Don't leak implementation details** - Abstract over internal dependencies

## Library vs Application Error Design

### Library Crates

Libraries should define specific, well-documented error types:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file '{path}'")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid config format at line {line}")]
    Parse {
        line: usize,
        #[source]
        source: toml::de::Error,
    },

    #[error("missing required field '{field}'")]
    MissingField { field: &'static str },

    #[error("invalid value for '{field}': {message}")]
    InvalidValue {
        field: &'static str,
        message: String,
    },
}
```

**Key principles:**
- Use `#[derive(Debug, Error)]` with `thiserror`
- Make each variant meaningful to callers
- Include relevant context (file paths, line numbers, field names)
- Use `#[source]` to chain underlying errors
- Document what causes each error variant

### Application Binaries

Applications can use `anyhow` for convenience:

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config()
        .context("failed to initialize application")?;

    run_server(&config)
        .context("server error")?;

    Ok(())
}

fn load_config() -> Result<Config> {
    let path = find_config_path()
        .context("could not find config file")?;

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    toml::from_str(&content)
        .context("invalid config format")
}
```

**Key principles:**
- Use `anyhow::Result<T>` as the default return type
- Add context with `.context()` or `.with_context()`
- Context should explain the high-level operation that failed

## Avoiding Inner Error Type Leakage

**Problem:** Exposing internal error types creates tight coupling:

```rust
// BAD: Leaks std::io::Error and serde_json::Error
pub enum ApiError {
    Io(std::io::Error),              // Internal detail
    Json(serde_json::Error),          // Internal detail
    Http(reqwest::Error),             // Internal detail
}
```

**Solution:** Abstract over internal errors:

```rust
// GOOD: Custom error types
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("network request failed: {message}")]
    Network {
        message: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("invalid response format")]
    InvalidResponse {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("resource not found: {resource}")]
    NotFound { resource: String },

    #[error("rate limited, retry after {retry_after:?}")]
    RateLimited { retry_after: Option<Duration> },
}

// Internal conversion
impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::Network {
            message: err.to_string(),
            source: Box::new(err),
        }
    }
}
```

## Error Type Patterns

### Using `thiserror`

The `thiserror` crate provides derive macros for `std::error::Error`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    // Simple message
    #[error("unexpected end of input")]
    UnexpectedEof,

    // With fields in message
    #[error("invalid character '{ch}' at position {pos}")]
    InvalidChar { ch: char, pos: usize },

    // With source error
    #[error("integer overflow")]
    Overflow(#[from] std::num::ParseIntError),

    // Transparent wrapper
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
```

### Boxed Trait Objects

For performance-critical code where error variants are rare:

```rust
// Option 1: Direct boxed error
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// Option 2: Named error type alias (pick one, not both)
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<T, BoxError>;
```

### Error Codes for FFI

When crossing FFI boundaries:

```rust
#[repr(C)]
pub enum ErrorCode {
    Success = 0,
    InvalidInput = 1,
    NotFound = 2,
    PermissionDenied = 3,
    InternalError = -1,
}

// Keep detailed error internally
thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

pub fn get_last_error() -> Option<String> {
    LAST_ERROR.with(|e| e.borrow().clone())
}
```

## Using the `?` Operator

### Basic Usage

```rust
fn read_config(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;  // Propagates io::Error
    let config = toml::from_str(&content)?;         // Propagates toml::Error
    Ok(config)
}
```

### With Conversion

The `?` operator calls `From::from` on the error:

```rust
impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::Read {
            path: PathBuf::new(),  // Would need actual path in real code
            source: err,
        }
    }
}

// Now ? works automatically
fn read_file(path: &Path) -> Result<String, ConfigError> {
    Ok(std::fs::read_to_string(path)?)  // io::Error converted to ConfigError
}
```

### Adding Context

```rust
// With anyhow
use anyhow::Context;

fn process_file(path: &Path) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    Ok(())
}

// With custom errors
fn process_file(path: &Path) -> Result<(), MyError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| MyError::FileRead {
            path: path.to_owned(),
            source: e,
        })?;
    Ok(())
}
```

## When to Panic

### Appropriate Panic Cases

1. **Programming errors / invariant violations:**
```rust
fn get_unchecked(&self, index: usize) -> &T {
    assert!(index < self.len(), "index out of bounds");
    // ...
}
```

2. **Unrecoverable initialization failures:**
```rust
fn init_logger() {
    env_logger::try_init().expect("logger already initialized");
}
```

3. **Test assertions:**
```rust
#[test]
fn test_parse() {
    let result = parse("valid input").unwrap();
    assert_eq!(result, expected);
}
```

### Avoid Panicking

1. **In library code** - Return `Result` instead
2. **For expected failures** - File not found, network timeout, invalid input
3. **In `Drop` implementations** - Can cause double-panic

### `unwrap()` and `expect()` Guidelines

```rust
// GOOD: In tests
#[test]
fn test_something() {
    let result = function().unwrap();
}

// GOOD: When logic guarantees validity
let values = vec![1, 2, 3];
if !values.is_empty() {
    let first = values.first().unwrap();  // Safe: we just checked non-empty
}

// GOOD: With expect() and clear message
let home = std::env::var("HOME")
    .expect("HOME environment variable must be set");

// BAD: In library code without checks
pub fn process(input: &str) -> Output {
    let parsed = parse(input).unwrap();  // Don't do this
}
```

## Error Handling in Async Code

```rust
use tokio::task::JoinError;

async fn fetch_all(urls: Vec<String>) -> Result<Vec<Response>, FetchError> {
    let handles: Vec<_> = urls
        .into_iter()
        .map(|url| tokio::spawn(fetch_one(url)))
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        let response = handle.await
            .map_err(FetchError::TaskPanic)?  // Handle JoinError
            .map_err(FetchError::Network)?;    // Handle fetch error
        results.push(response);
    }

    Ok(results)
}
```

## Custom Result Type Alias

Define a crate-level `Result` type:

```rust
// In lib.rs or error.rs
pub type Result<T, E = Error> = std::result::Result<T, E>;

// Usage throughout crate
pub fn do_something() -> Result<Value> { ... }
pub fn with_custom_error() -> Result<Value, CustomError> { ... }
```

## Error Reporting

For user-facing error messages:

```rust
fn report_error(err: &dyn std::error::Error) {
    eprintln!("Error: {err}");

    // Print error chain
    let mut source = err.source();
    while let Some(cause) = source {
        eprintln!("Caused by: {cause}");
        source = cause.source();
    }
}

// With anyhow, use {:#} for full chain
fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err:#}");
        std::process::exit(1);
    }
}
```
