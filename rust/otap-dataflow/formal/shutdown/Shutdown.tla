----------------------------- MODULE Shutdown -----------------------------
(***************************************************************************)
(* Graceful shutdown and drain of one OTAP dataflow pipeline.             *)
(*                                                                         *)
(* The plain-language properties are in spec.md. Each action and each    *)
(* state-transforming operator names the Rust function it mirrors. Paths *)
(* are relative to rust/otap-dataflow/crates/ and line numbers refer to  *)
(* commit 298846b57.                                                      *)
(*                                                                         *)
(* Nodes: "R" receiver, "P" the one processor of the topology, "A" "B"   *)
(* exporters (B only in T4). A pdata message is the id of the request it *)
(* carries. Frames are not modeled (Part 2): an outcome is routed to the *)
(* nearest subscriber, see Target.                                       *)
(*                                                                         *)
(* Time is abstracted. The deadline is one nondeterministic event        *)
(* (MgrDeadline). Delayed resumes and wakeups may fire at any step before *)
(* the shutdown latch (DelayedFire, WakeFire).                            *)
(*                                                                         *)
(* Node-level actions are written in a functional style: a handler takes *)
(* a record St of the node-visible state and returns the new record, and *)
(* Commit assigns it. This keeps UNCHANGED lists short and lets handlers  *)
(* compose the way the Rust code calls them.                              *)
(***************************************************************************)
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS
    Topo,               \* "T1" .. "T5", see spec.md "Topologies"
    NumRequests,        \* number of requests the receiver may admit
    Cap,                \* capacity of every pdata channel
    CtrlCap,            \* capacity of every node control channel
    BurstLimit,         \* stands for CONTROL_BURST_LIMIT (message.rs 20)
    MaxRetries,         \* T3: retry limit of the retry processor
    RecvModes,          \* subset of {"wait", "ff"}: wait_for_result or not
    RecvSkips,          \* may the receiver stop without ReceiverDrained?
    ExpSilents,         \* may an exporter drop a message with no outcome?
    ExpUnbounded,       \* does exporter shutdown wait for in-flight forever?
    Gates,              \* "none", "gated" (T3), "never", "router" (T5)
    FanoutTimeouts,     \* T4: is a destination timeout configured?
    FanoutMaxInflights, \* T4: max_inflight, 0 = unlimited
    Deadlines,          \* may the deadline fire in this behavior?
    GracefulPremise,    \* restrict no-deadline behaviors to S7's premise?
    Fixes               \* leads whose fix is applied, see spec.md "Fixes"

ASSUME Topo \in {"T1", "T2", "T3", "T4", "T5"}
ASSUME NumRequests \in Nat \ {0}
ASSUME {Cap, CtrlCap, BurstLimit} \subseteq Nat \ {0}
ASSUME MaxRetries \in Nat
ASSUME RecvModes \subseteq {"wait", "ff"} /\ RecvModes # {}
ASSUME RecvSkips \subseteq BOOLEAN /\ ExpSilents \subseteq BOOLEAN
ASSUME ExpUnbounded \subseteq BOOLEAN /\ FanoutTimeouts \subseteq BOOLEAN
ASSUME Gates \subseteq {"none", "gated", "never", "router"}
ASSUME FanoutMaxInflights \subseteq Nat
ASSUME Deadlines \subseteq BOOLEAN /\ GracefulPremise \in BOOLEAN
ASSUME Fixes \subseteq {"L2b", "L7", "L6", "L3", "L4"}

Fixed(l) == l \in Fixes

Requests == 1..NumRequests

-----------------------------------------------------------------------------
(* Topology *)

ExpSeq == IF Topo = "T4" THEN <<"A", "B">> ELSE <<"A">>
Exps == {ExpSeq[i] : i \in DOMAIN ExpSeq}
NonRecvSeq == <<"P">> \o ExpSeq          \* control_senders.non_receiver_ids
NonRecv == {"P"} \cup Exps
Nodes == {"R"} \cup NonRecv

Kind ==
    CASE Topo = "T1" -> "stateless"
      [] Topo = "T2" -> "holding"        \* local-wakeup flush, batch-like
      [] Topo = "T3" -> "retry"
      [] Topo = "T4" -> "fanout"
      [] Topo = "T5" -> "gate"

