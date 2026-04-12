# Architectural Assessment: Memory Limiter Rework Proposal

## Executive Summary

The Codex proposal attempts to address legitimate concerns about cross-thread coordination in the current memory limiter implementation, but **introduces significant complexity and subtle correctness issues that outweigh the benefits**. While the core problem identification is sound, the proposed solution is over-engineered and creates new failure modes.

**Recommendation**: Pursue a simpler alternative that achieves the same architectural goals with fewer moving parts and clearer semantics.

---

## 1. Correctness Analysis

### 1.1 Race Conditions and Ordering

**PROBLEM: Watch Channel Semantics Create Missed Transitions**

The `tokio::sync::watch` channel semantics are problematic here:
- `watch::Receiver::changed()` only signals when the *most recent* value changes
- If pressure transitions `Normal → Soft → Hard` rapidly between two `changed()` checks, the receiver only sees the latest value (`Hard`)
- The intermediate `Soft` transition is **lost**

This is particularly dangerous during oscillating pressure scenarios:
```
Timeline:
T0: Level = Normal, generation = 0
T1: Sampler updates: Soft, generation = 1  (watch publishes)
T2: Sampler updates: Hard, generation = 2  (watch publishes, overwrites)
T3: Receiver calls changed() → sees Hard, generation = 2
     ⚠️  The Soft transition at generation = 1 was never observed
```

**Impact**: If future logic depends on observing all transitions (e.g., for state machine invariants, metrics, or gradual admission throttling), the watch-based approach silently violates that contract.

**Current Implementation Comparison**: 
The atomic shared state doesn't have this problem—every `should_shed_ingress()` call sees the current level. There's no "missed update" concept because there's no update buffering.

### 1.2 Generation Counter Staleness Detection

**PROBLEM: Generation Counter Provides Weak Protection**

The proposal includes a generation counter in `ReceiverAdmissionState` to detect stale updates:
```rust
pub fn apply(&mut self, update: MemoryPressureChanged) -> bool {
    if update.generation <= self.generation {
        return false; // Stale or duplicate, ignore
    }
    // ... apply update
}
```

This guards against **duplicate or reordered** updates, but:
1. **It doesn't prevent missed transitions** (as discussed above)
2. **It requires careful initialization** — if a receiver constructs local state from process-wide state at `T0` but the first watch update arrives reflecting state at `T2`, the generation at `T0` is unknown to the receiver
3. **Generation overflow**: A `u64` counter is practically safe, but the proposal doesn't specify the type or overflow behavior

**Better Alternative**: If you need transition tracking, use a bounded MPSC queue of transitions instead of watch, accepting the memory cost for correctness.

### 1.3 Initialization Race

**PROBLEM: Bootstrap State May Be Stale**

The proposal suggests:
> 5. **Initialization**: Bootstrap local state from current process-wide state at construction time.

Consider this sequence:
1. Receiver constructs, calls `ReceiverAdmissionState::from_process_state()`
   - Reads global state: `level = Normal, generation = 100`
2. Memory limiter ticks, transitions to `Hard`, publishes `generation = 101`
3. Receiver finishes construction, starts main loop
4. First `changed()` fires immediately because value changed since receiver creation
5. Receiver applies update: `generation = 101, level = Hard` ✓

This **works correctly** in the happy path. However:

**Edge Case**: If the receiver is constructed during a tick in progress:
1. Receiver reads global state **mid-tick**: `level = Normal, usage = <partially updated>`
2. Tick completes, writes `level = Soft`, increments generation
3. Watch publishes
4. Receiver sees generation jump but may have based initialization on inconsistent snapshot

**Mitigation**: The `MemoryPressureState` atomic fields are independently consistent (each is a single atomic), but there's no cross-field snapshot guarantee. In practice, this is likely acceptable because:
- Worst case: Receiver starts with slightly stale `level` but corrects on first update
- The shared state is read-only for receivers, so no write-write conflict

**Verdict**: Minor issue, probably acceptable with clear documentation.

### 1.4 NodeControlMsg Delivery Ordering

**PROBLEM: Control Message Delivery Is Best-Effort**

The `RuntimeCtrlMsgManager::send()` method used for control message delivery:
```rust
// From pipeline_ctrl.rs, reconstructed logic:
self.send(node_id, NodeControlMsg::MemoryPressureChanged { update })
```

This ultimately calls `control_sender.try_send()` on a bounded channel. If the receiver's control channel is full, the message is:
- Buffered in `pending_sends: VecDeque<(usize, NodeControlMsg<PData>)>`
- Retried later with a 5ms delay timer

