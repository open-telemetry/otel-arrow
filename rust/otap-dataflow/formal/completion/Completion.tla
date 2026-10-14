----------------------------- MODULE Completion -----------------------------
(***************************************************************************)
(* Ack/Nack completion protocol of the OTAP dataflow engine.              *)
(*                                                                         *)
(* The plain-language properties are in spec.md. Each action and each    *)
(* state-transforming operator names the Rust function it mirrors. Paths *)
(* are relative to rust/otap-dataflow/crates/ and line numbers refer to  *)
(* commit 298846b57.                                                      *)
(*                                                                         *)
(* Nodes are strings: "R" receiver, "P" generic processor (T1), "F"      *)
(* fanout, "Y" retry, "D" debug, "A" "B" "C" exporters. A message is a  *)
(* record [units, lin, pay, stack]: the data units it carries, its ghost *)
(* lineage (set of requests), whether it carries a payload, and its      *)
(* frame stack (top = last element).                                     *)
(*                                                                         *)
(* Every frame carries a ghost identity `fid`. Context::subscribe_to      *)
(* always yields a fresh identity, also when it merges into a top frame  *)
(* of the same node, because each subscription expects its own outcome.  *)
(* A clone() of a message keeps the identities of the frames it copied.  *)
(*                                                                         *)
(* Slot keys (receiver, fanout request ids, split keys) are never reused, *)
(* which models the generational slot maps.                               *)
(***************************************************************************)
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS
    Topo,           \* "T1" .. "T5", see spec.md "Topologies"
    NumRequests,    \* number of requests the receiver may admit
    Behaviors,      \* T1: candidate behaviors of processor P
    FanoutModes,    \* T2: candidate fanout await_ack modes
    DebugPorts,     \* T5: candidate numbers of debug main ports
    DebugDetailed,  \* T5: candidate values of verbosity = Detailed
    MaxRetries,     \* T3, T4: retry limit (retry_limit)
    MetricsFrames,  \* push metrics-only entry frames on receive?
    AllowCancel,    \* may a client cancel its request (SlotGuard drop)?
    AllowExit,      \* may the dispatcher exit its run loop?
    Cap,            \* capacity of every pdata channel
    CtrlCap,        \* capacity of every node control channel
    ComplCap,       \* capacity of the pipeline completion channel
    NumDests        \* T2, T4, T5: number of exporters after fanout/debug

ASSUME Topo \in {"T1", "T2", "T3", "T4", "T5"}
ASSUME NumRequests \in Nat \ {0}
ASSUME Behaviors \subseteq
         {"forward", "fwdsub", "ackLocal", "refuse", "consume", "fail",
          "split"}
ASSUME FanoutModes \subseteq {"none", "primary", "all"}
ASSUME NumDests \in 1..3
ASSUME DebugPorts \subseteq 0..NumDests
ASSUME DebugDetailed \subseteq BOOLEAN
ASSUME MaxRetries \in Nat
ASSUME MetricsFrames \in BOOLEAN /\ AllowCancel \in BOOLEAN
ASSUME AllowExit \in BOOLEAN
ASSUME {Cap, CtrlCap, ComplCap} \subseteq Nat \ {0}

Requests == 1..NumRequests
NoCd == <<0, 0>> \* empty calldata

-----------------------------------------------------------------------------
(* Helpers *)

Range(s) == {s[i] : i \in DOMAIN s}
Top(s) == s[Len(s)]
Pop(s) == SubSeq(s, 1, Len(s) - 1)
DropAt(s, i) == [j \in 1..(Len(s) - 1) |-> IF j < i THEN s[j] ELSE s[j + 1]]
Put(f, k, v) == [x \in DOMAIN f \cup {k} |-> IF x = k THEN v ELSE f[x]]
Remove(f, k) == [x \in DOMAIN f \ {k} |-> f[x]]

(* Each request carries two data units, so a detached split can cut it. *)
Units(r) == {<<r, 1>>, <<r, 2>>}

-----------------------------------------------------------------------------
(* Topologies (spec.md "Topologies") *)

Mid ==
    CASE Topo = "T1" -> <<"P">>
      [] Topo = "T2" -> <<"F">>
      [] Topo = "T3" -> <<"Y">>
      [] Topo = "T4" -> <<"Y", "F">>
      [] Topo = "T5" -> <<"D">>
Procs == Range(Mid)
(* Destinations after fanout or debug, in port-name order; A is primary. *)
Dests == SubSeq(<<"A", "B", "C">>, 1, NumDests)
Exporters == IF Topo \in {"T1", "T3"} THEN {"A"} ELSE Range(Dests)
Nodes == {"R"} \cup Procs \cup Exporters

(* Permanent Nacks only matter where a retry processor reads them. *)
NackPerms == IF Topo \in {"T3", "T4"} THEN BOOLEAN ELSE {FALSE}

Configs ==
    [beh : Behaviors, fmode : FanoutModes, ports : DebugPorts,
     det : DebugDetailed]

