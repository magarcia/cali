# Safe Rust Pitfalls

This reference covers common pitfalls in safe Rust code that can lead to bugs, security issues, or unexpected behavior. These are all issues that compile successfully but may cause problems at runtime.

## Integer Overflow

### The Problem

In release mode, integer overflow wraps silently:

```rust
let x: u8 = 255;
let y = x + 1;  // Debug: panic! Release: y = 0
```

### Solutions

```rust
// Checked operations - return Option
let result = x.checked_add(1);  // None (would overflow)

// Saturating operations - clamp at bounds
let result = x.saturating_add(1);  // 255 (max value)

// Wrapping operations - explicit wrap
let result = x.wrapping_add(1);  // 0

// Overflowing operations - return (value, overflow)
let (result, overflow) = x.overflowing_add(1);  // (0, true)
```

### Best Practices

```rust
// For counters that shouldn't wrap
count = count.saturating_add(1);

// For sizes/indices where overflow is a bug
let new_len = old_len.checked_add(extra)
    .ok_or(Error::Overflow)?;

// For hashing/crypto where wrapping is intended
hash = hash.wrapping_mul(PRIME).wrapping_add(byte as u64);
```

## Unsafe `as` Casts

### The Problem

`as` casts can silently truncate, wrap, or change sign:

```rust
let big: u64 = 1000;
let small: u8 = big as u8;  // Silently truncates to 232

let negative: i32 = -1;
let unsigned: u32 = negative as u32;  // Becomes 4294967295
```

### Solutions

```rust
// Use TryFrom for fallible conversions
let small: u8 = big.try_into()?;

// Use From for infallible conversions
let bigger: u64 = small.into();

// Explicit bounds checking
let small: u8 = if big <= u8::MAX as u64 {
    big as u8
} else {
    return Err(Error::ValueTooLarge);
};
```

### Safe Cast Guidelines

```rust
// SAFE: Widening casts (smaller to larger)
let x: u32 = small_u8 as u32;      // u8 -> u32: always safe
let x: i64 = small_i32 as i64;      // i32 -> i64: always safe

// UNSAFE: Narrowing casts - use try_into()
let x: u8 = big_u32.try_into()?;

// UNSAFE: Sign changes - use try_into()
let x: u32 = signed_i32.try_into()?;
```

## Array Indexing Without Bounds Checking

### The Problem

Direct indexing panics on out-of-bounds:

```rust
let arr = [1, 2, 3];
let value = arr[index];  // Panic if index >= 3
```

### Solutions

```rust
// Use .get() for optional access
if let Some(value) = arr.get(index) {
    process(value);
}

// Use .get_mut() for mutable access
if let Some(value) = arr.get_mut(index) {
    *value = new_value;
}

// Use iterators to avoid indexing
for value in &arr {
    process(value);
}

// Pattern match on slices
match slice {
    [first, rest @ ..] => process_with_first(first, rest),
    [] => handle_empty(),
}
```

### When Direct Indexing Is Okay

```rust
// When you've just checked the bounds
if index < arr.len() {
    let value = arr[index];  // Safe
}

// When iterating with enumerate
for (i, value) in arr.iter().enumerate() {
    // i is guaranteed valid
}
```

## Sensitive Data in Debug Output

### The Problem

`#[derive(Debug)]` exposes all fields:

```rust
#[derive(Debug)]
struct Credentials {
    username: String,
    password: String,  // Exposed in debug output!
}

println!("{:?}", creds);  // Prints password!
```

### Solutions

```rust
// Custom Debug implementation
struct Credentials {
    username: String,
    password: String,
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

// Or use a newtype wrapper
#[derive(Clone)]
struct Secret<T>(T);

impl<T> std::fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

struct Credentials {
    username: String,
    password: Secret<String>,
}
```

## TOCTOU Race Conditions

### The Problem

Time-Of-Check to Time-Of-Use races in file operations:

```rust
// VULNERABLE: Check then use
if path.exists() {  // Time of check
    let content = std::fs::read_to_string(&path)?;  // Time of use
    // File could have been deleted/modified between check and use!
}
```

### Solutions

```rust
// GOOD: Just try the operation
match std::fs::read_to_string(&path) {
    Ok(content) => process(content),
    Err(e) if e.kind() == io::ErrorKind::NotFound => {
        // Handle missing file
    }
    Err(e) => return Err(e.into()),
}

// GOOD: Use atomic operations where available
use std::fs::OpenOptions;

let file = OpenOptions::new()
    .create_new(true)  // Fails if file exists
    .write(true)
    .open(&path)?;

// GOOD: Use exclusive file locks for critical sections
use fs2::FileExt;
let file = File::open(&path)?;
file.lock_exclusive()?;
// ... do work ...
file.unlock()?;
```

## Path Joining Surprises

### The Problem

`Path::join` replaces the path if the argument is absolute:

```rust
let base = Path::new("/home/user/data");
let file = Path::new("/etc/passwd");

let path = base.join(file);
// path is "/etc/passwd", not "/home/user/data/etc/passwd"!
```

### Solutions