**Implications**:
1. **Delivery is NOT guaranteed in real-time** — a receiver under heavy ingress load may not process control messages promptly
2. **Ordering is preserved** (thanks to VecDeque FIFO), but delivery latency is unbounded
3. **If a receiver never drains its control channel**, updates never apply

**Current Implementation Comparison**:
The atomic approach has **zero delivery latency** — every `should_shed_ingress()` call sees current state immediately, regardless of receiver control channel backlog.

**Is This Acceptable?**
For memory pressure transitions, delayed delivery could be problematic:
- `Normal → Hard` transition: Receiver continues accepting ingress until control message is processed
- Could admit work while process is already at hard limit

However, this is mitigated by:
- Receivers are **required** to prioritize control messages in their select! loops
- Control channels are typically lightly loaded (only timers, config, shutdown, now pressure updates)
- Pressure transitions are relatively rare compared to ingress rate

**Verdict**: Likely acceptable for Phase 1, but introduces a new latency vector that didn't exist before.

---

## 2. Alignment with Maintainer Feedback

### 2.1 Thread-Per-Core / Share-Nothing Alignment

**GOAL**: Eliminate shared mutable state on hot paths.

**Assessment**: ✅ **Mostly Achieved**

The proposal successfully moves admission decisions from:
```rust
// BEFORE: Two atomic loads on every ingress request
fn should_shed_ingress(&self) -> bool {
    mode_from_u8(self.inner.mode.load(Ordering::Relaxed)) == MemoryLimiterMode::Enforce
        && self.level() == MemoryPressureLevel::Hard
}
```

To:
```rust
// AFTER: Pure local read
impl ReceiverAdmissionState {
    fn should_shed_ingress(&self) -> bool {
        self.mode == MemoryLimiterMode::Enforce && self.level == MemoryPressureLevel::Hard
    }
}
```

**However**:
- The `watch::Receiver<MemoryPressureChanged>` itself is **`Clone + Send + Sync`** and internally uses `Arc` + atomics for notification
- Each `changed()` call involves **atomic operations** on the watch channel's internal state
- So you're trading hot-path atomic reads for cold-path (control loop) atomic operations

**Key Insight**: The hot path (ingress admission) is now lock-free and cache-local ✓, but the control path still coordinates via atomics. This is the right trade-off.

### 2.2 NUMA-Aware Future Compatibility

**GOAL**: Prepare for hierarchical memory budgeting with local fast-path admission.

**Assessment**: ⚠️ **Partially Aligned, But Incomplete**

The proposal adds:
> Longer term: hierarchical memory budgeting with local fast-path admission.

The `ReceiverAdmissionState` is a **per-receiver** state, but the maintainer's vision likely includes:
- **Per-NUMA-node** memory budgets
- **Per-core** or **per-runtime** budgets
- **Hierarchical enforcement**: Process-wide → NUMA-node → Core → Receiver

The current proposal doesn't directly enable this because:
1. State is per-receiver, not per-runtime or per-core
2. No budget abstraction — only pressure level
3. No aggregation or quota management

**What Would Be Better**:
If hierarchical budgeting is a real Phase 2+ goal, introduce:
```rust
struct LocalMemoryBudget {
    allocated_bytes: u64,
    limit_bytes: u64,
    generation: u64,
}

enum AdmissionDecision {
    Accept,
    Reject { retry_after: Duration },
    Throttle { delay: Duration },
}
```

Then receivers consult a **local budget** instead of global pressure level. The control plane periodically redistributes budget based on actual usage.

**Verdict**: The proposal is a step in the right direction but doesn't architecturally enable hierarchical budgets. More design work needed.

### 2.3 Control Plane Propagation

**GOAL**: Propagate state transitions through the control plane.

**Assessment**: ✅ **Correctly Implemented**

The proposal uses:
- `watch::Receiver` in `RuntimeCtrlMsgManager`
- `NodeControlMsg::MemoryPressureChanged` fanout to receiver nodes
- Single-threaded `LocalSet` runtimes with `select!` loop integration

This is **architecturally correct** for this codebase's control plane model. It:
- Respects the existing `NodeControlMsg` abstraction
- Integrates with the existing `select!` loop in `pipeline_ctrl.rs`
- Maintains receiver-first priority (control messages are biased in select!)

**Minor Concern**: Adding another branch to the `select!` macro increases complexity and poll overhead, but this is negligible compared to timer/data/completion traffic.

---

## 3. Design Quality

### 3.1 Is Watch Channel the Right Mechanism?

**PROBLEM: Watch Channel Is Optimized for Different Use Case**

