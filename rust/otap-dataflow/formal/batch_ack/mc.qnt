// -*- mode: Bluespec; -*-
// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Model-checking instances of batch_ack. Run them with check.sh.

/// Small bounds, all fixes on. Expected: every invariant holds.
module small {
  import batch_ack(
    MAX_INPUTS = 3, INBOUND_CAP = 2, OUTBOUND_CAP = 2,
    FIX_SHUTDOWN_WAKEUP = true, EARLY_WAKEUPS = false,
    MUTATION = "none", COVERAGE = false,
  ).* from "batch_ack"
}

/// Larger bounds, all fixes on. Expected: every invariant holds.
module large {
  import batch_ack(
    MAX_INPUTS = 4, INBOUND_CAP = 3, OUTBOUND_CAP = 3,
    FIX_SHUTDOWN_WAKEUP = true, EARLY_WAKEUPS = false,
    MUTATION = "none", COVERAGE = false,
  ).* from "batch_ack"
}

/// main as it is (F1/F2 fix off). Expected: no_crash fails.
module nofix {
  import batch_ack(
    MAX_INPUTS = 3, INBOUND_CAP = 2, OUTBOUND_CAP = 2,
    FIX_SHUTDOWN_WAKEUP = false, EARLY_WAKEUPS = false,
    MUTATION = "none", COVERAGE = true,
  ).* from "batch_ack"
}

/// F4 / D9: the live wakeup may fire early. Expected: no_stuck_buffer fails.
module early {
  import batch_ack(
    MAX_INPUTS = 3, INBOUND_CAP = 2, OUTBOUND_CAP = 2,
    FIX_SHUTDOWN_WAKEUP = true, EARLY_WAKEUPS = true,
    MUTATION = "none", COVERAGE = false,
  ).* from "batch_ack"
}

/// Mutation: release pending references before adding output references.
/// Expected: safety fails.
module mut_release {
  import batch_ack(
    MAX_INPUTS = 3, INBOUND_CAP = 2, OUTBOUND_CAP = 2,
    FIX_SHUTDOWN_WAKEUP = true, EARLY_WAKEUPS = false,
    MUTATION = "release_before_refs", COVERAGE = false,
  ).* from "batch_ack"
}

/// Mutation: a retained tail loses its delivery owner.
/// Expected: safety fails.
module mut_tail {
  import batch_ack(
    MAX_INPUTS = 3, INBOUND_CAP = 2, OUTBOUND_CAP = 2,
    FIX_SHUTDOWN_WAKEUP = true, EARLY_WAKEUPS = false,
    MUTATION = "tail_untracked", COVERAGE = false,
  ).* from "batch_ack"
}

/// Coverage: fixes on, events recorded. Expected: every witness except the
/// crash ones is reached.
module cov {
  import batch_ack(
    MAX_INPUTS = 3, INBOUND_CAP = 2, OUTBOUND_CAP = 2,
    FIX_SHUTDOWN_WAKEUP = true, EARLY_WAKEUPS = false,
    MUTATION = "none", COVERAGE = true,
  ).* from "batch_ack"
}

/// Wider bounds for random simulation only (too big for TLC).
/// Expected: every invariant holds.
module wide {
  import batch_ack(
    MAX_INPUTS = 6, INBOUND_CAP = 3, OUTBOUND_CAP = 2,
    FIX_SHUTDOWN_WAKEUP = true, EARLY_WAKEUPS = false,
    MUTATION = "none", COVERAGE = false,
  ).* from "batch_ack"
}
