# Common Patterns

This reference covers idiomatic patterns and best practices for writing clean, maintainable Rust code.

## Iterators Over Loops

### Prefer Iterator Methods

```rust
// BAD: Explicit loop with mutation
let mut results = Vec::new();
for item in items {
    if item.is_valid() {
        results.push(item.transform());
    }
}

// GOOD: Iterator chain
let results: Vec<_> = items
    .iter()
    .filter(|item| item.is_valid())
    .map(|item| item.transform())
    .collect();
```

### Common Iterator Patterns

```rust
// Find first match
let found = items.iter().find(|item| item.id == target_id);

// Check if any/all match
let any_valid = items.iter().any(|item| item.is_valid());
let all_valid = items.iter().all(|item| item.is_valid());

// Sum/product
let total: i32 = numbers.iter().sum();
let product: i32 = numbers.iter().product();

// Fold for custom accumulation
let stats = items.iter().fold(Stats::default(), |acc, item| {
    acc.update(item)
});

// Partition into two collections
let (valid, invalid): (Vec<_>, Vec<_>) = items
    .into_iter()
    .partition(|item| item.is_valid());

// Flatten nested iterators
let all_children: Vec<_> = parents
    .iter()
    .flat_map(|p| p.children())
    .collect();

// Chain iterators
let combined: Vec<_> = first
    .iter()
    .chain(second.iter())
    .collect();
```

### When Explicit Loops Are Better

```rust
// Complex control flow with early returns
fn find_and_process(items: &[Item]) -> Option<Result<Output>> {
    for item in items {
        if item.should_skip() {
            continue;
        }

        match item.process() {
            Ok(output) if output.is_complete() => return Some(Ok(output)),
            Err(e) if e.is_fatal() => return Some(Err(e)),
            _ => {}
        }
    }
    None
}

// Mutating external state
for item in items {
    state.update(item);
    database.save(item)?;
}
```

## Pattern Matching

### Exhaustive Matching

```rust
// GOOD: Handle all cases explicitly
match result {
    Ok(value) => process(value),
    Err(Error::NotFound) => handle_not_found(),
    Err(Error::PermissionDenied) => handle_permission_error(),
    Err(Error::Network(e)) => handle_network_error(e),
}

// Use _ only when you truly want to ignore remaining cases
match command {
    "help" => show_help(),
    "version" => show_version(),
    _ => show_unknown_command(),
}
```

### Pattern Guards

```rust
match value {
    n if n < 0 => "negative",
    0 => "zero",
    n if n < 10 => "small positive",
    _ => "large positive",
}

match result {
    Ok(n) if n > threshold => handle_large(n),
    Ok(n) => handle_normal(n),
    Err(e) => handle_error(e),
}
```

### Destructuring

```rust
// Struct destructuring
let Point { x, y } = point;
let Point { x, .. } = point;  // Ignore remaining fields

// Enum destructuring
match message {
    Message::Move { x, y } => move_to(x, y),
    Message::Write(text) => print(text),
    Message::Quit => quit(),
}

// Nested destructuring
match response {
    Response { status: 200, body: Some(data) } => process(data),
    Response { status: 404, .. } => handle_not_found(),
    Response { status, .. } => handle_other(status),
}

// Slice patterns
match slice {
    [] => "empty",
    [single] => "one element",
    [first, second] => "two elements",
    [first, .., last] => "multiple elements",
}
```

### `if let` and `while let`

```rust
// if let for single pattern
if let Some(value) = optional {
    process(value);
}

// let-else for early return
let Some(value) = optional else {
    return Err(Error::MissingValue);
};

// while let for iteration
while let Some(item) = iterator.next() {
    process(item);
}
```

### `matches!` Macro

```rust
// Check if value matches pattern
if matches!(value, Some(n) if n > 10) {
    // ...
}

// In boolean contexts
let is_valid = matches!(state, State::Ready | State::Running);
```

## The `?` Operator

### Basic Usage

```rust
fn read_config(path: &Path) -> Result<Config, Error> {
    let content = std::fs::read_to_string(path)?;
    let config = toml::from_str(&content)?;
    Ok(config)
}
```

### With Option

```rust
fn get_first_char(s: &str) -> Option<char> {
    let first = s.chars().next()?;
    Some(first.to_ascii_uppercase())
}
```

### `ok_or` and `ok_or_else`

```rust
fn process(opt: Option<Value>) -> Result<Output, Error> {
    let value = opt.ok_or(Error::MissingValue)?;
    // Or with lazy error construction
    let value = opt.ok_or_else(|| Error::missing("field_name"))?;
    Ok(value.process())
}
```

## Const Evaluation

### Compile-Time Constants

```rust
const MAX_SIZE: usize = 1024;
const BUFFER_SIZE: usize = MAX_SIZE * 2;

// Const fn for compile-time computation
const fn compute_mask(bits: u32) -> u32 {
    (1 << bits) - 1
}

const BYTE_MASK: u32 = compute_mask(8);
```