`tokio::sync::watch` is designed for:
- Broadcasting **latest value** to many receivers
- Receivers that only care about **most recent state**, not transitions

Memory pressure transitions are **not** purely state-based:
- Operators may want to count transitions (for alerting)
- Future logic may implement gradual throttling (Soft → reduce ingress by 50%, Hard → shed all)
- Metrics should track transition frequency, not just current level

**Better Alternatives**:

**Option A: Bounded MPSC for Transitions**
```rust
enum MemoryPressureEvent {
    Transitioned { from: Level, to: Level, generation: u64, usage_bytes: u64 },
}

// In RuntimeCtrlMsgManager
let (pressure_tx, pressure_rx) = mpsc::channel::<MemoryPressureEvent>(16);
```

**Pros**:
- Guaranteed delivery of all transitions (bounded by buffer)
- Explicit event semantics
- Easy to add transition metadata

**Cons**:
- Channel can fill if receivers lag → same latency issue as NodeControlMsg
- More memory per runtime

**Option B: Keep Atomic State, Add Notification Channel**
```rust
// For hot path: keep current approach
state.should_shed_ingress()  // Fast atomic read

// For observability/metrics: separate notification channel
enum MemoryPressureNotification {
    LevelChanged { from: Level, to: Level, at: Instant },
}
```

**Pros**:
- Hot path stays fast (unchanged)
- Transitions are observable but don't block hot path
- Clear separation of concerns

**Cons**:
- Doesn't address maintainer's concern about atomic coordination

### 3.2 ReceiverAdmissionState Abstraction

**PROBLEM: Redundant State Management**

The proposal adds:
```rust
struct ReceiverAdmissionState {
    generation: u64,
    level: MemoryPressureLevel,
    retry_after_secs: u32,
    mode: MemoryLimiterMode,
    usage_bytes: Option<u64>,  // "optional" usage tracking
}
```