VARIABLES
    cfg,        \* the configuration of this behavior
    data,       \* pdata channel of each node: seq of messages
    ctrl,       \* control channel of each node: seq of deliveries
    compl,      \* pipeline completion channel: seq of reports
    pending,    \* dispatcher pending_sends: seq of <<node, delivery>>
    todo,       \* effects a node still has to perform (blocked await)
    rtodo,      \* receiver: per request, the blocked initial send
    parked,     \* retry: messages handed to requeue_later
    admitted,   \* requests admitted by the receiver
    waiting,    \* requests with a live receiver slot
    answer,     \* answer seen by each client
    abandoned,  \* ghost: requests explicitly abandoned (spec.md Q5)
    nfid,       \* ghost: next frame identity
    nkey,       \* next fanout request id or split slot key
    slim,       \* fanout slim_inflight: id -> original message
    inflight,   \* fanout inflight: id -> [orig, st]
    splitIn,    \* split processor inbound slots: key -> [m, left, err]
    nodeFailed, \* node whose process() returned an error
    exited,     \* the dispatcher has left its run loop
    exAck,      \* ghost: data units each exporter has Acked
    delivered,  \* ghost: frame identities that received a delivery
    ffUp,       \* ghost: frames that a fanout `none` hand-off Acks
    flags       \* ghost: labels of properties that failed

vars == <<cfg, data, ctrl, compl, pending, todo, rtodo, parked, admitted,
          waiting, answer, abandoned, nfid, nkey, slim, inflight, splitIn,
          nodeFailed, exited, exAck, delivered, ffUp, flags>>