(* Upstream senders of each node's pdata channel. *)
Up(n) == IF n = "P" THEN {"R"} ELSE {"P"}

Configs ==
    [rmode : RecvModes, skip : RecvSkips, silent : ExpSilents,
     unb : ExpUnbounded, gate : Gates, fto : FanoutTimeouts,
     fmax : FanoutMaxInflights, dlOn : Deadlines]

(* spec.md S7: every exporter answers (or a configured timeout covers it), *)
(* no processor keeps admission closed forever, every receiver drains.   *)
Premise(c) ==
    /\ ~c.skip
    /\ ~c.unb
    /\ c.gate # "never"
    /\ (~c.silent \/ (Topo = "T4" /\ c.fto))

-----------------------------------------------------------------------------
(* Helpers *)

Put(f, k, v) == [x \in DOMAIN f \cup {k} |-> IF x = k THEN v ELSE f[x]]
Remove(f, k) == [x \in DOMAIN f \ {k} |-> f[x]]
Inc(x) == IF x < BurstLimit THEN x + 1 ELSE x
Min(S) == CHOOSE x \in S : \A y \in S : x <= y

(* Control messages. k: "drain" (DrainIngress), "shut" (Shutdown), "tick" *)
(* (TimerTick), "out" (Ack/Nack delivered by the dispatcher).            *)
Msg(k, r, v, src, from) == [k |-> k, r |-> r, v |-> v, src |-> src, from |-> from]
Ctl(k) == Msg(k, 0, "-", "-", "-")
Out(r, v, src, from) == Msg("out", r, v, src, from)

(* Local scheduler items: a ResumeData payload or a payload-free Wakeup. *)
Item(k, r) == [k |-> k, r |-> r]
IsResume(i) == i.k = "resume"

RECURSIVE ResumeSeq(_)
ResumeSeq(S) ==
    IF S = {} THEN <<>>
    ELSE <<Item("resume", Min(S))>> \o ResumeSeq(S \ {Min(S)})

RECURSIVE OutSeq(_)
OutSeq(S) == IF S = {} THEN <<>> ELSE <<<<"A", Min(S)>>>> \o OutSeq(S \ {Min(S)})

-----------------------------------------------------------------------------
(* State *)

VARIABLES
    cfg,    \* the configuration of this behavior
    mgr,    \* runtime-control manager: phase, timers, pending_sends, ...
    rtc,    \* runtime control channel into the manager
    disp,   \* completion dispatcher: channel, pending_sends, exited
    ns,     \* per node: lifecycle, inbox and local scheduler state
    data,   \* pdata channel of each node: seq of request ids
    ctrl,   \* control channel of each node: seq of Msg
    ps,     \* processor P private state
    ex,     \* exporter private state
    rv,     \* receiver private state
    outs,   \* ghost: outcomes each request's receiver slot received
    flags   \* ghost: labels of properties that failed

vars == <<cfg, mgr, rtc, disp, ns, data, ctrl, ps, ex, rv, outs, flags>>

NS0 == [st |-> "run",     \* "run", "drain" (R), "waitIn" (exporter), "stopped"
        latched |-> FALSE,  \* InboxCore.shutting_down_deadline is Some
        ib |-> FALSE,       \* InboxCore::shutdown ran: receivers dropped
        cc |-> 0,           \* InboxCore.consecutive_control, capped
        delayed |-> {},     \* scheduler delayed_resumes (request ids)
        due |-> <<>>,       \* scheduler items that are due (due_now / heap)
        wake |-> FALSE,     \* a wakeup is armed in the scheduler heap
        sch |-> FALSE,      \* scheduler shutting_down
        out |-> <<>>]       \* blocked send_message awaits: <<dest, r>>

Init ==
    /\ cfg \in {c \in Configs : c.dlOn \/ Premise(c) \/ ~GracefulPremise}
    /\ mgr = [phase |-> "running", timers |-> {}, pend |-> <<>>,
              dl |-> FALSE, pendR |-> FALSE]
    /\ rtc = <<>>
    /\ disp = [compl |-> <<>>, pend |-> <<>>, exited |-> FALSE]
    /\ ns = [n \in Nodes |-> NS0]
    /\ data = [n \in Nodes |-> <<>>]
    /\ ctrl = [n \in Nodes |-> <<>>]
    /\ ps = [buf |-> {}, att |-> [r \in Requests |-> 0],
             inflight |-> [r \in {} |-> {}], parked |-> 0, shut |-> FALSE,
             treq |-> FALSE]
    /\ ex = [e \in Exps |-> [held |-> {}, stuck |-> {}]]
    /\ rv = [admitted |-> {}, slots |-> {}, wait |-> {}, drained |-> FALSE]
    /\ outs = [r \in Requests |-> <<>>]
    /\ flags = {}

Dl == mgr.dl

(* Runtime timers run until the freeze: the shutdown request before the *)
(* L3 fix, downstream Shutdown after it (freeze_timers).                 *)
TimersLive ==
    IF Fixed("L3") THEN mgr.phase \in {"running", "drainIngress"}
    ELSE mgr.phase = "running"
Stopped(n) == ns[n].st = "stopped"
AllStopped == \A n \in Nodes : Stopped(n)
(* A node's control receiver is gone: the task stopped, or the inbox ran *)
(* InboxCore::shutdown (message.rs 314-322).                              *)
CtrlClosed(n) == Stopped(n) \/ ns[n].ib
(* pdata_rx.is_closed(): every upstream sender has been dropped. *)
InClosed(n) == \A u \in Up(n) : Stopped(u)
SchedDrained(n) ==                       \* is_drained (scheduler.rs 463)
    ns[n].delayed = {} /\ ns[n].due = <<>> /\ ~ns[n].wake

(* accept_pdata() of each node. *)
Accept(n) ==
    IF n \in Exps THEN TRUE
    ELSE CASE Kind = "retry" ->
                (cfg.gate = "gated" => ns[n].due = <<>> /\ ns[n].delayed = {})
           [] Kind = "fanout" ->     \* fanout 1052-1064
                cfg.fmax = 0 \/ Cardinality(DOMAIN ps.inflight) < cfg.fmax
           [] Kind = "gate" /\ cfg.gate = "router" ->
                ps.parked = 0         \* router_admission 161-166, 290-296
           [] Kind = "gate" -> ~ps.shut
           [] OTHER -> TRUE

(* Nearest subscriber of an outcome reported by `from` for request r.    *)
(* Retry and fanout subscribe on every message they send; the other      *)
(* processors forward without a frame of their own.                       *)
Target(from, r) ==
    IF from \in Exps /\ Kind \in {"retry", "fanout"} THEN "P"
    ELSE IF r \in rv.wait THEN "R" ELSE "none"

-----------------------------------------------------------------------------
(* Non-blocking control sends of the manager and the dispatcher. *)

(* RuntimeCtrlMsgManager::send (pipeline_ctrl.rs 915-937): try_send, park *)
(* on Full, drop on Closed. acc = [c |-> ctrl, p |-> pending_sends].      *)
RECURSIVE MSendAll(_, _, _)
MSendAll(ns_, m, acc) ==
    IF ns_ = <<>> THEN acc
    ELSE LET n == Head(ns_)
         IN MSendAll(Tail(ns_), m,
                IF CtrlClosed(n) THEN acc
                ELSE IF Len(acc.c[n]) < CtrlCap
                     THEN [acc EXCEPT !.c[n] = Append(@, m)]
                     ELSE [acc EXCEPT !.p = Append(@, <<n, m>>)])

(* RuntimeCtrlMsgManager::drain_pending_sends (941-961): one pass. *)
RECURSIVE MDrainR(_, _)
MDrainR(p, acc) ==
    IF p = <<>> THEN acc
    ELSE LET x == Head(p)
         IN MDrainR(Tail(p),
                IF CtrlClosed(x[1]) THEN acc
                ELSE IF Len(acc.c[x[1]]) < CtrlCap
                     THEN [acc EXCEPT !.c[x[1]] = Append(@, x[2])]
                     ELSE [acc EXCEPT !.p = Append(@, x)])
MDrain(c, p) == MDrainR(p, [c |-> c, p |-> <<>>])

(* PipelineCompletionMsgDispatcher::send / drain_pending_sends            *)
(* (1011-1070). A closed target drops the outcome with no signal (1039,   *)
(* 1064): that is an S8 violation, recorded in fl.                         *)
RECURSIVE DDrainR(_, _)
DDrainR(p, acc) ==
    IF p = <<>> THEN acc
    ELSE LET x == Head(p)
         IN DDrainR(Tail(p),
                IF CtrlClosed(x.to) THEN [acc EXCEPT !.fl = @ \cup {"S8"}]
                ELSE IF Len(acc.c[x.to]) < CtrlCap
                     THEN [acc EXCEPT !.c[x.to] = Append(@, x.m)]
                     ELSE [acc EXCEPT !.p = Append(@, x)])

-----------------------------------------------------------------------------
(* Node handlers, functional style. *)

St == [ns |-> ns, data |-> data, ctrl |-> ctrl, ps |-> ps, ex |-> ex,
       compl |-> disp.compl, rtc |-> rtc, flags |-> flags]

Commit(s) ==
    /\ ns' = s.ns /\ data' = s.data /\ ctrl' = s.ctrl /\ ps' = s.ps
    /\ ex' = s.ex /\ rtc' = s.rtc /\ flags' = s.flags
    /\ disp' = [disp EXCEPT !.compl = s.compl]
    /\ UNCHANGED <<cfg, mgr, rv, outs>>

(* notify_ack / notify_nack: put the outcome on the completion channel. *)
Report(s, from, r, v, src) ==
    LET t == Target(from, r)
    IN IF t = "none" THEN s   \* no frame: dropped by design
       ELSE [s EXCEPT !.compl = Append(@, [to |-> t, m |-> Out(r, v, src, from)])]

RECURSIVE ReportAll(_, _, _, _, _)
ReportAll(s, from, S, v, src) ==
    IF S = {} THEN s
    ELSE ReportAll(Report(s, from, Min(S), v, src), from, S \ {Min(S)}, v, src)

(* NodeLocalScheduler::begin_shutdown (scheduler.rs 441-460): delayed    *)
(* resumes become due now, later requeue_later and set_wakeup are        *)
(* rejected (257, 283). Before the L7 fix pending wakeups were dropped;  *)
(* with it each one is delivered once.                                    *)
KeepAtLatch(i) == IsResume(i) \/ Fixed("L7")

BeginShutdown(n_) ==
    [n_ EXCEPT !.due = SelectSeq(n_.due, KeepAtLatch) \o ResumeSeq(n_.delayed)
                       \o (IF Fixed("L7") /\ n_.wake
                           THEN <<Item("wakeup", 0)>> ELSE <<>>),
               !.delayed = {}, !.wake = FALSE, !.sch = TRUE]

(* EffectHandler::set_wakeup: rejected with ShuttingDown after the latch. *)
ArmWake(s) ==
    IF s.ns["P"].sch \/ s.ns["P"].wake THEN s
    ELSE [s EXCEPT !.ns["P"].wake = TRUE]

Fwd(s, r) == [s EXCEPT !.ns["P"].out = <<<<"A", r>>>>]

(* L7 fix, nack_unprobeable_parked: a parked message whose re-probe     *)
(* wakeup was refused (scheduler shutting down) is NACKed at once.       *)
RouterNackStranded(s) ==
    IF Fixed("L7") /\ s.ps.parked # 0 /\ s.ns["P"].sch
    THEN Report([s EXCEPT !.ps.parked = 0], "P", s.ps.parked, "nack", "shut")
    ELSE s

(* Holding processor flush (batch-like): send every buffered message. *)
Flush(s) == [s EXCEPT !.ns["P"].out = OutSeq(s.ps.buf), !.ps.buf = {}]

(* Exporter start() loop sees Shutdown: stop reading, settle in-flight. *)
ExpShut(s, e) == [s EXCEPT !.ns[e].st = "waitIn", !.ns[e].ib = TRUE]

(* process(PData) *)
PDataH(s, n, r, fate) ==
    IF n \in Exps
    THEN CASE fate = "normal" -> [s EXCEPT !.ex[n].held = @ \cup {r}]
           [] fate = "stuck" -> [s EXCEPT !.ex[n].stuck = @ \cup {r}]
           [] OTHER -> s         \* "drop": taken, never answered
    ELSE CASE Kind \in {"stateless", "retry"} -> Fwd(s, r)
           [] Kind = "gate" /\ cfg.gate # "router" -> Fwd(s, r)
           [] Kind = "gate" ->
                \* exclusive router, backpressure: try_send, park on Full
                \* and arm a wakeup (router_admission 169-206).
                IF Len(s.data["A"]) < Cap
                THEN [s EXCEPT !.data["A"] = Append(@, r)]
                ELSE RouterNackStranded(ArmWake([s EXCEPT !.ps.parked = r]))
           [] Kind = "holding" -> ArmWake([s EXCEPT !.ps.buf = @ \cup {r}])
           [] Kind = "fanout" ->
                \* fanout: subscribe and send to every destination; start
                \* the runtime timer on first need (ensure_timer, 477-481).
                LET s1 == [s EXCEPT !.ps.inflight = Put(@, r, Exps),
                                    !.ns["P"].out =
                                       [i \in DOMAIN ExpSeq |-> <<ExpSeq[i], r>>]]
                IN IF cfg.fto /\ ~s.ps.treq
                   THEN [s1 EXCEPT !.rtc = Append(@, [k |-> "st"]),
                                   !.ps.treq = TRUE]
                   ELSE s1

(* RetryProcessor handle_ack / handle_nack (retry 454-606). A rejected   *)
(* requeue_later Nacks upstream (595-603).                                *)
RetryOut(s, m) ==
    LET r == m.r
    IN IF m.v = "ack" THEN Report(s, "P", r, "ack", "exp")
       ELSE IF s.ps.att[r] < MaxRetries
            THEN IF s.ns["P"].sch
                 THEN Report(s, "P", r, "nack", "shut")
                 ELSE [s EXCEPT !.ns["P"].delayed = @ \cup {r},
                                !.ps.att[r] = @ + 1]
            ELSE Report(s, "P", r, "nack", "exp")

(* Fanout process_ack / process_nack, await_ack all. *)
FanoutOut(s, m) ==
    LET r == m.r
    IN IF r \notin DOMAIN s.ps.inflight THEN s
       ELSE IF m.v = "nack"
            THEN Report([s EXCEPT !.ps.inflight = Remove(@, r)],
                        "P", r, "nack", "exp")
            ELSE LET rem == s.ps.inflight[r] \ {m.from}
                 IN IF rem = {}
                    THEN Report([s EXCEPT !.ps.inflight = Remove(@, r)],
                                "P", r, "ack", "exp")
                    ELSE [s EXCEPT !.ps.inflight[r] = rem]

(* process(Control(Shutdown)) of P. Retry, stateless and the never-      *)
(* reopening gate ignore it (retry 705). Fanout ignored it before the L4 *)
(* fix (fanout 1107-1109); with it, nack_inflight_on_shutdown NACKs every *)
(* in-flight request upstream.                                            *)
OnShutP(s) ==
    CASE Kind = "fanout" /\ Fixed("L4") ->
           ReportAll([s EXCEPT !.ps.inflight = [r \in {} |-> {}]],
                     "P", DOMAIN s.ps.inflight, "nack", "shut")
      [] Kind = "holding" -> Flush(s)
      [] Kind = "gate" /\ cfg.gate = "router" /\ s.ps.parked # 0 ->
           \* handle_shutdown / drain_for_shutdown (router_admission 277-288)
           Report([s EXCEPT !.ps.parked = 0], "P", s.ps.parked, "nack", "shut")
      [] OTHER -> s

(* process(Control(msg)). c is the inflight entry a TimerTick finds     *)
(* expired (0 = none); time is abstracted.                                *)
CtrlH(s, n, m, c) ==
    IF n \in Exps
    THEN IF m.k = "shut" THEN ExpShut(s, n) ELSE s
    ELSE CASE m.k = "shut" -> OnShutP(s)
           [] m.k = "out" /\ Kind = "retry" -> RetryOut(s, m)
           [] m.k = "out" /\ Kind = "fanout" -> FanoutOut(s, m)
           [] m.k = "tick" /\ Kind = "fanout" /\ cfg.fto /\ c # 0 ->
                \* handle_timeout (fanout 667, 1090-1100)
                Report([s EXCEPT !.ps.inflight = Remove(@, c)],
                       "P", c, "nack", "proc")
           [] OTHER -> s

(* process(Control(ResumeData | Wakeup)) *)
DueH(s, item) ==
    IF item.k = "resume"
    THEN Fwd(s, item.r)                      \* retry handle_resumed (610)
    ELSE CASE Kind = "holding" -> Flush(s)
           [] Kind = "gate" /\ cfg.gate = "router" /\ s.ps.parked # 0 ->
                IF Len(s.data["A"]) < Cap
                THEN [s EXCEPT !.data["A"] = Append(@, s.ps.parked),
                               !.ps.parked = 0]
                ELSE RouterNackStranded(ArmWake(s)) \* repark_after_full (237)
           [] OTHER -> s

(* InboxCore::shutdown (message.rs 314-322) followed by the node's       *)
(* Shutdown handler. S3: the drain contract (365-379) must hold unless   *)
(* the deadline fired. Outcomes still queued in the dropped control      *)
(* channel are lost silently (S8).                                        *)
Release(s, n) ==
    LET b == BeginShutdown(s.ns[n])
        complete == InClosed(n) /\ s.data[n] = <<>> /\ b.due = <<>>
        lostOut == \E i \in DOMAIN s.ctrl[n] : s.ctrl[n][i].k = "out"
        s1 == [s EXCEPT !.ns[n] = [b EXCEPT !.ib = TRUE, !.latched = FALSE,
                                            !.cc = 0],
                        !.ctrl[n] = <<>>, !.data[n] = <<>>,
                        !.flags = @ \cup (IF ~complete /\ ~Dl
                                          THEN {"S3"} ELSE {})
                                    \cup (IF lostOut THEN {"S8"} ELSE {})]
    IN IF n \in Exps THEN ExpShut(s1, n) ELSE OnShutP(s1)

-----------------------------------------------------------------------------
(* InboxCore::recv_with_policy (message.rs 396-748): which message the  *)
(* next recv returns, or "block". The order of the tests is the order of *)
(* the code. Processors use HonorAdmission, exporters                     *)
(* ForceDrainDuringShutdown (361-363).                                    *)

RecvResult(n) ==
    LET s == ns[n]
        acc == Accept(n)
        dacc == acc \/ n \in Exps
        closed == InClosed(n)
        empty == data[n] = <<>>
        burst == s.cc >= BurstLimit
        \* A closed input may release only once retained work is back
        \* (closed_pdata_shutdown_when_drained, L2b fix). Before the fix
        \* every closed-input path released at once.
        mayRelease == ~Fixed("L2b") \/ BeginShutdown(s).due = <<>>
        latchSched == Fixed("L2b") /\ ~s.sch
        drainedRel == ~Fixed("L2b") \/ SchedDrained(n)
    IN IF ~acc /\ empty /\ closed /\ mayRelease THEN "release" \* 413-426
       ELSE IF ~acc /\ empty /\ closed /\ latchSched THEN "schedShut"
       ELSE IF s.latched
       THEN IF closed /\ empty /\ SchedDrained(n) THEN "release" \* 437-444
            ELSE IF dacc /\ burst /\ ~empty THEN "pdata"    \* 457-464
            ELSE IF dacc /\ burst /\ closed /\ drainedRel
                 THEN "release"                             \* 465-471
            ELSE IF ctrl[n] # <<>> THEN "ctrl"              \* 477-493
            ELSE IF s.due # <<>> THEN "due"                 \* 495-497
            ELSE IF Dl THEN "release"                       \* 506, 550
            ELSE IF dacc /\ ~empty THEN "pdata"             \* 514, 563
            ELSE IF dacc /\ closed /\ drainedRel
                 THEN "release"                             \* 516, 565
            ELSE "block"
       ELSE IF acc /\ burst /\ ~empty THEN "pdata"          \* 597-604
            ELSE IF acc /\ burst /\ closed /\ mayRelease
                 THEN "release"                             \* 605
            ELSE IF acc /\ burst /\ closed /\ latchSched THEN "schedShut"
            ELSE IF ctrl[n] # <<>>
                 THEN IF Head(ctrl[n]).k = "shut"
                      THEN IF Dl THEN "release_ctrl"        \* 623-629
                           ELSE "latch"                     \* 630-635
                      ELSE "ctrl"                           \* 637
            ELSE IF s.due # <<>> THEN "due"                 \* 643-645
            ELSE IF acc /\ ~empty THEN "pdata"              \* 651, 722
            ELSE IF acc /\ closed /\ mayRelease
                 THEN "release"                             \* 654, 725
            ELSE IF acc /\ closed /\ latchSched THEN "schedShut"
            ELSE "block"

Fates(n) ==
    IF n \in Exps
    THEN {"normal"} \cup (IF cfg.silent THEN {"drop"} ELSE {})
                    \cup (IF cfg.unb THEN {"stuck"} ELSE {})
    ELSE {"normal"}

(* One iteration of the processor loop (processor.rs 657-678) or of an  *)
(* exporter's start() loop: recv, then process.                          *)
Recv(n) ==
    /\ n \in NonRecv
    /\ ns[n].st = "run" /\ ~ns[n].ib /\ ns[n].out = <<>>
    /\ LET res == RecvResult(n)
       IN CASE res = "pdata" ->
                 \E f \in Fates(n) :
                   Commit(PDataH([St EXCEPT !.data[n] = Tail(@),
                                            !.ns[n].cc = 0],
                                 n, Head(data[n]), f))
            [] res = "ctrl" ->
                 \E c \in {0} \cup DOMAIN ps.inflight :
                   Commit(CtrlH([St EXCEPT !.ctrl[n] = Tail(@),
                                           !.ns[n].cc = Inc(@)],
                                n, Head(ctrl[n]), c))
            [] res = "due" ->
                 Commit(DueH([St EXCEPT !.ns[n].due = Tail(@),
                                        !.ns[n].cc = Inc(@)],
                             Head(ns[n].due)))
            [] res = "latch" ->
                 Commit([St EXCEPT !.ctrl[n] = Tail(@),
                                   !.ns[n] = [BeginShutdown(@) EXCEPT
                                                 !.latched = TRUE]])
            [] res = "release" -> Commit(Release(St, n))
            [] res = "schedShut" ->
                 \* closed input, retained work left: begin_shutdown only
                 Commit([St EXCEPT !.ns[n] = BeginShutdown(@)])
            [] res = "release_ctrl" ->
                 Commit(Release([St EXCEPT !.ctrl[n] = Tail(@)], n))
            [] OTHER -> FALSE

(* A blocked send_message().await completes, or fails on a closed      *)
(* channel (only reachable after the deadline).                          *)
Continue(n) ==
    /\ n \in NonRecv /\ ns[n].st = "run" /\ ns[n].out # <<>>
    /\ LET h == Head(ns[n].out)
           d == h[1]
       IN \/ /\ CtrlClosed(d)
             /\ ns' = [ns EXCEPT ![n].out = Tail(@)]
             /\ UNCHANGED data
          \/ /\ ~CtrlClosed(d) /\ Len(data[d]) < Cap
             /\ data' = [data EXCEPT ![d] = Append(@, h[2])]
             /\ ns' = [ns EXCEPT ![n].out = Tail(@)]
    /\ UNCHANGED <<cfg, mgr, rtc, disp, ctrl, ps, ex, rv, outs, flags>>

(* The next recv returns Closed (message.rs 404-406) and the processor   *)
(* loop ends (processor.rs 657). Anything still retained is dropped: S4. *)
NodeExit(n) ==
    /\ n = "P" /\ ns["P"].st = "run" /\ ns["P"].ib /\ ns["P"].out = <<>>
    /\ ns' = [ns EXCEPT !["P"].st = "stopped"]
    /\ flags' = flags \cup
         (IF ~Dl /\ (ns["P"].due # <<>> \/ ns["P"].delayed # {}
                     \/ ps.buf # {} \/ ps.parked # 0)
          THEN {"S4"} ELSE {})
    /\ UNCHANGED <<cfg, mgr, rtc, disp, data, ctrl, ps, ex, rv, outs>>

(* forced_shutdown_signal.triggered() (processor.rs 810-814). With the   *)
(* L6 fix exporters are cancelled too, after NODE_FORCED_SHUTDOWN_GRACE  *)
(* (start_until_forced_shutdown); the grace is abstracted: the node may *)
(* still finish on its own first. Work in flight is abandoned.           *)
ForcedStop(n) ==
    /\ Dl /\ ns[n].st # "stopped"
    /\ \/ n = "P"
       \/ n \in Exps /\ Fixed("L6")
    /\ ns' = [ns EXCEPT ![n].st = "stopped", ![n].out = <<>>, ![n].ib = TRUE]
    /\ ex' = IF n \in Exps THEN [ex EXCEPT ![n] = [held |-> {}, stuck |-> {}]]
             ELSE ex
    /\ UNCHANGED <<cfg, mgr, rtc, disp, data, ctrl, ps, rv, outs, flags>>

(* Exporter shutdown: wait for in-flight answers. A bounded exporter     *)
(* gives up at the deadline and Nacks the rest; an unbounded one         *)
(* (otlp_grpc 431-443, otlp_http 428-447, otap 707-738, azure_monitor    *)
(* 400-410) does not (L6).                                                *)
ExpStop(e) ==
    /\ e \in Exps /\ ns[e].st = "waitIn"
    /\ \/ ex[e].held = {} /\ ex[e].stuck = {}
       \/ Dl /\ ~cfg.unb
    /\ Commit(LET s == ReportAll(St, e, ex[e].held \cup ex[e].stuck,
                                 "nack", "shut")
              IN [s EXCEPT !.ns[e].st = "stopped",
                           !.ex[e] = [held |-> {}, stuck |-> {}]])

(* The exporter's remote answers one in-flight export. *)
Answer(e, r) ==
    /\ ns[e].st \in {"run", "waitIn"} /\ r \in ex[e].held
    /\ \E v \in {"ack", "nack"} :
         Commit(Report([St EXCEPT !.ex[e].held = @ \ {r}], e, r, v, "exp"))

NodeStep(n) ==
    Recv(n) \/ Continue(n) \/ NodeExit(n) \/ ForcedStop(n) \/ ExpStop(n)

(* Time passes in the local scheduler of P (pop_due, scheduler.rs 365). *)
DelayedFire ==
    /\ ~ns["P"].sch
    /\ \E r \in ns["P"].delayed :
         ns' = [ns EXCEPT !["P"].delayed = @ \ {r},
                          !["P"].due = Append(@, Item("resume", r))]
    /\ UNCHANGED <<cfg, mgr, rtc, disp, data, ctrl, ps, ex, rv, outs, flags>>

WakeFire ==
    /\ ns["P"].wake
    /\ ns' = [ns EXCEPT !["P"].wake = FALSE,
                        !["P"].due = Append(@, Item("wakeup", 0))]
    /\ UNCHANGED <<cfg, mgr, rtc, disp, data, ctrl, ps, ex, rv, outs, flags>>

TimeStep == DelayedFire \/ WakeFire

(* T5 "never": the processor closes admission for good (L5). *)
GateCloses ==
    /\ Kind = "gate" /\ cfg.gate = "never" /\ ~ps.shut
    /\ ps' = [ps EXCEPT !.shut = TRUE]
    /\ UNCHANGED <<cfg, mgr, rtc, disp, ns, data, ctrl, ex, rv, outs, flags>>

-----------------------------------------------------------------------------
(* Receiver, OTLP-style drain contract (otlp_receiver 736-800, 847-855). *)

ForceNack ==                     \* AckRegistry::force_shutdown (common.rs 74)
    [r \in Requests |->
        IF r \in rv.slots
        THEN Append(outs[r], [v |-> "nack", src |-> "forced"])
        ELSE outs[r]]

(* Admit one request and send it to P. Requests are admitted in order. *)
Admit(r) ==
    /\ ns["R"].st = "run" /\ ~CtrlClosed("P")
    /\ r = Cardinality(rv.admitted) + 1 /\ r \in Requests
    /\ Len(data["P"]) < Cap
    /\ data' = [data EXCEPT !["P"] = Append(@, r)]
    /\ rv' = [rv EXCEPT !.admitted = @ \cup {r},
                        !.slots = IF cfg.rmode = "wait" THEN @ \cup {r} ELSE @,
                        !.wait = IF cfg.rmode = "wait" THEN @ \cup {r} ELSE @]
    /\ UNCHANGED <<cfg, mgr, rtc, disp, ns, ctrl, ps, ex, outs, flags>>

(* Receiver control handling: DrainIngress (779-792), Ack/Nack routed to *)
(* a slot (route_ack_response, common.rs 89-119; expired keys change     *)
(* nothing), Shutdown (793-800).                                          *)
RCtrl ==
    /\ ns["R"].st \in {"run", "drain"} /\ ctrl["R"] # <<>>
    /\ ctrl' = [ctrl EXCEPT !["R"] = Tail(@)]
    /\ LET m == Head(ctrl["R"])
       IN CASE m.k = "drain" ->
                 /\ ns' = [ns EXCEPT !["R"].st = "drain"]
                 /\ UNCHANGED <<rv, outs>>
            [] m.k = "out" /\ m.r \in rv.slots ->
                 /\ outs' = [outs EXCEPT ![m.r] =
                               Append(@, [v |-> m.v, src |-> m.src])]
                 /\ rv' = [rv EXCEPT !.slots = @ \ {m.r}]
                 /\ UNCHANGED ns
            [] m.k = "shut" ->
                 /\ outs' = ForceNack
                 /\ rv' = [rv EXCEPT !.slots = {}]
                 /\ ns' = [ns EXCEPT !["R"].st = "stopped"]
            [] OTHER -> UNCHANGED <<ns, rv, outs>>
    /\ UNCHANGED <<cfg, mgr, rtc, disp, data, ps, ex, flags>>

(* notify_receiver_drained (effect_handler.rs 480) once the registry is *)
(* empty (754-755), then the receiver returns and drops its senders.     *)
RDrained ==
    /\ ns["R"].st = "drain" /\ rv.slots = {}
    /\ rtc' = Append(rtc, [k |-> "rd"])
    /\ rv' = [rv EXCEPT !.drained = TRUE]
    /\ ns' = [ns EXCEPT !["R"].st = "stopped"]
    /\ UNCHANGED <<cfg, mgr, disp, data, ctrl, ps, ex, outs, flags>>

(* An error path returns before notify_receiver_drained (spec.md D2). *)
RSkip ==
    /\ cfg.skip /\ ns["R"].st = "drain" /\ rv.slots = {}
    /\ ns' = [ns EXCEPT !["R"].st = "stopped"]
    /\ UNCHANGED <<cfg, mgr, rtc, disp, data, ctrl, ps, ex, rv, outs, flags>>

(* Drain deadline: force-Nack every open slot (745-750, 852). *)
RDeadline ==
    /\ ns["R"].st = "drain" /\ Dl /\ rv.slots # {}
    /\ outs' = ForceNack
    /\ rv' = [rv EXCEPT !.slots = {}]
    /\ UNCHANGED <<cfg, mgr, rtc, disp, ns, data, ctrl, ps, ex, flags>>

(* L6 fix: the receiver task is cancelled after the node grace, whether  *)
(* or not it ever saw DrainIngress or Shutdown (also covers L8). Open     *)
(* slots are dropped; their clients see an error, modeled as forced.     *)
RForcedStop ==
    /\ Fixed("L6") /\ Dl /\ ns["R"].st # "stopped"
    /\ outs' = ForceNack
    /\ rv' = [rv EXCEPT !.slots = {}]
    /\ ns' = [ns EXCEPT !["R"].st = "stopped"]
    /\ UNCHANGED <<cfg, mgr, rtc, disp, data, ctrl, ps, ex, flags>>

RecvStep == RCtrl \/ RDrained \/ RSkip \/ RDeadline \/ RForcedStop

-----------------------------------------------------------------------------
(* RuntimeCtrlMsgManager::run (pipeline_ctrl.rs 429). *)

(* RuntimeControlMsg::Shutdown (532-600). *)
MgrRequest ==
    /\ mgr.phase = "running"
    /\ LET a == MSendAll(<<"R">>, Ctl("drain"), [c |-> ctrl, p |-> mgr.pend])
       IN /\ ctrl' = a.c
          /\ mgr' = [mgr EXCEPT !.phase = "drainIngress",
                                !.timers = IF Fixed("L3") THEN @ ELSE {},
                                !.pend = a.p, !.pendR = TRUE]
    /\ UNCHANGED <<cfg, rtc, disp, ns, data, ps, ex, rv, outs, flags>>

(* ReceiverDrained (601-646) and StartTimer (647-660). *)
MgrRtc ==
    /\ mgr.phase # "exited" /\ rtc # <<>>
    /\ rtc' = Tail(rtc)
    /\ LET m == Head(rtc)
       IN IF m.k = "rd"
          THEN IF mgr.phase = "drainIngress"
               THEN LET a == MSendAll(NonRecvSeq, Ctl("shut"),
                                      [c |-> ctrl, p |-> mgr.pend])
                    IN /\ ctrl' = a.c
                       /\ mgr' = [mgr EXCEPT !.phase = "drainDown",
                                             !.timers = {},  \* freeze_timers
                                             !.pend = a.p, !.pendR = FALSE]
                       /\ flags' = flags \cup
                            (IF ~rv.drained /\ ~Dl THEN {"S1"} ELSE {})
               ELSE UNCHANGED <<ctrl, mgr, flags>>
          ELSE /\ mgr' = IF TimersLive
                         THEN [mgr EXCEPT !.timers = @ \cup {"P"}]
                         ELSE mgr                    \* ignored_start_timer
               /\ UNCHANGED <<ctrl, flags>>
    /\ UNCHANGED <<cfg, disp, ns, data, ps, ex, rv, outs>>

