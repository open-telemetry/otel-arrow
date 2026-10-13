------------------------------ MODULE Coverage ------------------------------
(***************************************************************************)
(* Vacuity check for BatchAggregation: does the model actually reach the  *)
(* risky paths that the properties are about?                             *)
(*                                                                         *)
(* Each run checks one path, named by the constant Target. The invariant  *)
(* TargetUnreached is EXPECTED TO FAIL: the violation is a trace that     *)
(* reaches the path. If TLC finds no error, the path is unreachable and   *)
(* the passing results of BatchAggregation may be vacuous for it.         *)
(*                                                                         *)
(* coverage.sh runs every path in every sizing mode.                      *)
(***************************************************************************)
EXTENDS BatchAggregation

CONSTANT Target

VARIABLES
    splitEver,  \* requests that ever had two or more references
    retEver,    \* requests that ever had a residual retained
    fragEver,   \* requests that ever had a split fragment retained
    sent        \* units that were ever sent downstream

covVars == <<vars, splitEver, retEver, fragEver, sent>>

Labels ==
    {"residual_retained", "split_request", "split_request_acked",
     "residual_request_acked", "outbound_full_nack", "inbound_full_nack",
     "batch_error", "downstream_nack", "tracked_pass_through"}
    \cup (IF Mode = "bytes"
          THEN {"fragment", "fragment_retained", "fragment_residual_acked"}
          ELSE {})

ASSUME Target \in Labels

(* The path named l is reached in the current state. *)
Reached(l) ==
    CASE l = "residual_retained" ->
             \E i \in DOMAIN buf : buf[i].ret
      [] l = "split_request" ->
             \E r \in live : cnt[r] >= 2
      [] l = "split_request_acked" ->
             \E r \in splitEver : nAck[r] = 1
      [] l = "residual_request_acked" ->
             \E r \in retEver : nAck[r] = 1
      [] l = "outbound_full_nack" ->
             /\ Cardinality(DOMAIN outs) = cfg.outl
             /\ \E r \in trk :
                    /\ nNack[r] > 0
                    /\ \E b \in inflight : b.kind = "batch" /\ b.key = NoOut
                                           /\ b.units \cap UnitsOf(r) # {}
      [] l = "inbound_full_nack" ->
             \E r \in trk : nNack[r] > 0
                            /\ UnitsOf(r) \cap (sent \cup lost) = {}
      [] l = "batch_error" ->
             \E u \in lost : u \notin sent
      [] l = "downstream_nack" ->
             lost \cap sent # {}
      [] l = "tracked_pass_through" ->
             \E b \in inflight : b.pt # NoKey
      [] l = "fragment" ->
             \E b \in inflight : <<0, 1>> \in b.units
      [] l = "fragment_retained" ->
             \E i \in DOMAIN buf : buf[i].ret /\ buf[i].frag
      [] l = "fragment_residual_acked" ->
             \E r \in fragEver : nAck[r] = 1

CovInit ==
    /\ Init
    /\ splitEver = {}
    /\ retEver = {}
    /\ fragEver = {}
    /\ sent = {}

CovNext ==
    /\ Next
    /\ splitEver' = splitEver \cup {r \in live' : cnt'[r] >= 2}
    /\ retEver' = retEver \cup {buf'[i].inkey : i \in {j \in DOMAIN buf' :
                                                        buf'[j].ret}}
    /\ fragEver' = fragEver \cup {buf'[i].inkey :
                                  i \in {j \in DOMAIN buf' :
                                          buf'[j].ret /\ buf'[j].frag}}
    /\ sent' = sent \cup UNION {b.units : b \in inflight'}

CovSpec == CovInit /\ [][CovNext]_covVars

(* Expected to be violated. See the module header. *)
TargetUnreached == ~Reached(Target)

=============================================================================