(* Exporters that must Ack a request's data before its Ack (spec.md Q4). *)
Required ==
    CASE Topo = "T1" -> IF cfg.beh = "ackLocal" THEN {} ELSE {"A"}
      [] Topo = "T2" -> (CASE cfg.fmode = "none" -> {}
                           [] cfg.fmode = "primary" -> {"A"}
                           [] OTHER -> Range(Dests))
      [] Topo = "T3" -> {"A"}
      [] Topo = "T4" -> {}
      [] Topo = "T5" -> IF cfg.ports = 0 THEN {} ELSE {"A"}  \* spec.md D2

(* Debug main ports: connected ports minus debug outports (debug 211). *)
Ports == SubSeq(Dests, 1, cfg.ports)

(* Next hop of the retry processor. *)
RetryOut == IF Topo = "T4" THEN "F" ELSE "A"

-----------------------------------------------------------------------------
(* Frames and context stack *)

Frame(fid, node, ack, nack, rd, cd, lin) ==
    [fid |-> fid, node |-> node, ack |-> ack, nack |-> nack, rd |-> rd,
     cd |-> cd, lin |-> lin]

NoFrame == Frame(0, "-", FALSE, FALSE, FALSE, NoCd, {})

Has(f, k) == IF k = "ack" THEN f.ack ELSE f.nack

(* Context::subscribe_to (pdata.rs 431-469): merge into the top frame if  *)
(* it belongs to the same node, otherwise push a frame that inherits      *)
(* RETURN_DATA from the frame below.                                      *)
Subscribe(m, node, ack, nack, rd, cd, fid) ==
    LET s == m.stack IN
    IF s # <<>> /\ Top(s).node = node
    THEN [m EXCEPT !.stack =
            Append(Pop(s), Frame(fid, node, Top(s).ack \/ ack,
                                 Top(s).nack \/ nack, Top(s).rd \/ rd,
                                 cd, m.lin))]
    ELSE [m EXCEPT !.stack =
            Append(s, Frame(fid, node, ack, nack,
                            rd \/ (s # <<>> /\ Top(s).rd), cd, m.lin))]

(* Context::push_entry_frame (pdata.rs 705-742), called on receive by     *)
(* after_processor_receive (processor.rs 668): a frame without Ack/Nack  *)
(* interest that inherits RETURN_DATA.                                    *)
Entry(m, n) ==
    IF MetricsFrames
    THEN LET s == m.stack IN
         [m EXCEPT !.stack =
            Append(s, Frame(nfid, n, FALSE, FALSE,
                            s # <<>> /\ Top(s).rd, NoCd, m.lin))]
    ELSE m
EInc == IF MetricsFrames THEN 1 ELSE 0

(* OtapPdata::clone_without_context (pdata.rs 1064): same data, no frames. *)
CloneWithoutContext(m) == [m EXCEPT !.stack = <<>>]

(* Identity of the topmost frame with interest k, or 0. *)
TopFid(s, k) ==
    LET I == {i \in DOMAIN s : Has(s[i], k)} IN
    IF I = {} THEN 0 ELSE s[CHOOSE i \in I : \A j \in I : j <= i].fid

-----------------------------------------------------------------------------
(* Node effects. A node handler computes a sequence of effects; they run  *)
(* in order until one would block (spec.md "Assumptions"). The rest stays *)
(* in todo[n] and the node takes no other input until it is done.         *)

(* send_message_to / send_message_with_source_node(_to) *)
Send(to, m) == [t |-> "send", to |-> to, kind |-> "-", perm |-> FALSE, m |-> m]
(* notify_ack / notify_nack (pdata.rs 1311-1323) *)
Report(k, p, m) == [t |-> "report", to |-> "-", kind |-> k, perm |-> p, m |-> m]
SendAll(ps, m) == [i \in 1..Len(ps) |-> Send(ps[i], m)]

CanDo(e, d, c) ==
    IF e.t = "send" THEN Len(d[e.to]) < Cap
    ELSE e.m.stack = <<>> \/ Len(c) < ComplCap

(* route_ack / route_nack (effect_handler.rs 364-410) skip a context     *)
(* without frames; otherwise they send DeliverAck / DeliverNack.          *)
RECURSIVE RunR(_, _, _)
RunR(d, c, es) ==
    IF es = <<>> \/ ~CanDo(Head(es), d, c)
    THEN [d |-> d, c |-> c, rest |-> es]
    ELSE LET e == Head(es) IN
         IF e.t = "send"
         THEN RunR([d EXCEPT ![e.to] = Append(@, e.m)], c, Tail(es))
         ELSE IF e.m.stack = <<>>
              THEN RunR(d, c, Tail(es))
              ELSE RunR(d, Append(c, [kind |-> e.kind, perm |-> e.perm,
                                      m |-> e.m]), Tail(es))

Exec(n, d0, es) ==
    LET res == RunR(d0, compl, es) IN
    /\ data' = res.d
    /\ compl' = res.c
    /\ todo' = [todo EXCEPT ![n] = res.rest]

Guard(n) == ~exited /\ ~nodeFailed[n] /\ todo[n] = <<>>

-----------------------------------------------------------------------------
Init ==
    /\ cfg \in Configs
    /\ data = [n \in Nodes |-> <<>>]
    /\ ctrl = [n \in Nodes |-> <<>>]
    /\ compl = <<>>
    /\ pending = <<>>
    /\ todo = [n \in Nodes |-> <<>>]
    /\ rtodo = [r \in Requests |-> <<>>]
    /\ parked = <<>>
    /\ admitted = {}
    /\ waiting = {}
    /\ answer = [r \in Requests |-> "none"]
    /\ abandoned = {}
    /\ nfid = 1
    /\ nkey = 1
    /\ slim = [k \in {} |-> 0]
    /\ inflight = [k \in {} |-> 0]
    /\ splitIn = [k \in {} |-> 0]
    /\ nodeFailed = [n \in Nodes |-> FALSE]
    /\ exited = FALSE
    /\ exAck = [e \in Exporters |-> {}]
    /\ delivered = {}
    /\ ffUp = {}
    /\ flags = {}

-----------------------------------------------------------------------------
(* Receiver: OTLP gRPC export handler (server_new.rs 470-547) and the    *)
(* receiver's Ack/Nack handling (otlp_receiver/mod.rs 398-421).           *)

(* send_message_with_source_node (server_new.rs 523-531). A send to a    *)
(* failed node's closed channel returns an error status to the client    *)
(* (pipeline_send_status) and the SlotGuard frees the slot.              *)
RecvSend(r, m, w0) ==
    LET p == Mid[1] IN
    IF nodeFailed[p]
    THEN /\ answer' = [answer EXCEPT ![r] = "err"]
         /\ waiting' = w0 \ {r}
         /\ rtodo' = [rtodo EXCEPT ![r] = <<>>]
         /\ UNCHANGED data
    ELSE IF Len(data[p]) < Cap
    THEN /\ data' = [data EXCEPT ![p] = Append(@, m)]
         /\ rtodo' = [rtodo EXCEPT ![r] = <<>>]
         /\ waiting' = w0
         /\ UNCHANGED answer
    ELSE /\ rtodo' = [rtodo EXCEPT ![r] = <<m>>]
         /\ waiting' = w0
         /\ UNCHANGED <<data, answer>>

(* allocate_slot + subscribe_to(ACKS | NACKS, key) (server_new.rs 486-509) *)
Admit(r) ==
    /\ ~exited
    /\ r \notin admitted
    /\ r - 1 \in admitted \cup {0}
    /\ LET m == Subscribe([units |-> Units(r), lin |-> {r}, pay |-> TRUE,
                           stack |-> <<>>],
                          "R", TRUE, TRUE, FALSE, <<r, 0>>, nfid)
       IN /\ admitted' = admitted \cup {r}
          /\ nfid' = nfid + 1
          /\ RecvSend(r, m, waiting \cup {r})
    /\ UNCHANGED <<cfg, ctrl, compl, pending, todo, parked, abandoned, nkey,
                   slim, inflight, splitIn, nodeFailed, exited, exAck,
                   delivered, ffUp, flags>>

(* The awaited send completes. *)
RecvResume(r) ==
    /\ ~exited
    /\ rtodo[r] # <<>>
    /\ nodeFailed[Mid[1]] \/ Len(data[Mid[1]]) < Cap
    /\ RecvSend(r, Head(rtodo[r]), waiting)
    /\ UNCHANGED <<cfg, ctrl, compl, pending, todo, parked, admitted,
                   abandoned, nfid, nkey, slim, inflight, splitIn,
                   nodeFailed, exited, exAck, delivered, ffUp, flags>>

(* The client gives up: Tonic drops the future, SlotGuard::drop cancels  *)
(* the slot (server_new.rs 368-378).                                      *)
Cancel(r) ==
    /\ AllowCancel
    /\ ~exited
    /\ r \in waiting
    /\ waiting' = waiting \ {r}
    /\ abandoned' = abandoned \cup {r}
    /\ rtodo' = [rtodo EXCEPT ![r] = <<>>]
    /\ UNCHANGED <<cfg, data, ctrl, compl, pending, todo, parked, admitted,
                   answer, nfid, nkey, slim, inflight, splitIn, nodeFailed,
                   exited, exAck, delivered, ffUp, flags>>

(* handle_ack / handle_nack -> route_ack_response / route_nack_response  *)
(* -> AckSlot::route_response (server_new.rs 129-150). A removed key     *)
(* returns Expired and changes nothing.                                   *)
ReceiverHandleOutcome ==
    /\ ~exited
    /\ ctrl["R"] # <<>>
    /\ LET it == Head(ctrl["R"])
           r == it.cd[1]
       IN /\ ctrl' = [ctrl EXCEPT !["R"] = Tail(@)]
          /\ IF r \in waiting
             THEN /\ answer' = [answer EXCEPT ![r] = it.kind]
                  /\ waiting' = waiting \ {r}
                  /\ flags' = flags \cup
                       (IF it.kind = "ack" /\
                           \E e \in Required : ~(Units(r) \subseteq exAck[e])
                        THEN {"Q4"} ELSE {})
             ELSE UNCHANGED <<answer, waiting, flags>>
    /\ UNCHANGED <<cfg, data, compl, pending, todo, rtodo, parked, admitted,
                   abandoned, nfid, nkey, slim, inflight, splitIn,
                   nodeFailed, exited, exAck, delivered, ffUp>>

-----------------------------------------------------------------------------
(* Exporter: takes one message and answers it once with the payload.     *)
(* Ack, transient Nack, or permanent Nack (spec.md "Assumptions").        *)

Outcomes ==
    {[kind |-> "ack", perm |-> FALSE]} \cup
    {[kind |-> "nack", perm |-> p] : p \in NackPerms}

Export(e) ==
    /\ Guard(e)
    /\ data[e] # <<>>
    /\ \E o \in Outcomes :
         LET m == Head(data[e]) IN
         /\ exAck' = IF o.kind = "ack"
                     THEN [exAck EXCEPT ![e] = @ \cup m.units]
                     ELSE exAck
         /\ Exec(e, [data EXCEPT ![e] = Tail(@)], <<Report(o.kind, o.perm, m)>>)
    /\ UNCHANGED <<cfg, ctrl, pending, rtodo, parked, admitted, waiting,
                   answer, abandoned, nfid, nkey, slim, inflight, splitIn,
                   nodeFailed, exited, delivered, ffUp, flags>>

(* A blocked send or notify completes (the awaited future resolves). *)
Continue(n) ==
    /\ ~exited
    /\ ~nodeFailed[n]
    /\ todo[n] # <<>>
    /\ CanDo(Head(todo[n]), data, compl)
    /\ Exec(n, data, todo[n])
    /\ UNCHANGED <<cfg, ctrl, pending, rtodo, parked, admitted, waiting,
                   answer, abandoned, nfid, nkey, slim, inflight, splitIn,
                   nodeFailed, exited, exAck, delivered, ffUp, flags>>

-----------------------------------------------------------------------------
(* T1: generic processor P. cfg.beh selects the pattern (spec.md        *)
(* "Behavior patterns").                                                  *)

PUnchanged ==
    UNCHANGED <<cfg, pending, rtodo, parked, admitted, waiting, answer,
                slim, inflight, exited, exAck, delivered, ffUp, flags>>

P_ProcessPData ==
    /\ Guard("P")
    /\ data["P"] # <<>>
    /\ LET m == Entry(Head(data["P"]), "P")
           f == nfid + EInc
           d0 == [data EXCEPT !["P"] = Tail(@)]
           Part(i) ==
               [units |-> {u \in m.units : u[2] = i}, lin |-> m.lin,
                pay |-> TRUE,
                stack |-> <<Frame(f + i - 1, "P", TRUE, TRUE, FALSE,
                                  <<nkey, i>>, m.lin)>>]
       IN CASE cfg.beh = "forward" ->
                 \* attributes, filter, log_sampling: move and send
                 /\ Exec("P", d0, <<Send("A", m)>>)
                 /\ nfid' = f
                 /\ UNCHANGED <<ctrl, nkey, splitIn, nodeFailed, abandoned>>
            [] cfg.beh = "fwdsub" ->
                 \* debug Detailed with one port (debug 327-347)
                 /\ Exec("P", d0,
                         <<Send("A", Subscribe(m, "P", TRUE, TRUE, FALSE,
                                               NoCd, f))>>)
                 /\ nfid' = f + 1
                 /\ UNCHANGED <<ctrl, nkey, splitIn, nodeFailed, abandoned>>
            [] cfg.beh = "ackLocal" ->
                 \* complete_processor_without_output + notify_ack
                 \* (partition 294-298, log_sampling 176-178)
                 /\ Exec("P", d0, <<Report("ack", FALSE, m)>>)
                 /\ nfid' = f
                 /\ UNCHANGED <<ctrl, nkey, splitIn, nodeFailed, abandoned>>
            [] cfg.beh = "refuse" ->
                 \* processor-originated Nack (log_sampling 152-160)
                 /\ Exec("P", d0, <<Report("nack", FALSE, m)>>)
                 /\ nfid' = f
                 /\ UNCHANGED <<ctrl, nkey, splitIn, nodeFailed, abandoned>>
            [] cfg.beh = "consume" ->
                 \* take the message, return Ok, never answer
                 /\ Exec("P", d0, <<>>)
                 /\ nfid' = f
                 /\ UNCHANGED <<ctrl, nkey, splitIn, nodeFailed, abandoned>>
            [] cfg.beh = "fail" ->
                 \* process() returns Err: the engine stops the node
                 \* (processor.rs 674-676); its channels close.
                 /\ nodeFailed' = [nodeFailed EXCEPT !["P"] = TRUE]
                 /\ abandoned' = abandoned \cup
                                 UNION {x.lin : x \in Range(data["P"])}
                 /\ data' = [data EXCEPT !["P"] = <<>>]
                 /\ ctrl' = [ctrl EXCEPT !["P"] = <<>>]
                 /\ nfid' = f
                 /\ UNCHANGED <<compl, todo, nkey, splitIn>>
            [] cfg.beh = "split" ->
                 \* detached split (partition 326-425): park the inbound
                 \* context, send two parts with clone_detached + subscribe
                 /\ splitIn' = Put(splitIn, nkey,
                        [m |-> [m EXCEPT !.pay =
                                  \E i \in DOMAIN m.stack : m.stack[i].rd],
                         left |-> 2, err |-> FALSE])
                 /\ Exec("P", d0, <<Send("A", Part(1)), Send("A", Part(2))>>)
                 /\ nfid' = f + 2
                 /\ nkey' = nkey + 1
                 /\ UNCHANGED <<ctrl, nodeFailed, abandoned>>
    /\ PUnchanged

P_ProcessControl ==
    /\ Guard("P")
    /\ ctrl["P"] # <<>>
    /\ LET it == Head(ctrl["P"])
           key == it.cd[1]
       IN /\ ctrl' = [ctrl EXCEPT !["P"] = Tail(@)]
          /\ CASE cfg.beh = "fwdsub" ->
                    \* debug Ack/Nack branch: forward upstream (debug 273-313)
                    /\ Exec("P", data, <<Report(it.kind, it.perm, it.m)>>)
                    /\ UNCHANGED splitIn
               [] cfg.beh = "split" /\ key \in DOMAIN splitIn ->
                    \* handle_ack_nack_inbound (transform 445-470)
                    LET s == splitIn[key]
                        err == s.err \/ it.kind = "nack"
                    IN IF s.left = 1
                       THEN /\ splitIn' = Remove(splitIn, key)
                            /\ Exec("P", data,
                                    <<Report(IF err THEN "nack" ELSE "ack",
                                             FALSE, s.m)>>)
                       ELSE /\ splitIn' = [splitIn EXCEPT ![key] =
                                             [s EXCEPT !.left = s.left - 1,
                                                       !.err = err]]
                            /\ UNCHANGED <<data, compl, todo>>
               [] OTHER ->
                    UNCHANGED <<splitIn, data, compl, todo>>
    /\ UNCHANGED <<nfid, nkey, nodeFailed, abandoned>>
    /\ PUnchanged

-----------------------------------------------------------------------------
(* Fanout processor, parallel mode, NumDests destinations, A is primary, *)
(* no fallback, no timeout (fanout 1066-1160).                            *)

FUnchanged ==
    UNCHANGED <<cfg, pending, rtodo, parked, admitted, waiting,
                answer, abandoned, splitIn, nodeFailed, exited, exAck,
                delivered, flags>>

F_ProcessPData ==
    /\ Guard("F")
    /\ data["F"] # <<>>
    /\ LET m == Entry(Head(data["F"]), "F")
           f == nfid + EInc
           d0 == [data EXCEPT !["F"] = Tail(@)]
           id == nkey
           Copy(i) == Subscribe(m, "F", TRUE, TRUE, FALSE, <<id, i>>, f + i - 1)
           Copies == [i \in 1..NumDests |-> Send(Dests[i], Copy(i))]
       IN CASE cfg.fmode = "none" ->
                 \* process_fire_and_forget (fanout 932-952): the copies
                 \* are clone_without_context(), fanout Acks upstream
                 /\ Exec("F", d0, SendAll(Dests, CloneWithoutContext(m))
                                 \o <<Report("ack", FALSE, m)>>)
                 /\ nfid' = f
                 /\ ffUp' = ffUp \cup ({TopFid(m.stack, "ack")} \ {0})
                 /\ UNCHANGED <<nkey, slim, inflight>>
            [] cfg.fmode = "primary" ->
                 \* process_slim_primary (fanout 968-994)
                 /\ Exec("F", d0, Copies)
                 /\ nfid' = f + NumDests
                 /\ nkey' = nkey + 1
                 /\ slim' = Put(slim, id, m)
                 /\ UNCHANGED <<inflight, ffUp>>
            [] cfg.fmode = "all" ->
                 \* register_inflight + dispatch_ready (fanout 486-600)
                 /\ Exec("F", d0, Copies)
                 /\ nfid' = f + NumDests
                 /\ nkey' = nkey + 1
                 /\ inflight' = Put(inflight, id,
                                    [orig |-> m,
                                     st |-> [i \in 1..NumDests |-> "in"]])
                 /\ UNCHANGED <<slim, ffUp>>
    /\ UNCHANGED ctrl
    /\ FUnchanged

F_ProcessControl ==
    /\ Guard("F")
    /\ ctrl["F"] # <<>>
    /\ LET it == Head(ctrl["F"])
           id == it.cd[1]
           idx == it.cd[2]
       IN /\ ctrl' = [ctrl EXCEPT !["F"] = Tail(@)]
          /\ CASE cfg.fmode = "primary" ->
                    \* process_ack_slim_primary / process_nack_slim_primary
                    \* (fanout 996-1046): non-primary outcomes are ignored
                    /\ IF idx = 1 /\ id \in DOMAIN slim
                       THEN /\ slim' = Remove(slim, id)
                            /\ Exec("F", data,
                                    <<Report(it.kind, it.perm, slim[id])>>)
                       ELSE UNCHANGED <<slim, data, compl, todo>>
                    /\ UNCHANGED inflight
               [] cfg.fmode = "all" /\ it.kind = "ack" ->
                    \* process_ack + mark_complete (fanout 751-857)
                    /\ IF id \in DOMAIN inflight /\
                          inflight[id].st[idx] # "nacked"
                       THEN LET st2 == [inflight[id].st EXCEPT ![idx] = "acked"]
                            IN IF \A j \in 1..NumDests : st2[j] = "acked"
                               THEN /\ inflight' = Remove(inflight, id)
                                    /\ Exec("F", data,
                                            <<Report("ack", FALSE,
                                                     inflight[id].orig)>>)
                               ELSE /\ inflight' = [inflight EXCEPT
                                                      ![id].st = st2]
                                    /\ UNCHANGED <<data, compl, todo>>
                       ELSE UNCHANGED <<inflight, data, compl, todo>>
                    /\ UNCHANGED slim
               [] cfg.fmode = "all" /\ it.kind = "nack" ->
                    \* process_nack + handle_failure (fanout 615-665,
                    \* 859-923): no fallback, Nack upstream from original
                    /\ IF id \in DOMAIN inflight
                       THEN /\ inflight' = Remove(inflight, id)
                            /\ Exec("F", data,
                                    <<Report("nack", it.perm,
                                             inflight[id].orig)>>)
                       ELSE UNCHANGED <<inflight, data, compl, todo>>
                    /\ UNCHANGED slim
               [] OTHER ->
                    \* `none`: process_ack finds no inflight entry
                    UNCHANGED <<slim, inflight, data, compl, todo>>
    /\ UNCHANGED <<nfid, nkey, ffUp>>
    /\ FUnchanged

-----------------------------------------------------------------------------
(* Retry processor (retry 454-672). The retry delay and the deadline are *)
(* abstracted: a parked message resumes at any time, and a transient     *)
(* Nack below the retry limit may still hit the deadline.                 *)

YUnchanged ==
    UNCHANGED <<cfg, pending, rtodo, admitted, waiting, answer, abandoned,
                nkey, slim, inflight, splitIn, nodeFailed, exited, exAck,
                delivered, ffUp, flags>>

(* process, PData branch: subscribe ACKS|NACKS|RETURN_DATA with a fresh  *)
(* RetryState, then send_or_nack (retry 656-672, 619-646).               *)
Y_ProcessPData ==
    /\ Guard("Y")
    /\ data["Y"] # <<>>
    /\ LET m == Entry(Head(data["Y"]), "Y")
           f == nfid + EInc
       IN /\ Exec("Y", [data EXCEPT !["Y"] = Tail(@)],
                  <<Send(RetryOut, Subscribe(m, "Y", TRUE, TRUE, TRUE,
                                             <<0, 0>>, f))>>)
          /\ nfid' = f + 1
    /\ UNCHANGED <<ctrl, parked>>
    /\ YUnchanged

(* handle_ack (retry 454-470) and handle_nack (retry 495-606). *)
Y_ProcessControl ==
    /\ Guard("Y")
    /\ ctrl["Y"] # <<>>
    /\ LET it == Head(ctrl["Y"])
           rs == it.cd[1]
       IN /\ ctrl' = [ctrl EXCEPT !["Y"] = Tail(@)]
          /\ IF it.kind = "ack"
             THEN /\ Exec("Y", data, <<Report("ack", FALSE, it.m)>>)
                  /\ UNCHANGED <<parked, nfid>>
             ELSE IF it.perm \/ ~it.m.pay \/ rs >= MaxRetries
             THEN \* terminate_nack: permanent, payload missing, limit
                  /\ Exec("Y", data, <<Report("nack", it.perm, it.m)>>)
                  /\ UNCHANGED <<parked, nfid>>
             ELSE \/ \* deadline reached (retry 557-566)
                     /\ Exec("Y", data, <<Report("nack", FALSE, it.m)>>)
                     /\ UNCHANGED <<parked, nfid>>
                  \/ \* subscribe_to + requeue_later (retry 586-606)
                     /\ parked' = Append(parked,
                                    Subscribe(it.m, "Y", TRUE, TRUE, TRUE,
                                              <<rs + 1, 0>>, nfid))
                     /\ nfid' = nfid + 1
                     /\ UNCHANGED <<data, compl, todo>>
    /\ YUnchanged

(* ResumeData -> handle_resumed -> send_or_nack (retry 610-646). *)
Y_Resume ==
    /\ Guard("Y")
    /\ parked # <<>>
    /\ \E i \in DOMAIN parked :
         /\ parked' = DropAt(parked, i)
         /\ Exec("Y", data, <<Send(RetryOut, parked[i])>>)
    /\ UNCHANGED <<ctrl, nfid>>
    /\ YUnchanged

-----------------------------------------------------------------------------
(* Debug processor with output ports (debug 200-350). *)

DUnchanged ==
    UNCHANGED <<cfg, pending, rtodo, parked, admitted, waiting, answer,
                abandoned, nkey, slim, inflight, splitIn, nodeFailed,
                exited, exAck, delivered, ffUp, flags>>

(* PData branch: optional subscribe in Detailed mode, then one message   *)
(* per main port, then the original is consumed locally (debug 327-350). *)
(* With no main port the processor is a sink: it Acks upstream without   *)
(* subscribing (fix 3).                                                  *)
D_ProcessPData ==
    /\ Guard("D")
    /\ data["D"] # <<>>
    /\ LET m == Entry(Head(data["D"]), "D")
           f == nfid + EInc
           sub == cfg.det /\ cfg.ports > 0
           m2 == IF sub
                 THEN Subscribe(m, "D", TRUE, TRUE, FALSE, NoCd, f)
                 ELSE m
           \* first main port keeps the context, the others get
           \* clone_without_context() copies (fix 2, spec.md D2)
           es == IF cfg.ports = 0
                 THEN <<Report("ack", FALSE, m)>>
                 ELSE <<Send(Ports[1], m2)>> \o
                      SendAll(Tail(Ports), CloneWithoutContext(m))
       IN /\ Exec("D", [data EXCEPT !["D"] = Tail(@)], es)
          /\ nfid' = f + (IF sub THEN 1 ELSE 0)
    /\ UNCHANGED ctrl
    /\ DUnchanged

(* Ack/Nack branch: forward the outcome upstream (debug 273-313). *)
D_ProcessControl ==
    /\ Guard("D")
    /\ ctrl["D"] # <<>>
    /\ LET it == Head(ctrl["D"]) IN
       /\ ctrl' = [ctrl EXCEPT !["D"] = Tail(@)]
       /\ Exec("D", data, <<Report(it.kind, it.perm, it.m)>>)
    /\ UNCHANGED nfid
    /\ DUnchanged

-----------------------------------------------------------------------------
(* PipelineCompletionMsgDispatcher (pipeline_ctrl.rs 974-1330). *)

(* unwind_frames (1237-1286): pop frames until one has interest k. *)
RECURSIVE UnwindR(_, _)
UnwindR(s, k) ==
    IF s = <<>> THEN [found |-> FALSE, frame |-> NoFrame, rest |-> <<>>]
    ELSE IF Has(Top(s), k)
         THEN [found |-> TRUE, frame |-> Top(s), rest |-> Pop(s)]
         ELSE UnwindR(Pop(s), k)

(* drain_pending_sends (1044-1068): one pass in order. A closed channel  *)
(* drops the message without a signal (1064).                            *)
RECURSIVE DrainR(_, _, _, _)
DrainR(p, c, keep, fl) ==
    IF p = <<>> THEN [p |-> keep, c |-> c, fl |-> fl]
    ELSE LET x == Head(p)
             n == x[1]
         IN IF nodeFailed[n]
            THEN DrainR(Tail(p), c, keep, fl \cup {"Q6"})
            ELSE IF Len(c[n]) < CtrlCap
                 THEN DrainR(Tail(p), [c EXCEPT ![n] = Append(@, x[2])],
                             keep, fl)
                 ELSE DrainR(Tail(p), c, Append(keep, x), fl)

(* One run-loop iteration that receives a message (1160-1201): drain    *)
(* pending sends, then unwind_ack / unwind_nack (1288-1325) and send    *)
(* (1011-1042).                                                          *)
DispatcherReceive ==
    /\ ~exited
    /\ compl # <<>>
    /\ LET dr == DrainR(pending, ctrl, <<>>, flags)
           it == Head(compl)
           u == UnwindR(it.m.stack, it.kind)
           f == u.frame
           n == f.node
           item == [kind |-> it.kind, perm |-> it.perm, cd |-> f.cd,
                    m |-> [it.m EXCEPT !.stack = u.rest,
                                       !.pay = it.m.pay /\ f.rd]]
           chk == (IF f.fid \in delivered THEN {"Q1"} ELSE {})
                  \cup (IF f.lin \cap it.m.lin = {} THEN {"Q2"} ELSE {})
                  \cup (IF f.fid # TopFid(it.m.stack, it.kind)
                        THEN {"Q3"} ELSE {})
                  \cup (IF f.rd /\ ~it.m.pay THEN {"Q7"} ELSE {})
                  \cup (IF f.fid \in ffUp /\
                           (it.kind = "nack" \/ f.fid \in delivered)
                        THEN {"Q8"} ELSE {})
       IN /\ compl' = Tail(compl)
          /\ IF ~u.found
             THEN \* record_*_dropped_no_interest (1302, 1321)
                  /\ pending' = dr.p
                  /\ ctrl' = dr.c
                  /\ flags' = dr.fl
                  /\ UNCHANGED delivered
             ELSE /\ delivered' = delivered \cup {f.fid}
                  /\ IF nodeFailed[n]
                     THEN \* SendError::Closed, dropped silently (1039)
                          /\ pending' = dr.p
                          /\ ctrl' = dr.c
                          /\ flags' = dr.fl \cup chk \cup {"Q6"}
                     ELSE IF Len(dr.c[n]) < CtrlCap
                     THEN /\ ctrl' = [dr.c EXCEPT ![n] = Append(@, item)]
                          /\ pending' = dr.p
                          /\ flags' = dr.fl \cup chk
                     ELSE \* SendError::Full -> pending_sends (1025-1037)
                          /\ pending' = Append(dr.p, <<n, item>>)
                          /\ ctrl' = dr.c
                          /\ flags' = dr.fl \cup chk
    /\ UNCHANGED <<cfg, data, todo, rtodo, parked, admitted, waiting,
                   answer, abandoned, nfid, nkey, slim, inflight, splitIn,
                   nodeFailed, exited, exAck, ffUp>>

(* The 5 ms retry timer fires: drain_pending_sends (1196-1201, 1162). *)
DispatcherRetryPending ==
    /\ ~exited
    /\ pending # <<>>
    /\ LET dr == DrainR(pending, ctrl, <<>>, flags) IN
       /\ dr.p # pending
       /\ pending' = dr.p
       /\ ctrl' = dr.c
       /\ flags' = dr.fl
    /\ UNCHANGED <<cfg, data, compl, todo, rtodo, parked, admitted, waiting,
                   answer, abandoned, nfid, nkey, slim, inflight, splitIn,
                   nodeFailed, exited, exAck, delivered, ffUp>>

(* The completion channel is closed and empty: the loop breaks (1187)   *)
(* and pending_sends is dropped with the dispatcher. Only after every   *)
(* node stopped sending; shutdown itself is Part 3.                      *)
DispatcherExit ==
    /\ AllowExit
    /\ ~exited
    /\ compl = <<>>
    /\ \A n \in Nodes : todo[n] = <<>>
    /\ \A r \in Requests : rtodo[r] = <<>>
    /\ exited' = TRUE
    /\ flags' = flags \cup (IF pending # <<>> THEN {"Q6"} ELSE {})
    /\ UNCHANGED <<cfg, data, ctrl, compl, pending, todo, rtodo, parked,
                   admitted, waiting, answer, abandoned, nfid, nkey, slim,
                   inflight, splitIn, nodeFailed, exAck, delivered, ffUp>>

-----------------------------------------------------------------------------
Next ==
    \/ \E r \in Requests : Admit(r) \/ RecvResume(r) \/ Cancel(r)
    \/ ReceiverHandleOutcome
    \/ \E e \in Exporters : Export(e)
    \/ \E n \in Nodes \ {"R"} : Continue(n)
    \/ "P" \in Procs /\ (P_ProcessPData \/ P_ProcessControl)
    \/ "F" \in Procs /\ (F_ProcessPData \/ F_ProcessControl)
    \/ "Y" \in Procs /\ (Y_ProcessPData \/ Y_ProcessControl \/ Y_Resume)
    \/ "D" \in Procs /\ (D_ProcessPData \/ D_ProcessControl)
    \/ DispatcherReceive
    \/ DispatcherRetryPending
    \/ DispatcherExit

Spec == Init /\ [][Next]_vars

-----------------------------------------------------------------------------
(* Properties (spec.md "Properties") *)

Flags == {"Q1", "Q2", "Q3", "Q4", "Q6", "Q7", "Q8"}

TypeOK ==
    /\ cfg \in Configs
    /\ admitted \subseteq Requests
    /\ waiting \subseteq admitted
    /\ abandoned \subseteq admitted
    /\ answer \in [Requests -> {"none", "ack", "nack", "err"}]
    /\ flags \subseteq Flags
    /\ \A n \in Nodes : Len(data[n]) <= Cap /\ Len(ctrl[n]) <= CtrlCap
    /\ Len(compl) <= ComplCap

AtMostOneDelivery == "Q1" \notin flags            \* Q1
LineageOnly == "Q2" \notin flags                  \* Q2
LifoDelivery == "Q3" \notin flags                 \* Q3
AckOnlyAfterDelivery == "Q4" \notin flags         \* Q4
NoSilentDispatcherDrop == "Q6" \notin flags       \* Q6
ReturnDataOnlyWhereAsked == "Q7" \notin flags     \* Q7
FanoutNoneAnswersOnceWithAck == "Q8" \notin flags \* Q8

Drained ==
    /\ compl = <<>>
    /\ pending = <<>>
    /\ parked = <<>>
    /\ \A n \in Nodes : data[n] = <<>> /\ ctrl[n] = <<>> /\ todo[n] = <<>>
    /\ \A r \in Requests : rtodo[r] = <<>>

DrainedMeansAnswered ==                           \* Q5
    Drained =>
        /\ \A r \in admitted : answer[r] # "none" \/ r \in abandoned
        /\ waiting \subseteq abandoned
        /\ DOMAIN slim = {}
        /\ DOMAIN inflight = {}
        /\ DOMAIN splitIn = {}

=============================================================================