(* drain_pending_sends at the top of the loop (448). Enabled only when  *)
(* it makes progress, so a stuck state is a TLC deadlock.                 *)
MgrPending ==
    /\ mgr.phase # "exited" /\ mgr.pend # <<>>
    /\ LET a == MDrain(ctrl, mgr.pend)
       IN /\ a.p # mgr.pend
          /\ ctrl' = a.c
          /\ mgr' = [mgr EXCEPT !.pend = a.p]
    /\ UNCHANGED <<cfg, rtc, disp, ns, data, ps, ex, rv, outs, flags>>

(* A due runtime timer fires (handle_due_events). One tick in flight at *)
(* a time. S2: never once timers are frozen (TimersLive).               *)
MgrTick ==
    /\ mgr.phase # "exited" /\ "P" \in mgr.timers /\ ~CtrlClosed("P")
    /\ ~\E i \in DOMAIN ctrl["P"] : ctrl["P"][i].k = "tick"
    /\ ~\E i \in DOMAIN mgr.pend : mgr.pend[i][2].k = "tick"
    /\ LET a == MSendAll(<<"P">>, Ctl("tick"), [c |-> ctrl, p |-> mgr.pend])
       IN /\ ctrl' = a.c
          /\ mgr' = [mgr EXCEPT !.pend = a.p]
    /\ flags' = flags \cup (IF ~TimersLive THEN {"S2"} ELSE {})
    /\ UNCHANGED <<cfg, rtc, disp, ns, data, ps, ex, rv, outs>>