**Issues**:
1. **State Duplication**: This mostly mirrors `MemoryPressureState` fields
2. **`usage_bytes: Option<u64>`**: Why optional? If receivers need it, make it required. If not, remove it.
3. **No Clear Ownership**: Who decides when `mode` changes? (Answer: It's set once at startup, so local copy is static)

**Simpler Alternative**:
```rust
struct ReceiverAdmissionState {
    generation: u64,
    should_shed: bool,  // Precomputed from mode + level
    retry_after_secs: u32,
}
```

Receivers don't need raw level/mode — they need admission decisions.

### 3.3 Fanout Logic Complexity

**PROBLEM: Fanout to All Receivers on Every Transition**

The proposal suggests:
```rust
for receiver_id in control_senders.receiver_ids() {
    self.send(receiver_id, NodeControlMsg::MemoryPressureChanged { update });
}
```

**Scalability Concern**:
- If a deployment has 100 pipeline runtimes with 5 receivers each = 500 receiver nodes
- Every pressure transition sends 500 control messages
- Each message is 16-32 bytes, so ~8-16 KB per transition
- At 1 transition/second → manageable, but burst transitions could spike

**Mitigation**: This is probably fine because:
- Pressure transitions are infrequent (sampling is typically 1-10 seconds)
- Control channels are bounded and buffered
- This is not on the hot data path

**Alternative**: Per-runtime state instead of per-receiver state (reduces fanout by ~5×).

---

## 4. Risks and Gaps

### 4.1 Missing Pieces

1. **No Admin/Metrics/Readiness Integration**
   - Proposal says "Keep `MemoryPressureState` for admin/metrics/readiness only"
   - But doesn't specify how receivers expose their **local** state for observability
   - Operators need per-receiver admission status for debugging

2. **No Rollback Plan**
   - If the new approach causes latency spikes or missed transitions, rolling back requires:
     - Removing `NodeControlMsg::MemoryPressureChanged` variant
     - Restoring atomic checks in all receiver hot paths
     - This is feasible but introduces merge conflict risk

3. **No Testing Strategy**
   - How do you test "receiver handles delayed control message delivery"?
   - Need chaos/fault injection tests: fill control channel, verify eventual consistency

### 4.2 Failure Modes

**Scenario 1: Receiver Starves Control Channel**
- Receiver is CPU-bound processing ingress
- Control channel fills with `MemoryPressureChanged` messages
- Receiver never drains control channel → never updates admission state
- Process hits OOM while receiver still admits traffic

**Mitigation**: 
- Receivers MUST use `biased; select!` with control first
- But Rust `select!` doesn't guarantee control-branch priority under sustained load
- Better: Add timeout/escape hatch in ingress loops

**Scenario 2: Watch Notification Lost During Restart**
- Receiver runtime restarts (e.g., live reconfig)
- New receiver bootstraps from global state at `T0`
- Transition happens at `T1`
- Watch notification missed because receiver wasn't subscribed yet

**Mitigation**:
- Bootstrap reads current state, so eventual consistency is guaranteed
- First `changed()` call will sync to latest

---

## 5. Complexity Cost vs. Benefit

### 5.1 Complexity Added

**Code Changes**:
1. New `MemoryPressureChanged` payload struct
2. New `NodeControlMsg::MemoryPressureChanged` variant
3. New `ReceiverAdmissionState` with 5 methods
4. Watch channel setup in `RuntimeCtrlMsgManager::new()`
5. New select! branch in `pipeline_ctrl` event loop
6. Rewire 4+ receivers (OTLP HTTP, OTLP gRPC, OTAP, Syslog TCP/UDP)
7. Update receiver tests to inject mock control messages

**Estimated LOC**: +400-600 lines, -100 lines = **+300-500 net lines**

**Conceptual Complexity**:
- Developers must understand: atomics → atomics + watch + control messages
- Debugging admission issues now requires tracing control message delivery
- New failure mode: "Why is receiver still admitting traffic?" → Check control channel backlog

### 5.2 Performance Benefit

**Before (Atomic Approach)**:
```rust
// Per ingress request (HTTP, gRPC, OTAP, Syslog):
fn should_shed_ingress(&self) -> bool {
    let mode = self.inner.mode.load(Ordering::Relaxed);  // ~1-2 cycles
    let level = self.inner.level.load(Ordering::Relaxed); // ~1-2 cycles
    mode_from_u8(mode) == MemoryLimiterMode::Enforce && level_from_u8(level) == Hard
}
```

**Cost**: ~2-4 CPU cycles, 2 cache line reads (if not cached)

**After (Local State Approach)**:
```rust
fn should_shed_ingress(&self) -> bool {
    self.should_shed  // Pure stack/register read, <1 cycle
}
```

**Benefit**: ~2-4 cycles per ingress request

**Is This Meaningful?**
- At 100,000 req/sec: Saves ~200-400K cycles/sec = 0.0001 CPU cores @ 3 GHz
- Negligible unless you're saturating the pipeline at millions of req/sec

**Cache Benefit**:
- Atomic reads may cause cache invalidation if multiple cores are checking
- Local state keeps admission logic in L1 cache

**Verdict**: The performance gain is **real but small**. Only worthwhile if you're already bottlenecked on memory limiter checks (unlikely).

---

## 6. Alternative Approaches

### 6.1 Hybrid: Atomic State + Local Cache

**Idea**: Keep the atomic shared state but add a **local cache** in receivers with TTL-based invalidation.

```rust
struct ReceiverAdmissionCache {
    cached_decision: bool,
    cached_at: Instant,
    ttl: Duration,  // e.g., 100ms
    state: MemoryPressureState,  // Arc<AtomicU8>
}

impl ReceiverAdmissionCache {
    fn should_shed_ingress(&mut self, now: Instant) -> bool {
        if now - self.cached_at > self.ttl {
            self.cached_decision = self.state.should_shed_ingress();
            self.cached_at = now;
        }
        self.cached_decision
    }
}
```

**Pros**:
- Reduces atomic reads from every-request to every-100ms
- No control message delivery latency
- No missed transitions
- Simpler: ~50 LOC, no watch channel, no control message variant

**Cons**:
- Still has atomics (but 1000× less frequent)
- Introduces TTL tuning parameter
- Receivers can lag up to `ttl` behind true state

**Verdict**: This is **significantly simpler** and achieves 99% of the benefit.

### 6.2 Per-Runtime State (Not Per-Receiver)

**Idea**: Instead of per-receiver state, maintain **one admission state per `RuntimeCtrlMsgManager`**.

```rust
struct RuntimeAdmissionState {
    should_shed: bool,
    retry_after_secs: u32,
}

// In RuntimeCtrlMsgManager
let admission_state = Rc<RefCell<RuntimeAdmissionState>>;

// Pass Rc<RefCell<...>> to each receiver in this runtime
```

**Pros**:
- Fanout is 1 update per runtime instead of N updates per runtime
- Aligns with future per-core budgeting
- Simpler than per-receiver state

**Cons**:
- `RefCell` borrow checking at runtime (but negligible cost)
- All receivers in a runtime share state (but they're on same core anyway)

**Verdict**: This is **architecturally cleaner** and scales better.

### 6.3 Keep Current Implementation + Add Transition Events

**Idea**: Don't change hot path, but add optional **transition event channel** for observability.

```rust
// Hot path: unchanged
state.should_shed_ingress()

// Observability: sampler emits events
enum MemoryPressureEvent {
    Transitioned { from: Level, to: Level, at: Instant },
}

// Receivers can subscribe if they want transition tracking
```

**Pros**:
- Zero risk to hot path
- Addresses observability concern
- Trivial to implement (~100 LOC)

**Cons**:
- Doesn't address maintainer's concern about shared atomics

**Verdict**: If the maintainer is okay with atomics remaining, this is the **lowest-risk option**.

---

## 7. Recommendations

### 7.1 What I Would Do Instead

**Short-Term (Phase 1)**:
Implement **Alternative 6.2: Per-Runtime State** with these changes:

1. **Add `RuntimeAdmissionState` to `RuntimeCtrlMsgManager`**:
   ```rust
   struct RuntimeAdmissionState {
       should_shed: bool,
       retry_after_secs: u32,
       generation: u64,
   }
   ```

2. **Keep watch channel for control-plane propagation** (it's fine for this use case)

3. **Pass `Rc<RefCell<RuntimeAdmissionState>>` to all receivers in the runtime**:
   - Each receiver holds an `Rc` clone
   - On `MemoryPressureChanged`, `RuntimeCtrlMsgManager` updates the `RefCell` once
   - Fanout is eliminated (1 update instead of N)

4. **Receivers check local state on hot path**:
   ```rust
   let admission = self.admission_state.borrow();
   if admission.should_shed { /* reject */ }
   ```

**Benefits**:
- ✅ Eliminates per-receiver fanout
- ✅ Hot path is local read (cache-friendly)
- ✅ Aligns with per-core budgeting future
- ✅ Simpler than per-receiver state

**Drawbacks**:
- `RefCell` borrow adds ~5 cycles (but still faster than atomics)
- Shared state within runtime (but they're on same core, so cache-coherent)

### 7.2 If You Proceed with Original Proposal

**Critical Changes**:

1. **Replace `watch` with bounded MPSC** to guarantee transition delivery:
   ```rust
   let (tx, mut rx) = mpsc::channel::<MemoryPressureChanged>(16);
   ```

2. **Add staleness timeout** in receiver hot paths:
   ```rust
   if now - self.admission_state.last_update_at > Duration::from_secs(5) {
       // Control message delivery lagging → fail safe
       return true; // Shed ingress
   }
   ```

3. **Add per-receiver observability**:
   - Expose `ReceiverAdmissionState` via admin API
   - Add metric: `memory_pressure_update_lag_seconds`

4. **Document failure modes** in CONTRIBUTING.md:
   - What happens if control channel fills?
   - How to debug "receiver still admits during Hard pressure"?

### 7.3 Long-Term Architecture

If the goal is truly **hierarchical memory budgeting**, this proposal is a stepping stone at best. You'd ultimately want:

```rust
struct HierarchicalMemoryBudget {
    process: ProcessBudget,        // Global cap
    numa_nodes: Vec<NumaBudget>,   // Per-NUMA budget
    cores: Vec<CoreBudget>,        // Per-core budget
    receivers: Vec<ReceiverQuota>, // Per-receiver quota
}

impl ReceiverQuota {
    fn try_reserve(&mut self, bytes: u64) -> Result<Reservation, QuotaExceeded>;
}
```

The current proposal doesn't enable this. If this is the real Phase 2+ goal, **design that system first** and work backwards to Phase 1.

---

## 8. Final Verdict

| Criterion | Score | Notes |
|-----------|-------|-------|
| **Correctness** | ⚠️ 6/10 | Watch channel loses transitions; control message delivery latency |
| **Alignment** | ✅ 7/10 | Achieves local hot-path, but doesn't enable hierarchical budgeting |
| **Design Quality** | ⚠️ 5/10 | Over-engineered; watch channel is wrong tool; fanout is inefficient |
| **Risk** | ⚠️ 6/10 | New failure modes (missed transitions, control channel lag) |
| **Complexity** | ❌ 4/10 | +300-500 LOC for ~2-4 cycle gain per request |

**Overall Assessment**: ⚠️ **Do Not Proceed As-Is**

The proposal identifies a real architectural concern (shared atomics on hot path) but solves it with excessive complexity and introduces subtle correctness issues.

**Recommended Path Forward**:
1. Implement **Alternative 6.2 (Per-Runtime State)** as Phase 1
2. If hierarchical budgeting is a real goal, design it explicitly before proceeding
3. If not, consider **Alternative 6.1 (Atomic + Cache)** as the simplest correct solution

The current proposal is not wrong, but it's not the best solution to this problem.
