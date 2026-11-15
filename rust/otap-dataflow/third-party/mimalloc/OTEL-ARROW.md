# Microsoft mimalloc

The native allocator and Rust bindings are from [mimalloc v3.6.1][upstream],
commit `56dc5eb80145f9a3ba7ca23d47c6a248b9d8586c`, under the [MIT license](LICENSE).
The vendored paths are `LICENSE`, `include/`, `src/`, and `contrib/rust/`.

When updating the pin, review these local changes against the new release.

- `contrib/rust/sys/build.rs` builds the pinned source and enables the unwinder
  and POSIX file protections used by the built-in profiler.
- `src/profile/pprof.c` records Linux mapping offsets in protobuf snapshots and
  can write an in-memory snapshot of the environment-enabled profiler. Snapshot
  encoders and sampling callbacks share atomic counters so allocations and
  frees continue to be recorded during a dump.
- `include/mimalloc-profile.h` declares the in-memory snapshot entry point.

[upstream]: https://github.com/microsoft/mimalloc/tree/v3.6.1