(* Deadline branch (468-504): drain pending once, send Shutdown to every *)
(* non-receiver and every pending receiver, trigger the forced signal,   *)
(* and exit. Sends still pending at exit are lost.                        *)
MgrDeadline ==
    /\ cfg.dlOn /\ ~mgr.dl /\ mgr.phase \in {"drainIngress", "drainDown"}
    /\ LET a0 == MDrain(ctrl, mgr.pend)
           a1 == MSendAll(NonRecvSeq, Ctl("shut"), a0)
           a2 == IF mgr.pendR THEN MSendAll(<<"R">>, Ctl("shut"), a1) ELSE a1
       IN ctrl' = a2.c
    /\ mgr' = [mgr EXCEPT !.dl = TRUE, !.phase = "exited", !.pend = <<>>]
    /\ UNCHANGED <<cfg, rtc, disp, ns, data, ps, ex, rv, outs, flags>>

MgrStep == MgrRtc \/ MgrPending \/ MgrTick

-----------------------------------------------------------------------------
(* PipelineCompletionMsgDispatcher::run (pipeline_ctrl.rs 1160-1235). *)

(* One loop iteration that receives an outcome: drain pending, deliver. *)
DispDeliver ==
    /\ ~disp.exited /\ disp.compl # <<>>
    /\ LET d0 == DDrainR(disp.pend, [c |-> ctrl, p |-> <<>>, fl |-> flags])
           d1 == DDrainR(<<Head(disp.compl)>>, d0)
       IN /\ ctrl' = d1.c
          /\ flags' = d1.fl
          /\ disp' = [disp EXCEPT !.compl = Tail(@), !.pend = d1.p]
    /\ UNCHANGED <<cfg, mgr, rtc, ns, data, ps, ex, rv, outs>>

