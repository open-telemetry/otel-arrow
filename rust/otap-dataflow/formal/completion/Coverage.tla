------------------------------ MODULE Coverage ------------------------------
(***************************************************************************)
(* Vacuity check for Completion: does the model actually reach the paths  *)
(* that the properties are about?                                         *)
(*                                                                         *)
(* Each run checks one path, named by the constant Target. The invariant  *)
(* TargetUnreached is EXPECTED TO FAIL: the violation is a trace that     *)
(* reaches the path. If TLC finds no error, the path is unreachable and   *)
(* the passing results of Completion may be vacuous for it.               *)
(*                                                                         *)
(* coverage.sh runs every path that applies to each topology.             *)
(***************************************************************************)
EXTENDS Completion

CONSTANT Target

VARIABLES
    expiredEver,   \* a delivery reached the receiver after its slot was gone
    retriedEver,   \* requests whose data retry ever parked for a resend
    detachedEver,  \* an exporter answered a copy that carries no frames
    ignoredEver    \* fanout `primary` consumed a secondary outcome

covVars == <<vars, expiredEver, retriedEver, detachedEver, ignoredEver>>

Labels ==
    {"ack_answered", "nack_answered", "send_blocked", "pending_used",
     "expired_delivery", "abandoned", "error_answer", "split_partial",
     "fanout_all_partial", "secondary_ignored", "retry_parked",
     "retried_then_acked", "detached_copy_answered", "sink_ack"}

ASSUME Target \in Labels

(* The path named l is reached in the current state. *)
Reached(l) ==
    CASE l = "ack_answered" ->
             \E r \in Requests : answer[r] = "ack"
      [] l = "nack_answered" ->
             \E r \in Requests : answer[r] = "nack"
      [] l = "send_blocked" ->
             \/ \E n \in Nodes : todo[n] # <<>>
             \/ \E r \in Requests : rtodo[r] # <<>>
      [] l = "pending_used" ->
             pending # <<>>
      [] l = "expired_delivery" ->
             expiredEver
      [] l = "abandoned" ->
             abandoned # {}
      [] l = "error_answer" ->
             \E r \in Requests : answer[r] = "err"
      [] l = "split_partial" ->
             \E k \in DOMAIN splitIn : splitIn[k].left = 1
      [] l = "fanout_all_partial" ->
             \E k \in DOMAIN inflight : "acked" \in Range(inflight[k].st)
      [] l = "secondary_ignored" ->
             ignoredEver
      [] l = "retry_parked" ->
             parked # <<>>
      [] l = "retried_then_acked" ->
             \E r \in retriedEver : answer[r] = "ack"
      [] l = "detached_copy_answered" ->
             detachedEver
      [] l = "sink_ack" ->
             cfg.ports = 0 /\ \E r \in Requests : answer[r] = "ack"

CovInit ==
    /\ Init
    /\ expiredEver = FALSE
    /\ retriedEver = {}
    /\ detachedEver = FALSE
    /\ ignoredEver = FALSE

(* Only ReceiverHandleOutcome removes from ctrl["R"], only Export removes *)
(* from an exporter's data channel, and only F_ProcessControl removes    *)
(* from ctrl["F"].                                                        *)
CovNext ==
    /\ Next
    /\ expiredEver' =
         \/ expiredEver
         \/ /\ ctrl["R"] # <<>>
            /\ ctrl'["R"] = Tail(ctrl["R"])
            /\ Head(ctrl["R"]).cd[1] \notin waiting
    /\ retriedEver' =
         retriedEver \cup UNION {parked'[i].lin : i \in DOMAIN parked'}
    /\ detachedEver' =
         \/ detachedEver
         \/ \E e \in Exporters :
              /\ data[e] # <<>>
              /\ data'[e] = Tail(data[e])
              /\ Head(data[e]).stack = <<>>
    /\ ignoredEver' =
         \/ ignoredEver
         \/ /\ "F" \in Procs
            /\ cfg.fmode = "primary"
            /\ ctrl["F"] # <<>>
            /\ ctrl'["F"] = Tail(ctrl["F"])
            /\ Head(ctrl["F"]).cd[2] = 2

CovSpec == CovInit /\ [][CovNext]_covVars

(* Expected to be violated. See the module header. *)
TargetUnreached == ~Reached(Target)

=============================================================================
