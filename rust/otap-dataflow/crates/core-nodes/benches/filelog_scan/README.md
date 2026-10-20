# Filelog scanner measurements

Build and run the repository benchmark on Linux:

```sh
cargo bench -p otel-arrow-dfe-core-nodes --no-default-features --bench filelog_scan
FILELOG_BENCH_FILES=4096 \
  cargo bench -p otel-arrow-dfe-core-nodes --no-default-features --bench filelog_scan
cargo test -p otel-arrow-dfe-core-nodes --no-default-features \
  --test filelog_scan_allocations
```

Set `FILELOG_BENCH_ROOT` to a directory on the filesystem being measured. The
regular-file population is controlled by `FILELOG_BENCH_FILES` (256 by default,
128 through 100,000 supported). Each regular file has an excluded counterpart.
Followed-link cases use a separate watched directory pointing to the nested
source tree. Every scenario configures an exclusion-root mapping.

The comparison uses the same scanner and tree with two requests: full candidate
validation and presence-only observation of already-known locators. The retained
lookup is prepared outside timing. Presence-only results cannot authorize
admission, reconnection, or cancellation of pending disappearance. New locators
still receive full validation.

Twenty measured passes follow a warm-up pass. Initial corpus creation, retained
lookup construction, and CSV output are outside timing. Intentional directory
mutations in churn scenarios are included in timing. The reported p99 is the largest
of twenty samples, not a tail-latency SLO. These are synchronous scanner timings,
not integrated receiver latency. Discovery normally runs on its separate
completion-timed reconciliation schedule, not the reader's EOF-reprobe interval.

## Churn and coverage

`churn_direct` and `churn_links` change an empty pod directory during every fourth
pass. `continuous_churn` changes it during every pass. The benchmark checks that
static files still produce positive observations, while affected passes remain
incomplete. It reports complete-pass counts alongside observation counts.

Continuous detected churn can therefore stall discovery-derived finalization
and retention even while collection continues. The benchmark exposes this cost;
it does not relax the specification's absence rules. Measuring pinned readers,
confirmation delays, and admission pressure also requires receiver integration.

## Heap and filesystem qualification

The isolated allocation test combines flat and six-level trees, excluded files,
and followed aliases. It alternates full and presence-only requests after warming
the bounded buffers. It checks both retained heap and ongoing allocation count;
caller-owned retained lookup memory has a separate owner.

The filesystem tests exercise stable multi-batch and nested traversal, mutation,
root replacement, and positive progress after lost coverage. Set `TMPDIR` to a
mounted test filesystem when running those tests. Privileged bind-mount, chroot,
and overlayfs tests must run in an isolated mount namespace. Source access and
directory resumption require their own qualification; passing on one filesystem
or kernel does not qualify all configurations of another.

## Example measurements

The repository benchmark on ext4, Linux 7.0.0-1017-azure, Intel Xeon 6973P-C
(48 logical CPUs), produced these warm-cache results. The optimized bench profile
used `debug=0`; workspace validation ran concurrently, so these are comparative
measurements rather than isolated performance limits.

| Regular files | Layout | Full p50 / p99 (ms) | Presence-only p50 / p99 (ms) |
| --- | --- | --- | --- |
| 256 | Flat | 6.65 / 7.32 | 2.94 / 2.96 |
| 256 | Six-level | 9.34 / 9.52 | 4.52 / 4.59 |
| 256 | Followed links | 10.88 / 11.00 | 5.99 / 6.05 |
| 4,096 | Flat | 121.52 / 126.67 | 52.58 / 54.99 |
| 4,096 | Six-level | 169.27 / 181.52 | 89.11 / 93.83 |
| 4,096 | Followed links | 188.66 / 202.80 | 104.58 / 114.25 |

All stable workloads completed 20 of 20 passes. Both periodic-churn layouts
completed 15 of 20, and continuous churn completed none. Every pass still
observed the full static file population, including at 4,096 files. These results
show lower cost for known-file observations, not zero overhead or guaranteed
end-to-end latency. Continuous churn still prevents negative conclusions.

The mixed flat/nested allocation test retained 441,446 measured bytes after
warm-up, unchanged after six further passes (three full, three presence-only).
Those six passes allocated six temporary blocks totaling 90 bytes. This checks
buffer reuse for that fixture; it does not include caller-owned lookup memory.

On the same Linux kernel, repository test binaries also passed multi-batch and
six-level traversal, root replacement, delivery-time mutation, and churn-progress
cases on loop-mounted XFS and btrfs (five cases per filesystem). Separate isolated
mount-namespace tests passed for tmpfs, bind-mount exclusions, unreachable targets,
and overlayfs replay. These runs qualify the tested configurations, not every
kernel or mount option.