DispRetry ==
    /\ ~disp.exited /\ disp.pend # <<>>
    /\ LET d == DDrainR(disp.pend, [c |-> ctrl, p |-> <<>>, fl |-> flags])
       IN /\ d.p # disp.pend
          /\ ctrl' = d.c
          /\ flags' = d.fl
          /\ disp' = [disp EXCEPT !.pend = d.p]
    /\ UNCHANGED <<cfg, mgr, rtc, ns, data, ps, ex, rv, outs>>

(* The completion channel closes once every sender holder has stopped   *)
(* (spec.md L1); pending sends are dropped with no signal (1187).        *)
DispExit ==
    /\ ~disp.exited /\ disp.compl = <<>> /\ AllStopped
    /\ disp' = [disp EXCEPT !.exited = TRUE]
    /\ flags' = flags \cup (IF disp.pend # <<>> THEN {"S8"} ELSE {})
    /\ UNCHANGED <<cfg, mgr, rtc, ns, data, ctrl, ps, ex, rv, outs>>

DispStep == DispDeliver \/ DispRetry \/ DispExit

-----------------------------------------------------------------------------

(* Everything stopped: stutter, so that a TLC deadlock means a stall. *)
Done == AllStopped /\ disp.exited /\ UNCHANGED vars

Next ==
    \/ MgrRequest \/ MgrDeadline \/ MgrStep
    \/ DispStep
    \/ \E r \in Requests : Admit(r)
    \/ RecvStep
    \/ \E n \in NonRecv : NodeStep(n)
    \/ \E e \in Exps, r \in Requests : Answer(e, r)
    \/ TimeStep
    \/ GateCloses
    \/ Done

