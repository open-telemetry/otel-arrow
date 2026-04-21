# Fake Data Generator -- SUT Saturation Benchmarks

## Purpose

Verify that a **single fake-gen sender core can saturate a single
`df_engine` (SUT) core** over OTLP gRPC, so that performance tests
don't need multiple load-generator cores.

```text
  sender (1 core)           SUT (1 core)            backend (1 core)
  fake-gen -> OTLP-export --> OTLP-recv -> OTLP-export --> OTLP-recv -> noop
       :8080                     :8081                      :8082
```

## Configs (`bench/`)

| File                        | Role        | Pipeline                          |
|-----------------------------|-------------|-----------------------------------|
| `sender-static-fresh.yaml`  | Load gen    | fake-gen(fresh) -> OTLP export    |
| `sender-static-pregen.yaml` | Load gen    | fake-gen(pregen) -> OTLP export   |
| `sut-otlp-forward.yaml`     | SUT         | OTLP recv -> OTLP export          |
| `backend-noop.yaml`         | Backend     | OTLP recv -> noop                 |

## Quick start

```bash
cd rust/otap-dataflow
cargo build --release
bash crates/core-nodes/src/receivers/fake_data_generator/bench/bench.sh
```

---

## Results

Apple M3 Pro (14 cores, 24 GB), 1 pipeline core per process, release
build, logs only. Averaged over 4 runs.

| Config              | Throughput       | Sender core | SUT core     |
|---------------------|------------------|-------------|--------------|
| static/fresh 1KB    | ~480 K logs/sec  | ~82 %       | **~97 %**    |
| static/pregen 1KB   | ~500 K logs/sec  | ~66 %       | **~95 %**    |

---

## Conclusion

**Use `static` + `pre_generated` + `log_body_size_bytes: 1024`.**

This configuration saturates the SUT to ~95 % while consuming only
~66 % of a single sender core. `pre_generated` uses ~16 % less sender
CPU than `fresh` (~66 % vs ~82 %) at similar throughput, leaving more
headroom. One load-gen core is sufficient!
