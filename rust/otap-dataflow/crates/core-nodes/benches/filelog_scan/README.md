# Filelog scanner benchmark

Run on Linux from `rust/otap-dataflow`:

```sh
cargo bench -p otel-arrow-dfe-core-nodes --no-default-features --bench filelog_scan
cargo test -p otel-arrow-dfe-core-nodes --no-default-features \
  --test filelog_scan_allocations
```

Set `FILELOG_BENCH_FILES` to change the file count (default: 256; range:
128 to 100,000). Set `FILELOG_BENCH_ROOT` to choose an ext4, XFS, or btrfs
fixture. Replay backends are covered by the filesystem tests.

The benchmark compares full validation with presence-only checks for known files
across flat, nested, followed-link, and directory-churn workloads, with exclusions.
CSV output reports scan timings, observed files, and complete-pass counts.
The allocation test covers 255/256/257-byte and longer native paths, buffer
reuse, and stable retained heap.

Results cover 20 warm-cache passes; p99 is their maximum, not a latency guarantee.
These are scanner timings, not end-to-end receiver latency. Directory churn can
leave scans incomplete even when all static files are observed.

Filesystem tests cover ext4, XFS, btrfs, tmpfs, and overlayfs on Linux
7.0.0-1017-azure. Tmpfs and overlayfs replay remains conservative: successful
prefix verification does not qualify complete resumed scans or other kernels.
See the [scanner guide](../../src/receivers/filelog_receiver/docs/discovery.md)
for behavior and qualification requirements.
