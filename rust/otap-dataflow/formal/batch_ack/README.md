# Batch processor Ack/Nack model

Executable Quint model of the batch processor's Ack/Nack aggregation, part 1
of [#4395](https://github.com/open-telemetry/otel-arrow/issues/4395).
[spec.md](spec.md) states the invariants, the findings and the open
decisions; this file explains how to run the checks.

## Files

| File | Content |
| --- | --- |
| `spec.md` | Plain-language invariants traced to the Rust functions, findings, intended behaviors, decisions, inventory |
| `batch_ack.qnt` | The model: state, actions, invariants, coverage witnesses |
| `mc.qnt` | Instances with concrete bounds, toggles and mutations |
| `batch_ack_test.qnt` | Short deterministic `run` scenarios, suitable for CI |
| `check.sh` | Runs every check below and fails on an unexpected result |

## Requirements

- Quint 0.33 or later: `npm i -g @informalsystems/quint`. The first run
  downloads the Rust evaluator and Apalache into `~/.quint`. Tested with
  Quint 0.33.1 and Apalache 0.62.4.
- Java 17 or later for `quint verify`. TLC (`--backend=tlc`) is the
  exhaustive backend; Apalache does not accept this model (see
  [Results](#results)).

## Running

```sh
./check.sh            # scenarios, TLC on the small bounds, toggles, coverage
./check.sh --large    # also 100k-trace simulations on larger bounds
```

Single checks:

```sh
quint test batch_ack_test.qnt --main=batch_ack_test
quint verify mc.qnt --main=small --invariant=all_invariants --backend=tlc
quint run mc.qnt --main=cov --invariant=all_invariants --max-samples=5000 \
  --max-steps=25 --witnesses w_tail_retained w_batch_error
```

To re-run a simulation on another seed, space the seeds further apart than
`--max-samples`; nearby seeds explore almost the same traces.

## Model in one paragraph

One `SignalBuffer` is modeled. Each step is one `process()` call: an input
that stays buffered or triggers a size flush, a timer wakeup, a downstream
Ack or Nack, the inbox latching `Shutdown` (drain), or the final
`Shutdown`. The batcher returns any range list that `validate_input_ranges`
accepts, encoded as one token per drained input (completed, keep open,
close, split, split and close), or an error. A retained tail, a
`finish_batch` failure and the size threshold are nondeterministic. Ghost
state records every upstream answer and every output that carried an
input's data, which is what the invariants check. See
[spec.md](spec.md#model-abstraction) for the full mapping.

## Instances and toggles

| Instance | Bounds (inputs, inbound, outbound) | Toggle | Expected |
| --- | --- | --- | --- |
| `small` | 3, 2, 2 | fix on | all invariants hold |
| `large` | 4, 3, 3 | fix on | all invariants hold |
| `nofix` | 3, 2, 2 | `FIX_SHUTDOWN_WAKEUP = false` (main) | `no_crash` fails (F1, F2) |
| `early` | 3, 2, 2 | `EARLY_WAKEUPS = true` (D9) | `no_stuck_buffer` fails (F4) |
| `mut_release` | 3, 2, 2 | mutation: release pending references before adding output references | `safety` fails |
| `mut_tail` | 3, 2, 2 | mutation: a retained tail loses its delivery owner | `tail_attribution` fails |
| `cov` | 3, 2, 2 | coverage events on | every witness reached |
| `early_cov` | 3, 2, 2 | `EARLY_WAKEUPS` and coverage on | `w_early_wakeup` reached |
| `wide` | 6, 3, 2 | fix on, simulation only | all invariants hold |

`FIX_SHUTDOWN_WAKEUP` mirrors the fix to `SignalBuffer::set_arrival` in
`crates/core-nodes/src/processors/batch_processor/mod.rs`: treat
`WakeupError::ShuttingDown` as success with no armed wakeup. Turning it off
brings back both counterexamples, as do the Rust tests named in
[spec.md](spec.md#fix-for-f1-and-f2) when the fix is reverted.

## Invariants

| Name | Spec | Meaning |
| --- | --- | --- |
| `answered_at_most_once` | S1 | at most one upstream answer per input, only for tracked inputs |
| `ack_only_after_all_outputs_acked` | S2 | an Ack means every output that carried the input was Acked and nothing of it is buffered |
| `nack_answers_at_once` | S2 | a live input has no Nacked or dropped output |
| `ref_conservation` | S3 | refcount = pending entries + outbound slots naming the input |
| `live_means_unanswered` | S1 | a live inbound slot belongs to an unanswered input |
| `tail_attribution` | S4 | pending entries name distinct inputs and keep their input's tracking; a tracked one is live or already Nacked |
| `slot_bounds` | S5 | slot capacities are respected |
| `no_crash` | S6 | `process()` never returns `Err` |
| `no_leak_when_settled` | L1 | nothing buffered or in flight before `Shutdown` means every tracked input is answered and no slot is held |
| `no_stuck_buffer` | L2 | while running, buffered data always has a live wakeup |

`safety` is S1 to S5, `liveness` is L1 and L2, `all_invariants` is both plus
`no_crash`. L1 and L2 are checked in state form: together with S3 they
imply that every input is answered once downstream answers every output.

## Results

Quint 0.33.1, TLC through `quint verify --backend=tlc`, macOS arm64 with
10 cores, on `main` at `108a4ff87`.

| Check | Engine | Result | States | Time |
| --- | --- | --- | --- | --- |
| scenarios (9 runs) | `quint test` | pass | - | under 1 s |
| `small`, all invariants | TLC, exhaustive | pass | 1.9M distinct, depth 17 | 54 s |
| `large`, all invariants | TLC, exhaustive | no violation, stopped unfinished | 38.7M distinct after 29 min, queue still growing | stopped |
| `large`, all invariants | simulation, 100k traces of 40 steps | pass | - | 315 s |
| `wide`, all invariants | simulation, 100k traces of 40 steps | pass | - | 589 s |
| `nofix`, `no_crash` | TLC | violation (F1) | 3-state trace | 7 s |
| `nofix`, `no_crash_in_flush` | TLC | violation (F2) | 3-state trace | 7 s |
| `early`, `no_stuck_buffer` | TLC | violation (F4) | 3-state trace | 7 s |
| `mut_release`, `safety` | TLC | violation | 3-state trace | 7 s |
| `mut_tail`, `tail_attribution` | TLC | violation | 3-state trace | 9 s |
| coverage, 30 witnesses (`cov`, `nofix`, `early_cov`) | simulation, 5k traces of 25 steps, seed 1 | every witness reached (1% to 100% of traces) | - | 29 s |

`./check.sh` takes about 2 minutes. The shortest counterexamples are:

- F1: `begin_drain`, then one input that stays buffered.
- F2: `begin_drain`, then one input split into two outputs whose remainder
  is retained as a tail.
- F4: one buffered input, then its wakeup fires early.

To reproduce the exhaustive `large` run, which takes hours:

```sh
quint verify mc.qnt --main=large --invariant=all_invariants --backend=tlc
```

Apalache (`quint verify` without `--backend`) does not accept this model: it
refuses to expand the set of batcher choices ("Trying to expand a set of
functions"). The model favors TLC's explicit enumeration of batcher
results; making it Apalache-friendly would need fixed-size choice domains,
which multiplies TLC's branching.

## Limitations

- At most one split boundary per input, and `finish_batch` fails on at most
  one of the first two outputs of a flush.
- Passthrough (`forward_as_is`) and the inline rejects in
  `process_signal_impl` take no slot and are not modeled (D6).
- Engine-level failures of `send_message` and `notify_*` are not modeled
  (F3, D5).
- Time is abstracted: a wakeup fires at its deadline unless `EARLY_WAKEUPS`
  is on.
- Answers for outputs of the shutdown flush are out of scope (D3).