```rust
// Check if path is relative
fn safe_join(base: &Path, relative: &Path) -> Option<PathBuf> {
    if relative.is_absolute() {
        return None;  // Reject absolute paths
    }

    let joined = base.join(relative);

    // Verify the result is still under base
    if joined.starts_with(base) {
        Some(joined)
    } else {
        None  // Path escapes base directory
    }
}

// Strip path components that escape
fn sanitize_path(path: &Path) -> PathBuf {
    path.components()
        .filter(|c| !matches!(c, Component::ParentDir | Component::RootDir))
        .collect()
}
```

## Uninitialized Memory with `MaybeUninit`

### The Problem

Incorrect use of `MaybeUninit` can lead to undefined behavior:

```rust
// WRONG: Reading uninitialized memory
let x: MaybeUninit<u32> = MaybeUninit::uninit();
let value = unsafe { x.assume_init() };  // UB!
```

### Safe Patterns

```rust
// GOOD: Initialize then assume_init
let mut x = MaybeUninit::<u32>::uninit();
x.write(42);
let value = unsafe { x.assume_init() };

// GOOD: Use zeroed for types where zero is valid
let x: MaybeUninit<u64> = MaybeUninit::zeroed();
let value = unsafe { x.assume_init() };  // Safe for numeric types

// GOOD: Initialize array elements
let mut arr: [MaybeUninit<u32>; 10] = unsafe {
    MaybeUninit::uninit().assume_init()
};
for (i, elem) in arr.iter_mut().enumerate() {
    elem.write(i as u32);
}
let arr = unsafe { std::mem::transmute::<_, [u32; 10]>(arr) };
```

## Forgetting to Handle `#[must_use]`

### The Problem

Ignoring important return values:

```rust
let mut vec = vec![1, 2, 3];
vec.pop();  // Warning: unused `Option` that must be used
// What was the value? Was there one?
```

### Solutions

```rust
// Handle the result
if let Some(value) = vec.pop() {
    process(value);
}

// Explicitly discard if intentional
let _ = vec.pop();

// For Results, handle or propagate
let file = File::open(path)?;  // Propagate
let _ = file.sync_all();       // Explicitly ignore (rare cases only)
```

### Add `#[must_use]` to Your Code

```rust
#[must_use = "this returns a new value and doesn't modify the original"]
pub fn with_capacity(capacity: usize) -> Self { ... }

#[must_use = "this `Result` may contain an error that should be handled"]
pub fn save(&self) -> Result<(), SaveError> { ... }
```

## Blocking in Async Code

### The Problem

Blocking calls in async contexts starve the runtime:

```rust
async fn process() {
    let data = std::fs::read_to_string("large_file.txt")?;  // BLOCKS!
    // Other tasks can't run while this blocks
}
```

### Solutions

```rust
// Use async I/O
use tokio::fs;
async fn process() {
    let data = fs::read_to_string("large_file.txt").await?;
}

// Use spawn_blocking for unavoidable sync code
async fn process() {
    let data = tokio::task::spawn_blocking(|| {
        std::fs::read_to_string("large_file.txt")
    }).await??;
}

// Use block_in_place in current thread runtime
async fn process() {
    let data = tokio::task::block_in_place(|| {
        std::fs::read_to_string("large_file.txt")
    })?;
}
```

## String Indexing

### The Problem

Rust strings are UTF-8, not byte arrays:

```rust
let s = "hello";
let c = s[0];  // ERROR: cannot index into String

let s = "héllo";
let c = &s[0..2];  // May panic or give unexpected results!
```

### Solutions

```rust
// Use chars() for character iteration
for c in s.chars() {
    process_char(c);
}

// Use char_indices() for positions
for (i, c) in s.char_indices() {
    println!("Char {} at byte {}", c, i);
}

// Get nth character
let third = s.chars().nth(2);

// Safe slicing with character boundaries
fn safe_slice(s: &str, start: usize, end: usize) -> Option<&str> {
    let mut indices = s.char_indices().map(|(i, _)| i).collect::<Vec<_>>();
    indices.push(s.len());

    let start_byte = *indices.get(start)?;
    let end_byte = *indices.get(end)?;
    Some(&s[start_byte..end_byte])
}
```

## Float Comparison

### The Problem

Floating point comparison is inexact:

```rust
let a = 0.1 + 0.2;
let b = 0.3;
assert_eq!(a, b);  // FAILS! a is 0.30000000000000004
```

### Solutions

```rust
// Use approximate comparison
fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() < epsilon
}

assert!(approx_eq(0.1 + 0.2, 0.3, 1e-10));

// Use a crate for proper float comparison
use float_cmp::approx_eq;
assert!(approx_eq!(f64, 0.1 + 0.2, 0.3, epsilon = 1e-10));

// Use integers for exact values (e.g., money)
struct Money(i64);  // Cents, not dollars
```

## Empty Iterator Assumptions

### The Problem

Operations on empty iterators can give unexpected results:

```rust
let empty: Vec<i32> = vec![];
let max = empty.iter().max();  // None, not panic
let sum: i32 = empty.iter().sum();  // 0, is this correct?
let product: i32 = empty.iter().product();  // 1, not 0!
```

### Solutions

```rust
// Handle empty case explicitly
let max = items.iter().max()
    .ok_or(Error::EmptyCollection)?;

// Use fold with explicit initial value
let sum = items.iter().fold(0, |acc, x| acc + x);

// Check for empty first
if items.is_empty() {
    return Err(Error::EmptyInput);
}
```
