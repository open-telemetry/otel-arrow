# PData Codec Benchmark Method

Use the `payload_measurements` and `syslog_cef_receiver` Criterion benchmarks to
compare representation changes with the legacy implementation.

## Controlled comparison

1. Build release benchmark executables for the baseline and candidate commits.
2. Select an otherwise idle logical CPU and confirm that its SMT sibling is idle.
3. Pin every benchmark process to that CPU.
4. Alternate baseline and candidate runs to reduce temperature and run-order bias.
5. Use identical warm-up, measurement, sample-count, and input-size settings.
6. Retain Criterion confidence intervals and record the tested commit IDs.

Matching-representation forwarding must preserve the original `Bytes` buffer and
must not allocate a representation wrapper. Native OTAP conversion must move the
records directly. If confidence intervals do not overlap, repeat the interleaved
runs and use CPU counters or allocation profiling before attributing a regression
to the codec abstraction.

HTTP compression measurements include encoding directly from the reusable output
buffer. Keep transport compression separate from codec encoding in reports.
