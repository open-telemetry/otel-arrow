# mpsc channel sender-wakeup fix: correctness and performance tradeoffs

## The bug

When multiple senders are blocked on a full `mpsc::Channel`, the receiver's
`try_recv` only wakes one sender on the exact transition from
`buffer.len() == capacity` to `buffer.len() == capacity - 1`. If the receiver
drains past that threshold before the woken sender runs, subsequent pops do not
wake any additional blocked senders. Those senders remain blocked indefinitely.

This is a correctness bug that can cause permanent deadlock in fan-in topologies
where multiple upstream nodes send to a single downstream channel.

## The fix

Replace the conditional wake:

```rust
if state.buffer.len() == state.capacity - 1 {
    state.wake_one_sender_waiter();
}
```

with an unconditional wake on every successful pop:

```rust
state.wake_one_sender_waiter();
```

`wake_one_sender_waiter` is a no-op when no senders are waiting, so the
uncontended path (no blocked senders) has negligible overhead.

## Performance impact

### What is a spurious wakeup?

A spurious wakeup occurs when a blocked sender is woken by the receiver, gets
scheduled by the tokio executor, polls its `SendFuture`, calls `send()`, finds
the buffer still full, re-registers as a waiter, and returns `Pending`. The
sender did a full poll cycle for nothing -- it was woken but could not make
progress.

With the fix, every pop calls `wake_one_sender_waiter`. If the receiver pops N
items before yielding to the executor, it wakes N senders. But those senders
have not run yet (single-threaded executor, the receiver is still executing).
When the receiver finally yields, the woken senders run. The first sender to run
may fill multiple buffer slots in its own tight loop before blocking. That means
subsequent senders find the buffer full when they run -- their wakeup was
spurious.

For example, with 8 blocked senders and a receiver that drains 4 items before
yielding: 4 senders are woken, but only 1 sender may refill all 4 slots. The
other 3 senders each do a full poll cycle (executor schedules the task, polls the
future, borrows channel state, attempts send, gets Full, re-registers waiter,
returns Pending) for nothing. This overhead is spread across the tokio scheduler
(task state transitions, queue operations, atomic CAS, driver polls) with no
single function dominating -- profiling shows the extra time is distributed
across hundreds of functions in the scheduler loop, each individually small.

### Benchmark results

Measured on Apple Silicon with `release-debug` profile. Note: the two tables
below use different total message counts -- the tight-loop benchmark uses
1,048,576 messages while the consumer-yield benchmark uses 32,768 messages
(reduced because the per-message cost is much higher with a yield on every
recv).

#### Tight loop (no yields, worst case)

1,048,576 total messages. Producers send as fast as possible, consumer drains
without yielding. This is a pathological micro-benchmark -- not representative
of real workloads.

| Capacity | Producers | Buggy (ms) | Fixed (ms) | Overhead |
|----------|-----------|------------|------------|----------|
| 1024     | 1         | 5.17       | 5.23       | +1%      |
| 1024     | 2         | 5.21       | 5.39       | +4%      |
| 1024     | 4         | 5.13       | 5.69       | +11%     |
| 1024     | 8         | 5.15       | 6.05       | +18%     |
| 256      | 1         | 6.51       | 6.67       | +2%      |
| 256      | 2         | 6.56       | 7.12       | +9%      |
| 256      | 4         | 6.60       | 8.08       | +22%     |
| 256      | 8         | 6.51       | 10.10      | +55%     |
| 64       | 1         | 12.24      | 12.59      | +3%      |
| 64       | 2         | 12.24      | 14.41      | +18%     |
| 64       | 4         | 12.31      | 18.29      | +49%     |
| 64       | 8         | 12.22      | 25.86      | +112%    |

Overhead scales with producer count (more blocked senders to spuriously wake)
and inversely with capacity (more drain cycles per unit of work).

#### Consumer yields every recv (realistic slow-consumer scenario)

32,768 total messages. Producers send as fast as possible, consumer yields after
every recv (simulating a processor that does work between reads). The channel
stays at capacity and senders block, but the consumer never drains more than one
item before yielding.

| Capacity | Producers | Buggy (ms) | Fixed (ms) | Overhead |
|----------|-----------|------------|------------|----------|
| 1024     | 1         | 15.39      | 15.38      | 0%       |
| 1024     | 8         | 15.39      | 15.38      | 0%       |
| 256      | 1         | 15.54      | 15.63      | +1%      |
| 256      | 8         | 15.58      | 15.54      | 0%       |
| 64       | 1         | 15.56      | 15.67      | +1%      |
| 64       | 8         | 15.45      | 15.66      | +1%      |

Zero measurable overhead regardless of capacity or producer count.

## When the overhead appears

The overhead requires a specific combination of conditions:

- **Fan-in topology**: multiple upstream senders to one channel
- **CPU-bound receiver**: drains multiple items from the channel without
  yielding to the executor between pops
- **CPU-bound senders**: refill the buffer in tight loops without yielding

This corresponds to a fan-in bottleneck processor that is CPU-bound, never
yields, and is fed by senders that also never yield.

## Fairness regression

The buggy code's FIFO waiter queue provided perfect round-robin fairness among
blocked senders in the tight-loop scenario. Because only one sender was woken
per drain cycle, each sender got an equal turn.

The fix wakes multiple senders per drain cycle. The tokio executor decides which
of the woken tasks to poll first, and the first sender to run fills the buffer
while the rest bounce. This breaks the strict round-robin rotation:

```text
Buggy code -- per-sender progress at 25%/50%/75%/100% of messages:
  checkpoint 1: [25%, 25%, 25%, 25%, 25%, 25%, 25%, 25%]
  checkpoint 2: [50%, 50%, 50%, 50%, 50%, 50%, 50%, 50%]
  checkpoint 3: [75%, 75%, 75%, 75%, 75%, 75%, 75%, 75%]

Fixed code -- same scenario:
  checkpoint 1: [100%, 0%, 0%, 0%, 100%, 0%, 0%, 0%]
  checkpoint 2: [100%, 66%, 0%, 66%, 100%, 0%, 66%, 0%]
  checkpoint 3: [100%, 100%, 0%, 100%, 100%, 100%, 100%, 0%]
```

All senders eventually complete, but some finish all their messages before
others start. This unfairness only manifests in the same tight-loop scenario
that causes the spurious wakeup overhead. When the consumer yields between
recvs (the realistic case), only one sender is woken per pop and fairness is
preserved.

## The same bug exists in `mpmc.rs`

The `mpmc::Channel` implementation has the identical wakeup condition and
requires the same fix.
