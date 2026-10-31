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

## MPSC backpressure and progress

When an MPSC send waits for capacity, it can receive a reservation before it
is woken. Other sends cannot consume that reserved slot. Immediate `send()`
calls can therefore return `Full` even when the physical buffer has space.
Canceling a granted send transfers its capacity to the next waiter.

The receiver grants capacity when a full buffer first gains space and when
the buffer becomes empty. Consuming the final outstanding grant passes any
remaining capacity to the next waiter. This reduces wakeups during bursts
and lets a running producer fill unreserved space.

For example, suppose a four-slot queue is full and sends A and B are waiting.
The receiver removes two items and pauses. A receives a reserved slot; B
remains asleep. If no other sender uses the remaining space, A completing
its send grants that space to B without another receive. Canceling A also
passes its grant to B. Reserving every freed slot eagerly would instead
wake both A and B immediately.

The tradeoff is that B can wait for A despite available space. If A is never
polled or canceled and the receiver stops draining, B can remain asleep.
Draining the buffer to empty grants another waiter independently. Progress
depends on notified sends being polled or canceled and the receiver making
space as needed. Reservations protect individual sends; they do not promise
equal producer bandwidth or bounded waiting time.

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
