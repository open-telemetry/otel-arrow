# OTAP pdata-optimization — low-batch repeat benchmark

Boxing `RawBatchStore.batches` (`[Option<RecordBatch>; COUNT]` -> `Box<[...]>`) to shrink `OtapArrowRecords`/`OtapPayload` stack size. **baseline** = main (inline, ~760 B), **optimized** = boxed (16 B).

- Workload: OTAP, no compression, passthrough (receiver -> exporter)
- Config: low batch (`loadgen_max_batch_size: 64`), single engine core
- Rate: 400k signals/s offered | Observation: 60s | Repeats: 3 (plus one discarded warm-up)
- Generated: 2026-05-22

## metrics / low-batch (400k, 60s)

| Image | Run | Delivered/s | Drop % | CPU avg % | CPU max % | RAM max MiB |
|---|---|--:|--:|--:|--:|--:|
| baseline | 1 | 226,128 :warning: ramp/under-delivered | -2.48 | 39.5 | 67.0 | 55 |
| baseline | 2 | 401,388 | -1.69 | 67.0 | 67.5 | 53 |
| baseline | 3 | 402,147 | -1.69 | 65.9 | 66.3 | 56 |
| optimized | 1 | 402,007 | -1.26 | 64.8 | 65.2 | 53 |
| optimized | 2 | 401,679 | -1.69 | 65.9 | 66.4 | 53 |
| optimized | 3 | 402,343 | -1.69 | 66.4 | 70.1 | 49 |

**Averages (valid runs only):** CPU baseline 66.4% -> optimized 65.7% (**-1.1%**); RAM baseline 55 -> optimized 52 MiB (-4.9%).

## logs / low-batch (400k, 60s)

| Image | Run | Delivered/s | Drop % | CPU avg % | CPU max % | RAM max MiB |
|---|---|--:|--:|--:|--:|--:|
| baseline | 1 | 402,470 | -1.70 | 49.6 | 49.8 | 51 |
| baseline | 2 | 403,538 | -1.69 | 49.5 | 49.7 | 53 |
| baseline | 3 | 402,423 | -1.69 | 49.0 | 49.3 | 49 |
| optimized | 1 | 401,373 | -1.26 | 48.1 | 48.4 | 55 |
| optimized | 2 | 403,573 | -1.70 | 48.1 | 48.3 | 53 |
| optimized | 3 | 404,358 | -1.69 | 47.6 | 48.0 | 49 |

**Averages (valid runs only):** CPU baseline 49.4% -> optimized 47.9% (**-2.9%**); RAM baseline 51 -> optimized 53 MiB (+2.3%).

## Notes

- metrics/baseline run1 delivered only ~226k of 400k (loadgen ramp inside the window); excluded from the average as an invalid sample.
- logs triplets are tight (+-0.3 CPU), so the -2.9% is repeatable; metrics is ~-1% on valid runs.
- RAM max shows no meaningful difference at this batch size/rate.
- Takeaway: small but consistent CPU win (~1-3%) on low-batch OTAP passthrough; no RAM benefit here.

