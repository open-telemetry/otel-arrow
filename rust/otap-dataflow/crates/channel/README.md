# Channel implementations optimized for single-threaded async runtime

This crate contains various implementations of optimized asynchronous channels
for:

- A single-threaded async runtime
- Detailed instrumentation (not yet implemented)
- Maximum control within the context of this project

Current implementations include:

- MPMC: A multi-producer multi-consumer channel
- MPSC: A multi-producer single-consumer channel

Implementations not yet available: SPSC and broadcast.
