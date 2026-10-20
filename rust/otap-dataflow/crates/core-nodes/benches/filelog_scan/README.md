# Filelog scanner benchmark

Run on Linux from `rust/otap-dataflow`:

```sh
cargo bench -p otel-arrow-dfe-core-nodes --no-default-features --bench filelog_scan
cargo test -p otel-arrow-dfe-core-nodes --no-default-features \
  --test filelog_scan_allocations
```

Set `FILELOG_BENCH_FILES` to change the file count (default: 256; range:
128 to 100,000). Set `FILELOG_BENCH_ROOT` to choose the fixture filesystem.

The benchmark compares full validation with presence-only checks for known files
across flat, nested, followed-link, and directory-churn workloads, with exclusions.
CSV output reports scan timings, observed files, and complete-pass counts.
The allocation test checks buffer reuse and stable retained heap.

Results cover 20 warm-cache passes; p99 is their maximum, not a latency guarantee.
These are scanner timings, not end-to-end receiver latency. Directory churn can
leave scans incomplete even when all static files are observed.

Filesystem tests have passed on ext4, tmpfs, XFS, and btrfs, with separate overlayfs
replay tests on Linux 7.0.0-1017-azure. This does not qualify other configurations.
See the [scanner guide](../../src/receivers/filelog_receiver/docs/discovery.md)
for behavior and qualification requirements.
