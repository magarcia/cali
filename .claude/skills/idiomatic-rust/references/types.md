# Type Safety

This reference covers type-safe design patterns in Rust, including newtypes, builders, and techniques for making invalid states unrepresentable.

## The Newtype Pattern

Newtypes wrap primitive types to add type safety and domain meaning.

### Basic Newtype

```rust
// Primitive obsession - easy to mix up arguments
fn process_order(user_id: u64, order_id: u64, product_id: u64) { ... }

// Newtype pattern - compiler catches mistakes
struct UserId(u64);
struct OrderId(u64);
struct ProductId(u64);

fn process_order(user_id: UserId, order_id: OrderId, product_id: ProductId) { ... }

// Compiler error: expected UserId, found OrderId
process_order(order_id, user_id, product_id);  // Won't compile!
```

### Newtype with Validation

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

#[derive(Debug, thiserror::Error)]
#[error("invalid email address: {0}")]
pub struct InvalidEmail(String);

impl Email {
    pub fn new(s: impl Into<String>) -> Result<Self, InvalidEmail> {
        let s = s.into();
        if s.contains('@') && s.len() >= 3 {
            Ok(Email(s))
        } else {
            Err(InvalidEmail(s))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Cannot create invalid Email
let email = Email::new("invalid")?;  // Returns Err
let email = Email::new("user@example.com")?;  // Returns Ok
```

### Deriving Common Traits

```rust
use derive_more::{Display, From, Into, AsRef, Deref};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, From, AsRef, Deref)]
pub struct Username(String);

// Or manually implement what you need
impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

## Making Invalid States Unrepresentable

### Use Enums for Mutually Exclusive States

```rust
// BAD: Multiple booleans allow invalid combinations
struct Connection {
    is_connected: bool,
    is_authenticated: bool,
    is_disconnected: bool,  // What if connected AND disconnected?
}

// GOOD: Enum makes invalid states impossible
enum ConnectionState {
    Disconnected,
    Connected { socket: TcpStream },
    Authenticated { socket: TcpStream, user: User },
}
```

### Use Types to Enforce Invariants

```rust
// BAD: Range could be invalid (start > end)
struct DateRange {
    start: DateTime,
    end: DateTime,
}

// GOOD: Constructor enforces invariant
pub struct DateRange {
    start: DateTime,
    end: DateTime,
}

impl DateRange {
    pub fn new(start: DateTime, end: DateTime) -> Option<Self> {
        if start <= end {
            Some(DateRange { start, end })
        } else {
            None
        }
    }

    // Guaranteed: start <= end
    pub fn duration(&self) -> Duration {
        self.end - self.start
    }
}
```

### NonZero Types

```rust
use std::num::NonZeroU32;

struct Pagination {
    page: NonZeroU32,      // Guaranteed non-zero
    per_page: NonZeroU32,
}

impl Pagination {
    pub fn offset(&self) -> u32 {
        // No need to check for zero - type guarantees it
        (self.page.get() - 1) * self.per_page.get()
    }
}
```

## The Builder Pattern

### Basic Builder

```rust
pub struct Server {
    host: String,
    port: u16,
    max_connections: usize,
    timeout: Duration,
}

#[derive(Default)]
pub struct ServerBuilder {
    host: Option<String>,
    port: Option<u16>,
    max_connections: Option<usize>,
    timeout: Option<Duration>,
}

impl ServerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = Some(max);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> Result<Server, BuilderError> {
        Ok(Server {
            host: self.host.ok_or(BuilderError::MissingField("host"))?,
            port: self.port.unwrap_or(8080),
            max_connections: self.max_connections.unwrap_or(100),
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
        })
    }
}

// Usage
let server = ServerBuilder::new()
    .host("localhost")
    .port(3000)
    .timeout(Duration::from_secs(60))
    .build()?;
```

### Type-State Builder

Enforce required fields at compile time:

```rust
pub struct ServerBuilder<Host, Port> {
    host: Host,
    port: Port,
    max_connections: usize,
}

pub struct Missing;
pub struct Set<T>(T);

impl ServerBuilder<Missing, Missing> {
    pub fn new() -> Self {
        ServerBuilder {
            host: Missing,
            port: Missing,
            max_connections: 100,
        }
    }
}

impl<Port> ServerBuilder<Missing, Port> {
    pub fn host(self, host: String) -> ServerBuilder<Set<String>, Port> {
        ServerBuilder {
            host: Set(host),
            port: self.port,
            max_connections: self.max_connections,
        }
    }
}

impl<Host> ServerBuilder<Host, Missing> {
    pub fn port(self, port: u16) -> ServerBuilder<Host, Set<u16>> {
        ServerBuilder {
            host: self.host,
            port: Set(port),
            max_connections: self.max_connections,
        }
    }
}

// build() only available when both Host and Port are Set
impl ServerBuilder<Set<String>, Set<u16>> {
    pub fn build(self) -> Server {
        Server {
            host: self.host.0,
            port: self.port.0,
            max_connections: self.max_connections,
        }
    }
}

// Compile error if host or port not set
let server = ServerBuilder::new()
    .host("localhost".into())
    // .port(3000)  // Missing!
    .build();  // Error: build() not available
```

## Type-State Pattern

Use different types to represent different states:

```rust
use std::marker::PhantomData;

pub struct Closed;
pub struct Open;

pub struct File<State> {
    path: PathBuf,
    handle: Option<std::fs::File>,
    _state: PhantomData<State>,
}

impl File<Closed> {
    pub fn new(path: PathBuf) -> Self {
        File {
            path,
            handle: None,
            _state: PhantomData,
        }
    }

    pub fn open(self) -> io::Result<File<Open>> {
        let handle = std::fs::File::open(&self.path)?;
        Ok(File {
            path: self.path,
            handle: Some(handle),
            _state: PhantomData,
        })
    }
}

impl File<Open> {
    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // SAFETY: handle is always Some in Open state
        self.handle.as_mut().unwrap().read(buf)
    }

    pub fn close(self) -> File<Closed> {
        // handle dropped here
        File {
            path: self.path,
            handle: None,
            _state: PhantomData,
        }
    }
}

// Can't read from closed file - won't compile
let file = File::new("data.txt".into());
file.read(&mut buf);  // Error: read() doesn't exist on File<Closed>
```

## Using From and TryFrom

### Infallible Conversions with From

```rust
impl From<UserId> for u64 {
    fn from(id: UserId) -> u64 {
        id.0
    }
}

impl From<u64> for UserId {
    fn from(n: u64) -> UserId {
        UserId(n)
    }
}

// Usage
let id: UserId = 42u64.into();
let n: u64 = id.into();
```

### Fallible Conversions with TryFrom

```rust
impl TryFrom<i64> for UserId {
    type Error = InvalidId;

    fn try_from(n: i64) -> Result<Self, Self::Error> {
        if n >= 0 {
            Ok(UserId(n as u64))
        } else {
            Err(InvalidId::Negative)
        }
    }
}

// Usage
let id: UserId = 42i64.try_into()?;
let id: UserId = (-1i64).try_into()?;  // Returns Err
```

### Prefer TryFrom over `as` Casts

```rust
// BAD: Silent truncation
let n: u8 = large_number as u8;  // Silently wraps

// GOOD: Explicit handling
let n: u8 = large_number.try_into()
    .map_err(|_| MyError::NumberTooLarge)?;

// GOOD: Explicit saturation
let n: u8 = large_number.min(u8::MAX as u64) as u8;
```

## Bitflags for Flag Sets

```rust
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Permissions: u32 {
        const READ    = 0b0001;
        const WRITE   = 0b0010;
        const EXECUTE = 0b0100;
        const ALL     = Self::READ.bits() | Self::WRITE.bits() | Self::EXECUTE.bits();
    }
}

impl Permissions {
    pub fn can_read(self) -> bool {
        self.contains(Self::READ)
    }

    pub fn can_write(self) -> bool {
        self.contains(Self::WRITE)
    }
}

// Usage
let perms = Permissions::READ | Permissions::WRITE;
if perms.can_read() { ... }
```

## Smart Pointers for Ownership

Choose the right smart pointer:

| Type | Use Case |
|------|----------|
| `Box<T>` | Single ownership, heap allocation |
| `Rc<T>` | Multiple owners, single-threaded |
| `Arc<T>` | Multiple owners, thread-safe |
| `Cow<'a, T>` | Clone-on-write, avoid allocation when possible |
| `Cell<T>` | Interior mutability, Copy types |
| `RefCell<T>` | Interior mutability, runtime borrow checking |
| `Mutex<T>` | Interior mutability, thread-safe |
| `RwLock<T>` | Interior mutability, multiple readers |

```rust
// Cow for optional cloning
fn process(input: Cow<'_, str>) -> Cow<'_, str> {
    if needs_modification(&input) {
        Cow::Owned(modify(input.into_owned()))
    } else {
        input
    }
}
```