### Static vs Const

```rust
// const: Inlined at each use site
const MESSAGE: &str = "Hello";

// static: Single memory location, has address
static COUNTER: AtomicUsize = AtomicUsize::new(0);

// Use const for:
// - Simple values that should be inlined
// - Configuration constants

// Use static for:
// - Values that need a stable memory address
// - Mutable state (with appropriate synchronization)
```

## Module Organization

### Flat Structure (Small Crates)

```
src/
├── lib.rs
├── parser.rs
├── analyzer.rs
└── error.rs
```

```rust
// lib.rs
mod parser;
mod analyzer;
mod error;

pub use parser::Parser;
pub use analyzer::Analyzer;
pub use error::{Error, Result};
```

### Nested Structure (Larger Crates)

```
src/
├── lib.rs
├── parser/
│   ├── mod.rs
│   ├── lexer.rs
│   └── ast.rs
├── rules/
│   ├── mod.rs
│   ├── engine.rs
│   └── builtin.rs
└── config/
    ├── mod.rs
    ├── schema.rs
    └── loader.rs
```

### Prelude Pattern

```rust
// prelude.rs
pub use crate::config::Config;
pub use crate::error::{Error, Result};
pub use crate::parser::Parser;
pub use crate::traits::{Analyze, Transform};

// Users can import common items easily
use my_crate::prelude::*;
```

## Trait Objects vs Generics

### When to Use Generics

```rust
// GOOD: Zero-cost abstraction, inlined
fn process<T: Display>(item: T) {
    println!("{}", item);
}

// GOOD: Different concrete types at each call site
fn sort<T: Ord>(slice: &mut [T]) {
    slice.sort();
}
```

### When to Use Trait Objects

```rust
// GOOD: Heterogeneous collection
let handlers: Vec<Box<dyn Handler>> = vec![
    Box::new(FileHandler),
    Box::new(NetworkHandler),
];

// GOOD: Reducing binary size / compile time
fn process(handler: &dyn Handler) {
    handler.handle();
}

// GOOD: Plugin systems where types aren't known at compile time
type Plugin = Box<dyn PluginTrait>;
fn load_plugins() -> Vec<Plugin> { ... }
```

## Default Implementations

### Using `#[derive(Default)]`

```rust
#[derive(Default)]
struct Config {
    name: String,        // Default: ""
    port: u16,           // Default: 0
    enabled: bool,       // Default: false
    items: Vec<Item>,    // Default: vec![]
}

let config = Config::default();
```

### Custom Default

```rust
struct Config {
    name: String,
    port: u16,
    timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            name: String::new(),
            port: 8080,  // Custom default
            timeout: Duration::from_secs(30),
        }
    }
}
```

### Struct Update Syntax

```rust
let config = Config {
    name: "my-app".into(),
    ..Config::default()  // Use defaults for remaining fields
};
```

## Entry API for Maps

```rust
use std::collections::HashMap;

let mut counts: HashMap<String, usize> = HashMap::new();

// BAD: Check then insert
if !counts.contains_key(key) {
    counts.insert(key.clone(), 0);
}
*counts.get_mut(key).unwrap() += 1;

// GOOD: Entry API
*counts.entry(key.clone()).or_insert(0) += 1;

// GOOD: With default value computation
counts.entry(key.clone()).or_insert_with(|| compute_default());

// GOOD: Entry API variants
match counts.entry(key) {
    Entry::Occupied(mut e) => *e.get_mut() += 1,
    Entry::Vacant(e) => { e.insert(1); }
}
```

## Interior Mutability

```rust
use std::cell::{Cell, RefCell};
use std::sync::{Mutex, RwLock};

// Cell for Copy types (single-threaded)
struct Counter {
    count: Cell<usize>,
}

impl Counter {
    fn increment(&self) {  // Note: &self, not &mut self
        self.count.set(self.count.get() + 1);
    }
}

// RefCell for non-Copy types (single-threaded)
struct Cache {
    data: RefCell<HashMap<Key, Value>>,
}

impl Cache {
    fn get_or_compute(&self, key: &Key) -> Value {
        if let Some(value) = self.data.borrow().get(key) {
            return value.clone();
        }
        let value = compute(key);
        self.data.borrow_mut().insert(key.clone(), value.clone());
        value
    }
}

// Mutex for thread-safe mutation
struct SharedState {
    data: Mutex<Data>,
}

// RwLock for read-heavy workloads
struct ReadHeavyCache {
    data: RwLock<HashMap<Key, Value>>,
}
```

## RAII and Drop

```rust
struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new() -> io::Result<Self> {
        let path = generate_temp_path();
        std::fs::File::create(&path)?;
        Ok(TempFile { path })
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

// File automatically deleted when temp goes out of scope
{
    let temp = TempFile::new()?;
    do_work(&temp.path)?;
}  // temp file deleted here
```
