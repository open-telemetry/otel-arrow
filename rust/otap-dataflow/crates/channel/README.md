# otel-arrow-dfe-channel

This crate is currently pre-1.0. Its public API may evolve between minor
releases.

Asynchronous channel implementations optimized for single-threaded runtimes.

## Overview

This crate provides channels designed for:

- A single-threaded async runtime
- Detailed instrumentation (not yet implemented)
- Maximum control within the context of this project

Current implementations include:

- MPMC: A multi-producer multi-consumer channel
- MPSC: A multi-producer single-consumer channel

Implementations not yet available: SPSC and broadcast.

## Types

Both the `mpsc` and `mpmc` modules expose:

- `Channel<T>`: creates a bounded channel
- `Sender<T>`: sends immediately or waits asynchronously for capacity
- `Receiver<T>`: receives asynchronously

The MPSC receiver has one consumer. The MPMC receiver is cloneable for
multiple consumers. These channels use `Rc` internally and are intended for
tasks running on the same thread.

## Usage

```sh
cargo add otel-arrow-dfe-channel
```

```rust
use otel_arrow_dfe_channel::mpsc::Channel;

async fn round_trip() {
    let (sender, receiver) = Channel::new(8);

    sender.send("hello").expect("channel has capacity");
    assert_eq!(receiver.recv().await.expect("sender is open"), "hello");
}
```
