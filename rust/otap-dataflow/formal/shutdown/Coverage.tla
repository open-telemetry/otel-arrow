------------------------------ MODULE Coverage ------------------------------
(***************************************************************************)
(* Vacuity check for Shutdown: does the model actually reach the paths    *)
(* that the properties and leads are about?                               *)
(*                                                                         *)
(* Each run checks one path, named by the constant CovTarget. The        *)
(* invariant TargetUnreached is EXPECTED TO FAIL: the violation is a     *)
(* trace that reaches the path. If TLC finds no error, the path is       *)
(* unreachable and the passing results of Shutdown may be vacuous for it. *)
(*                                                                         *)
(* coverage.sh runs every path that applies to each topology.             *)
(***************************************************************************)
EXTENDS Shutdown

CONSTANT CovTarget

Labels ==
    {"real_ack", "real_nack", "forced_nack", "shut_nack", "timeout_nack",
     "deadline_fired", "all_stopped", "mgr_pending_sends", "disp_pending_sends",
     "burst_path", "retained_converted", "closed_before_shutdown",
     "wakeup_after_latch", "router_parked_at_latch", "tick_during_drain",
     "receiver_forced_stop", "fanout_inflight_at_shutdown",
     "holding_input_after_latch"}

ASSUME CovTarget \in Labels

HasOutcome(v, src) ==
    \E r \in Requests : \E i \in DOMAIN outs[r] :
        outs[r][i].v = v /\ outs[r][i].src = src

(* The path named l is reached in the current state. *)
Reached(l) ==
    CASE l = "real_ack" -> HasOutcome("ack", "exp")
      [] l = "real_nack" -> HasOutcome("nack", "exp")
      [] l = "forced_nack" -> HasOutcome("nack", "forced")
      [] l = "shut_nack" -> HasOutcome("nack", "shut")
      [] l = "timeout_nack" -> HasOutcome("nack", "proc")
      [] l = "deadline_fired" -> Dl
      [] l = "all_stopped" -> AllStopped /\ disp.exited
      [] l = "mgr_pending_sends" -> mgr.pend # <<>>
      [] l = "disp_pending_sends" -> disp.pend # <<>>
         \* the fairness burst path is armed while draining (L2c)
      [] l = "burst_path" -> ns["P"].latched /\ ns["P"].cc >= BurstLimit
         \* begin_shutdown turned a retained payload into a due ResumeData
      [] l = "retained_converted" ->
             ns["P"].sch /\ \E i \in DOMAIN ns["P"].due : ns["P"].due[i].k = "resume"
         \* the scheduler latched because the input closed first (L2b fix)
      [] l = "closed_before_shutdown" ->
             ns["P"].sch /\ ~ns["P"].latched /\ ~ns["P"].ib
         \* a pending wakeup survived the latch as a due item (L7 fix)
      [] l = "wakeup_after_latch" ->
             ns["P"].sch /\ \E i \in DOMAIN ns["P"].due : ns["P"].due[i].k = "wakeup"
      [] l = "router_parked_at_latch" -> ps.parked # 0 /\ ns["P"].sch
         \* a TimerTick reaches P while receivers still drain (L3 fix)
      [] l = "tick_during_drain" ->
             mgr.phase = "drainIngress"
             /\ \E i \in DOMAIN ctrl["P"] : ctrl["P"][i].k = "tick"
         \* the receiver stopped at the deadline without ReceiverDrained
      [] l = "receiver_forced_stop" -> Dl /\ Stopped("R") /\ ~rv.drained
         \* fanout gets Shutdown with requests still waiting (L4)
      [] l = "fanout_inflight_at_shutdown" ->
             DOMAIN ps.inflight # {}
             /\ \E i \in DOMAIN ctrl["P"] : ctrl["P"][i].k = "shut"
         \* batch-like holding processor takes input into an empty buffer
         \* after the latch, when the timer cannot be armed (L9)
      [] l = "holding_input_after_latch" ->
             Kind = "holding" /\ ns["P"].latched /\ data["P"] # <<>>
             /\ ps.buf = {}

TargetUnreached == ~Reached(CovTarget)

(* Safety-only spec: coverage needs reachability, not fairness. *)
CovSpec == Init /\ [][Next]_vars

=============================================================================
