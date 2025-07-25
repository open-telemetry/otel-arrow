# Async Best Practices for OTAP Dataflow

This document outlines best practices for writing async-safe code in the OTAP
Dataflow project.

## Common Blocking Operations to Avoid

### 1. Synchronous I/O Operations

**Don't use:**

```rust
use std::io::Write;
use std::fs::File;

async fn bad_example() {
    let mut file = File::create("example.txt").unwrap(); // BLOCKING!
    file.write_all(b"data").unwrap(); // BLOCKING!
}
```

**Use instead:**

```rust
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

async fn good_example() {
    let mut file = File::create("example.txt").await.unwrap();
    file.write_all(b"data").await.unwrap();
}
```

### 2. File System Operations

**Don't use:**

```rust
use std::fs;

async fn bad_fs_operations() {
    let contents = std::fs::read_to_string("file.txt").unwrap(); // BLOCKING!
    std::fs::write("output.txt", "data").unwrap(); // BLOCKING!
}
```

**Use instead:**

```rust
use tokio::fs;

async fn good_fs_operations() {
    let contents = fs::read_to_string("file.txt").await.unwrap();
    fs::write("output.txt", "data").await.unwrap();
}
```

### 3. Thread Sleep

**Don't use:**

```rust
async fn bad_sleep() {
    std::thread::sleep(Duration::from_secs(1)); // BLOCKING!
}
```

**Use instead:**

```rust
use tokio::time::{sleep, Duration};

async fn good_sleep() {
    sleep(Duration::from_secs(1)).await;
}
```

### 4. Synchronous Network Operations

**Don't use:**

```rust
use std::net::TcpStream;

async fn bad_network() {
    let stream = TcpStream::connect("127.0.0.1:8080").unwrap(); // BLOCKING!
}
```

**Use instead:**

```rust
use tokio::net::TcpStream;

async fn good_network() {
    let stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
}
```

## 🛠️ Script To Detect Issues

Run our custom script to detect blocking operations:

```bash
./scripts/check-async-blocking.sh
```

This script checks for:

- `std::io::Write` usage in async contexts
- Blocking `std::fs` operations
- Blocking `File::open/create` calls
- `thread::sleep` usage

## When You Must Use Blocking Operations

Sometimes you need to use libraries that don't have async versions. Use
`spawn_blocking`:

```rust
use tokio::task::spawn_blocking;

async fn use_blocking_library() -> Result<String, Box<dyn std::error::Error>> {
    let result = spawn_blocking(|| {
        // Blocking operation here
        some_blocking_library_call()
    }).await?;

    Ok(result)
}
```

## Code Review Checklist

When reviewing async code, check for:

- [ ] All I/O operations use async alternatives (`tokio::fs`, `tokio::io`,
  `tokio::net`)
- [ ] No `std::thread::sleep` - use `tokio::time::sleep` instead
- [ ] Blocking operations are wrapped in `spawn_blocking`
- [ ] No locks held across `.await` points

## ⚡ Quick Reference

| Blocking              | Async Alternative       |
|-----------------------|-------------------------|
| `std::fs::read`       | `tokio::fs::read`       |
| `std::fs::write`      | `tokio::fs::write`      |
| `std::fs::File`       | `tokio::fs::File`       |
| `std::io::Write`      | `tokio::io::AsyncWrite` |
| `std::io::Read`       | `tokio::io::AsyncRead`  |
| `std::net::TcpStream` | `tokio::net::TcpStream` |
| `std::thread::sleep`  | `tokio::time::sleep`    |
| `std::sync::mpsc`     | `tokio::sync::mpsc`     |