(* The shutdown request, the deadline, admission and the gate closing    *)
(* are environment choices and get no fairness.                          *)
Fairness ==
    /\ WF_vars(MgrStep)
    /\ WF_vars(DispStep)
    /\ WF_vars(RecvStep)
    /\ \A n \in NonRecv : WF_vars(NodeStep(n))
    /\ \A e \in Exps : WF_vars(\E r \in Requests : Answer(e, r))
    /\ WF_vars(TimeStep)

Spec == Init /\ [][Next]_vars /\ Fairness

-----------------------------------------------------------------------------
(* Properties (spec.md "Properties") *)

Flags == {"S1", "S2", "S3", "S4", "S8"}

TypeOK ==
    /\ cfg \in Configs
    /\ mgr.phase \in {"running", "drainIngress", "drainDown", "exited"}
    /\ \A n \in Nodes :
         /\ ns[n].st \in {"run", "drain", "waitIn", "stopped"}
         /\ Len(data[n]) <= Cap
         /\ Len(ctrl[n]) <= CtrlCap
    /\ rv.slots \subseteq rv.wait /\ rv.wait \subseteq rv.admitted
    /\ flags \subseteq Flags

IngressFirst == "S1" \notin flags                    \* S1
TimersFreeze == "S2" \notin flags                    \* S2
DrainByClosure == "S3" \notin flags                  \* S3
RetainedComeBack == "S4" \notin flags                \* S4
NoSilentLateOutcome == "S8" \notin flags             \* S8

OneOutcome ==                                        \* S6, at most one
    \A r \in Requests : Len(outs[r]) <= 1

AnsweredAtEnd ==                                     \* S6, at least one
    AllStopped => \A r \in rv.wait : Len(outs[r]) = 1

(* S7: without a deadline every outcome is a real one. The other half of *)
(* S7 is TLC deadlock detection (CHECK_DEADLOCK TRUE).                   *)
GracefulOutcomes ==
    ~cfg.dlOn =>
        \A r \in Requests : \A i \in DOMAIN outs[r] :
            outs[r][i].src \in {"exp", "proc"}

ProcessorsStopByDeadline == Dl ~> Stopped("P")       \* S5a
EveryNodeStopsAfterDeadline == Dl ~> AllStopped      \* S5b

=============================================================================
