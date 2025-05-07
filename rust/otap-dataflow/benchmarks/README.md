# 📈 Benchmarks

**Status:** 🚧 *Work in Progress*

## Benches
This workspace includes **Criterion-based micro-benchmarks** for the `otap-dataflow` crates. These benchmarks help evaluate and track the performance of individual components and cross-cutting functionality over time.


### 📁 Bench Directory Structure

All benchmarks are defined under the `benchmarks/benches/` directory. The organization mirrors the structure of the crates in `crates/`:

- 📦 **Crate-specific benchmarks**:  
  Located in subdirectories named after the crate, e.g., `benchmarks/benches/config/` for `crates/config`.

- 🔄 **Cross-crate or e2e benchmarks**:  
  General-purpose or integration-style benchmarks that span multiple crates may be placed in directories such as `benchmarks/benches/e2e/`.


### ➕ Adding New Benchmarks

To add a benchmark for a new crate:

#### 1. Add the crate as a dependency in `benchmarks/Cargo.toml`

Example:

```toml
[dependencies]
otap-df-config = { path = "../crates/config" }
```


#### 2. Add a [[bench]] entry for the benchmark target
This declares the benchmark file and disables the default test harness (as required by Criterion):

```toml
[[bench]]
name = "config"
harness = false
```

This will expect a file at: `benchmarks/benches/config/main.rs`

#### 3. Create the benchmark file
Use Criterion’s macro-based structure. Example:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use otap_df_config::some_function;

fn bench_some_function(c: &mut Criterion) {
    c.bench_function("some_function", |b| b.iter(|| some_function()));
}

criterion_group!(benches, bench_some_function);
criterion_main!(benches);
```


### 🏃 Running Benchmarks
From the workspace root:

```bash
cargo bench -p benchmarks
```
---