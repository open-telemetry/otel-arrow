-------------------------- MODULE BatchAggregation --------------------------
(***************************************************************************)
(* Ack/Nack aggregation in the OTAP batch processor.                      *)
(*                                                                         *)
(* The plain-language properties are in spec.md. Each action and each    *)
(* state-transforming operator below names the Rust function it mirrors   *)
(* in crates/core-nodes/src/processors/batch_processor/mod.rs, as of the  *)
(* metrics debug_assert fix (see spec.md). Line numbers refer to that    *)
(* file unless noted.                                                     *)
(*                                                                         *)
(* One signal, one format. Data is measured in abstract units. Every unit *)
(* of request r is the tuple <<r, k>>; header units that byte splitting   *)
(* duplicates are <<0, k>> and belong to no request.                      *)
(*                                                                         *)
(* Slot keys: an inbound key is the request id; an outbound key is the    *)
(* set of units its batch carries. Neither is ever reused, which models   *)
(* the generational slot map in crates/otap/src/accessory/slots.rs.       *)
(***************************************************************************)
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS
    NumRequests,     \* number of distinct requests that may arrive
    Mode,            \* "aligned" (OTAP logs/traces), "rows" (OTAP
                     \* metrics), "bytes" (OTLP)
    MaxReqSize,      \* request sizes range over 0..MaxReqSize units
    MinSizes,        \* candidate min_size values
    MaxSizes,        \* candidate max_size values; 0 means unset
    InboundLimits,   \* candidate inbound_request_limit values
    OutboundLimits,  \* candidate outbound_request_limit values
    Overheads,       \* bytes mode: header units repeated per fragment
    AllowBatchError, \* may make_batches fail (mod.rs 1151-1164)?
    AllowFallback,   \* bytes mode: may a split be refused by a budget?
    FreeSplit        \* bytes mode: TRUE explores every fragment layout;
                     \* FALSE uses equal blocks with overhead cfg.h

ASSUME Mode \in {"aligned", "rows", "bytes"}
ASSUME NumRequests \in Nat \ {0} /\ MaxReqSize \in Nat \ {0}
ASSUME AllowBatchError \in BOOLEAN /\ AllowFallback \in BOOLEAN
ASSUME FreeSplit \in BOOLEAN

Requests == 1..NumRequests
NoKey == 0          \* BatchPortion.inkey = None
NoOut == {}         \* batch sent without an outbound slot

-----------------------------------------------------------------------------
(* Configurations. All combinations are explored in one run; cfg stays   *)
(* fixed after Init. Invalid combinations are excluded as FormatConfig::  *)
(* validate (mod.rs 460-506) would reject them.                          *)

Configs ==
    {c \in [min : MinSizes, max : MaxSizes, inl : InboundLimits,
            outl : OutboundLimits,
            h : IF Mode = "bytes" THEN Overheads ELSE {0}] :
        c.max = 0 \/ c.max >= c.min}

VARIABLES
    cfg,        \* the configuration of this behavior
    buf,        \* SignalBuffer.inputs: seq of portions, see Portion below
    live,       \* requests with a live inbound slot
    cnt,        \* BatchContext.outbound, per request
    outs,       \* outbound slot map: key -> seq of [r : request]
    inflight,   \* batches sent downstream and not yet answered
    nAck,       \* Acks sent upstream, per request
    nNack,      \* Nacks sent upstream, per request
    confirmed,  \* ghost: units confirmed by a downstream Ack
    lost,       \* ghost: units in a Nacked batch or dropped on error
    failed,     \* ghost: labels of guards or properties that failed
    arrived,    \* ghost: requests that have arrived
    trk,        \* ghost: arrived requests that asked for tracking
    reqSize,    \* ghost: size of each arrived request
    shutdown    \* Shutdown has been processed

vars == <<cfg, buf, live, cnt, outs, inflight, nAck, nNack, confirmed,
          lost, failed, arrived, trk, reqSize, shutdown>>

Lower == cfg.min    \* FormatConfig::lower_limit (439); min_size is set
Max == cfg.max      \* 0 means max_size unset

(* A portion mirrors BatchPortion (525) plus its payload:                 *)
(*   p     : the payload, a seq of chunks; a chunk is a seq of units     *)
(*           (an entry in bytes mode, a metric row in rows mode)          *)
(*   inkey : request id, or NoKey when untracked                          *)
(*   w     : weight in the active sizer's unit                            *)
(*   ret   : the `retained` flag                                          *)
(*   src   : ghost, the request the data came from                        *)

-----------------------------------------------------------------------------
(* Sequence helpers *)

Min(a, b) == IF a < b THEN a ELSE b
Range(s) == {s[i] : i \in DOMAIN s}

RECURSIVE Flatten(_)
Flatten(s) == IF s = <<>> THEN <<>> ELSE Head(s) \o Flatten(Tail(s))

PSize(p) == Len(Flatten(p))        \* real size of a payload

RECURSIVE SumW(_)
SumW(s) == IF s = <<>> THEN 0 ELSE Head(s).w + SumW(Tail(s))

RECURSIVE SumOwn(_)
SumOwn(s) == IF s = <<>> THEN 0 ELSE Head(s).own + SumOwn(Tail(s))

RECURSIVE SumF(_, _)
SumF(f, ks) ==
    IF ks = {} THEN 0
    ELSE LET k == CHOOSE x \in ks : TRUE IN f[k] + SumF(f, ks \ {k})

Remove(f, key) == [k \in DOMAIN f \ {key} |-> f[k]]

DataUnits(p) == {u \in Range(Flatten(p)) : u[1] # 0}
UnitsOf(r) == {<<r, k>> : k \in 1..reqSize[r]}

(* Cut a seq of units into blocks of at most m, the first of size f. *)
RECURSIVE Blocks(_, _, _)
Blocks(u, f, m) ==
    IF Len(u) <= f THEN <<u>>
    ELSE <<SubSeq(u, 1, f)>> \o Blocks(SubSeq(u, f + 1, Len(u)), m, m)

-----------------------------------------------------------------------------
(* Request shapes. A request of n units is a composition of n into      *)
(* chunks. In aligned mode chunk boundaries do not matter, so one shape  *)
(* per size is enough.                                                    *)

RECURSIVE Compositions(_)
Compositions(n) ==
    IF n = 0 THEN {<<>>}
    ELSE UNION {{<<k>> \o c : c \in Compositions(n - k)} : k \in 1..n}

Shapes ==
    IF Mode = "aligned"
    THEN {<<>>} \cup {<<n>> : n \in 1..MaxReqSize}
    ELSE UNION {Compositions(n) : n \in 0..MaxReqSize}

RECURSIVE Offset(_, _)
Offset(c, i) == IF i = 0 THEN 0 ELSE Offset(c, i - 1) + c[i]

MakePayload(r, c) ==
    [i \in 1..Len(c) |-> [k \in 1..c[i] |-> <<r, Offset(c, i - 1) + k>>]]

-----------------------------------------------------------------------------
(* Batching: Batcher::make_batches (890, 931). Each output batch is       *)
(* [chunks, real, own]: its payload, its real size, and its ownership     *)
(* weight. Batches come out in input order.                               *)

(* frag is a ghost flag: TRUE for a split fragment.                      *)
ItemBatch(b) ==
    [chunks |-> b, real |-> PSize(b), own |-> PSize(b), frag |-> FALSE]

(* generic_concatenate (crates/pdata/src/otap/groups.rs 244): join pieces *)
(* greedily, starting a new batch when the next piece would exceed Max.  *)
RECURSIVE Concat(_, _)
Concat(ps, cur) ==
    IF ps = <<>> THEN (IF cur = <<>> THEN <<>> ELSE <<cur>>)
    ELSE LET p == Head(ps) IN
         IF cur # <<>> /\ Max # 0 /\ PSize(cur) + PSize(p) > Max
         THEN <<cur>> \o Concat(Tail(ps), p)
         ELSE Concat(Tail(ps), cur \o p)

(* plan_split (crates/pdata/src/otap/transform/split.rs): cut each input *)
(* so its first piece fills the rest of the current Max-sized bin.        *)
RECURSIVE AlignedPieces(_, _)
AlignedPieces(ps, off) ==
    IF ps = <<>> THEN <<>>
    ELSE LET u == Flatten(Head(ps))
             bl == Blocks(u, Max - (off % Max), Max)
         IN [i \in 1..Len(bl) |-> <<bl[i]>>]
            \o AlignedPieces(Tail(ps), off + Len(u))

(* plan_metrics_split (split.rs 152): cut each input on row boundaries   *)
(* when the next row would exceed Max. Not aligned to the running total; *)
(* a row larger than Max stays whole.                                     *)
RECURSIVE RowSplit(_, _)
RowSplit(rows, cur) ==
    IF rows = <<>> THEN (IF cur = <<>> THEN <<>> ELSE <<cur>>)
    ELSE LET row == Head(rows) IN
         IF cur # <<>> /\ PSize(cur) + Len(row) > Max
         THEN <<cur>> \o RowSplit(Tail(rows), <<row>>)
         ELSE RowSplit(Tail(rows), Append(cur, row))

RECURSIVE RowPieces(_)
RowPieces(ps) ==
    IF ps = <<>> THEN <<>> ELSE RowSplit(Head(ps), <<>>) \o RowPieces(Tail(ps))

ItemBatches(pieces) ==
    LET bs == Concat(pieces, <<>>) IN [i \in 1..Len(bs) |-> ItemBatch(bs[i])]

(* make_bytes_batches_owned (crates/pdata/src/otlp/batching.rs 560).      *)
FlushCur(cur) == IF cur = <<>> THEN <<>> ELSE <<ItemBatch(cur)>>

(* push_resource_entry (185) for an entry larger than Max: split it into *)
(* standalone fragments (split_resource_entry 251, split_scope_entry),   *)
(* then reset ownership to even shares (rescale_ownership 144). A        *)
(* refused split (fb) emits the entry whole.                              *)
(*                                                                         *)
(* The real split packs whole scopes first and only splits an oversize    *)
(* scope into records, so fragment sizes depend on scope and record sizes *)
(* and each fragment repeats either the resource header or the resource   *)
(* and scope headers. With FreeSplit, the model allows every layout: any  *)
(* data block sizes, each fragment with its own overhead in 0..cfg.h, as  *)
(* long as it fits in Max. This includes every layout the code produces.  *)
(* Without FreeSplit, blocks are equal and every overhead is cfg.h.       *)
Whole(e) == <<[chunks |-> <<e>>, real |-> Len(e), own |-> Len(e),
               frag |-> FALSE]>>

Share(total, n, i) == (total \div n) + (IF i <= total % n THEN 1 ELSE 0)

HeaderOf(h) == [j \in 1..h |-> <<0, j>>]

(* A layout is a seq of [c : data units, h : header units]. *)
RECURSIVE FragLayouts(_)
FragLayouts(n) ==
    IF n = 0 THEN {<<>>}
    ELSE UNION {{<<[c |-> kj[1], h |-> kj[2]]>> \o rest :
                    rest \in FragLayouts(n - kj[1])} :
                kj \in {x \in (1..n) \X (0..cfg.h) : x[1] + x[2] <= Max}}

RECURSIVE LayoutOffset(_, _)
LayoutOffset(lay, i) ==
    IF i = 0 THEN 0 ELSE LayoutOffset(lay, i - 1) + lay[i].c

Fragments(e, lay) ==
    [i \in 1..Len(lay) |->
        [chunks |-> <<HeaderOf(lay[i].h)
                      \o SubSeq(e, LayoutOffset(lay, i - 1) + 1,
                                LayoutOffset(lay, i))>>,
         real |-> lay[i].h + lay[i].c,
         own |-> Share(Len(e), Len(lay), i),
         frag |-> TRUE]]

EqualSplit(e) ==
    LET bl == Blocks(e, Max - cfg.h, Max - cfg.h) IN
    Fragments(e, [i \in 1..Len(bl) |-> [c |-> Len(bl[i]), h |-> cfg.h]])

SplitChoices(e, fb) ==
    IF fb THEN {Whole(e)}
    ELSE IF FreeSplit THEN {Fragments(e, lay) : lay \in FragLayouts(Len(e))}
    ELSE IF Max - cfg.h < 1 THEN {Whole(e)}
    ELSE {EqualSplit(e)}

(* The set of possible outputs of make_bytes_batches_owned.               *)
RECURSIVE BytesPack(_, _, _)
BytesPack(es, cur, fb) ==
    IF es = <<>> THEN {FlushCur(cur)}
    ELSE LET e == Head(es) IN
         IF PSize(cur) + Len(e) <= Max
         THEN BytesPack(Tail(es), Append(cur, e), fb)
         ELSE IF Len(e) <= Max
         THEN {FlushCur(cur) \o r : r \in BytesPack(Tail(es), <<e>>, fb)}
         ELSE {FlushCur(cur) \o fr \o r :
                 fr \in SplitChoices(e, fb),
                 r \in BytesPack(Tail(es), <<>>, fb)}

(* The set of possible outputs of Batcher::make_batches (890, 931). Only *)
(* bytes mode with splitting has more than one.                           *)
Batchings(payloads, fb) ==
    IF Mode = "bytes"
    THEN IF Max = 0 THEN {<<ItemBatch(Flatten(payloads))>>}
         ELSE BytesPack(Flatten(payloads), <<>>, fb)
    ELSE IF Max = 0 THEN {ItemBatches(payloads)}
    ELSE IF Mode = "aligned" THEN {ItemBatches(AlignedPieces(payloads, 0))}
    ELSE {ItemBatches(RowPieces(payloads))}

Payloads(b) == [i \in 1..Len(b) |-> b[i].p]

-----------------------------------------------------------------------------
(* State transformers. St is the processor state (plus ghosts) as one   *)
(* record so that a single message can be modeled as a chain of calls.   *)

St == [buf |-> buf, live |-> live, cnt |-> cnt, outs |-> outs,
       inflight |-> inflight, nAck |-> nAck, nNack |-> nNack,
       confirmed |-> confirmed, lost |-> lost, failed |-> failed]

SetSt(S) ==
    /\ buf' = S.buf /\ live' = S.live /\ cnt' = S.cnt /\ outs' = S.outs
    /\ inflight' = S.inflight /\ nAck' = S.nAck /\ nNack' = S.nNack
    /\ confirmed' = S.confirmed /\ lost' = S.lost /\ failed' = S.failed

Outcome(S, r, ok) ==
    IF ok THEN [S EXCEPT !.nAck[r] = @ + 1] ELSE [S EXCEPT !.nNack[r] = @ + 1]

(* kind is "batch" for a flushed batch or "pt" for a pass-through       *)
(* input; pt is the tracked pass-through request, or NoKey.              *)
Send(S, units, key, kind, pt) ==
    [S EXCEPT !.inflight = @ \cup
        {[kind |-> kind, key |-> key, units |-> units, pt |-> pt]}]

(* handle_partial_responses (1702): per portion, Err removes the slot and *)
(* Nacks; Ok decrements, and at the last reference removes and Acks.      *)
RECURSIVE HandleParts(_, _, _)
HandleParts(ps, ok, S) ==
    IF ps = <<>> THEN S
    ELSE LET r == Head(ps).r
             S1 == IF r \notin S.live THEN S
                   ELSE IF ~ok
                   THEN Outcome([S EXCEPT !.live = @ \ {r}], r, FALSE)
                   ELSE IF S.cnt[r] > 1
                   THEN [S EXCEPT !.cnt[r] = @ - 1]
                   ELSE Outcome([S EXCEPT !.live = @ \ {r}], r, TRUE)
         IN HandleParts(Tail(ps), ok, S1)

(* flush_signal_impl, outbound slots exhausted (1257-1278): every carried *)
(* request whose slot is still live is removed and Nacked.                *)
RECURSIVE NackParts(_, _)
NackParts(ps, S) ==
    IF ps = <<>> THEN S
    ELSE LET r == Head(ps).r IN
         NackParts(Tail(ps),
             IF r \in S.live
             THEN Outcome([S EXCEPT !.live = @ \ {r}], r, FALSE) ELSE S)

(* drain_context (1651): walk the buffered portions in order and assign  *)
(* this batch a window of exactly `w` units. A retained portion hands    *)
(* its reference over instead of taking a new one (1680).                *)
RECURSIVE Drain(_, _, _, _, _, _)
Drain(w, ctx, pos, live0, c, out) ==
    IF w = 0 \/ pos > Len(ctx)
    THEN [ctx |-> ctx, pos |-> pos, cnt |-> c, out |-> out, short |-> w > 0]
    ELSE LET bp == ctx[pos]
             take == Min(bp.w, w)
             nctx == [ctx EXCEPT ![pos].w = bp.w - take, ![pos].ret = FALSE]
             npos == IF bp.w - take = 0 THEN pos + 1 ELSE pos
             k == bp.inkey
             tracked == k # NoKey /\ k \in live0
             nc == IF tracked /\ ~bp.ret THEN [c EXCEPT ![k] = @ + 1] ELSE c
             nout == IF tracked THEN Append(out, [r |-> k]) ELSE out
         IN Drain(w - take, nctx, npos, live0, nc, nout)

(* flush_signal_impl send loop (1236-1290).                               *)
RECURSIVE SendLoop(_, _, _, _)
SendLoop(bs, ctx, pos, S) ==
    IF bs = <<>> THEN S
    ELSE LET b == Head(bs)
             d == Drain(b.own, ctx, pos, S.live, S.cnt, <<>>)
             units == Range(Flatten(b.chunks))
             S1 == [S EXCEPT !.cnt = d.cnt,
                             !.failed = IF d.short THEN @ \cup {"drain_short"}
                                        ELSE @]
             S2 == IF d.out = <<>>
                   THEN Send(S1, units, NoOut, "batch", NoKey)
                   ELSE IF Cardinality(DOMAIN S1.outs) < cfg.outl
                   THEN Send([S1 EXCEPT !.outs = (units :> d.out) @@ @],
                             units, units, "batch", NoKey)
                   ELSE Send(NackParts(d.out, S1), units, NoOut, "batch",
                             NoKey)
         IN SendLoop(Tail(bs), d.ctx, d.pos, S2)

(* flush_signal_impl, batching succeeded (1166-1297), with take_remaining *)
(* (1589) when a size flush leaves a small last batch.                    *)
FlushOk(S, reason, bs) ==
    LET n == Len(bs)
        last == bs[n]
        li == S.buf[Len(S.buf)]
        cand == reason = "size" /\ Max # 0 /\ n > 1          \* 1174-1177
        retain == cand /\ last.real < Lower /\ last.own = last.real  \* 1217
        k == li.inkey
        holds == retain /\ k # NoKey /\ k \in S.live           \* 1632-1637
        newPart == [p |-> last.chunks, inkey |-> k, w |-> last.real,
                    ret |-> holds, src |-> li.src,
                    frag |-> last.frag]                        \* 1627, 1641
        toSend == IF retain THEN SubSeq(bs, 1, n - 1) ELSE bs
        flags ==
            (IF cand /\ Mode = "aligned" /\ bs[1].real < Lower  \* 1189
             THEN {"1189"} ELSE {})
            \cup (IF retain /\ last.real > li.w THEN {"1623"} ELSE {})
            \cup (IF retain /\ \E u \in DataUnits(last.chunks) : u[1] # li.src
                  THEN {"P5"} ELSE {})
            \cup (IF SumOwn(bs) # SumW(S.buf) THEN {"own_sum"} ELSE {})
            \cup (IF \E i \in 1..n : bs[i].own < 1 THEN {"rescale"} ELSE {})
        S1 == [S EXCEPT !.buf = IF retain THEN <<newPart>> ELSE <<>>,
                        !.cnt = IF holds THEN [@ EXCEPT ![k] = @ + 1] ELSE @,
                        !.failed = @ \cup flags]
    IN SendLoop(toSend, S.buf, 1, S1)

(* flush_signal_impl, make_batches failed (1151-1164): Nack every         *)
(* buffered tracked request; the buffered data is dropped.                *)
FlushErr(S) ==
    LET parts == [i \in 1..Len(S.buf) |-> [r |-> S.buf[i].inkey]]
        tracked == SelectSeq(parts, LAMBDA x : x.r # NoKey)
        data == UNION {DataUnits(S.buf[i].p) : i \in DOMAIN S.buf}
    IN HandleParts(tracked, FALSE,
                   [S EXCEPT !.buf = <<>>, !.lost = @ \cup data])

(* flush_signal_impl (1083). An empty buffer is a no-op (1096).          *)
(* bs is the batcher output chosen from Batchings; unused on error.       *)
Flush(S, reason, err, bs) ==
    IF S.buf = <<>> THEN S
    ELSE IF err THEN FlushErr(S)
    ELSE FlushOk(S, reason, bs)

(* Batcher outputs to choose from for a flush of buffer b.               *)
BatchChoices(b, err, fb) ==
    IF err \/ b = <<>> THEN {<<>>} ELSE Batchings(Payloads(b), fb)

ErrChoices == IF AllowBatchError THEN BOOLEAN ELSE {FALSE}
FbChoices == IF Mode = "bytes" /\ AllowFallback THEN BOOLEAN ELSE {FALSE}

PassThrough(w) == w >= Lower /\ (Max = 0 \/ w <= Max)   \* 448

-----------------------------------------------------------------------------
(* Actions. Each is one message handled by BatchProcessor::process.      *)

Init ==
    /\ cfg \in Configs
    /\ buf = <<>>
    /\ live = {}
    /\ cnt = [r \in Requests |-> 0]
    /\ outs = [k \in {} |-> <<>>]
    /\ inflight = {}
    /\ nAck = [r \in Requests |-> 0]
    /\ nNack = [r \in Requests |-> 0]
    /\ confirmed = {}
    /\ lost = {}
    /\ failed = {}
    /\ arrived = {}
    /\ trk = {}
    /\ reqSize = [r \in Requests |-> 0]
    /\ shutdown = FALSE

(* accept_payload (972), reached through process_signal_impl (823).      *)
AcceptPayload ==
    /\ ~shutdown
    /\ \E r \in Requests \ arrived, c \in Shapes, t \in BOOLEAN,
          err \in ErrChoices, fb \in FbChoices :
        LET p == MakePayload(r, c)
            w == PSize(p)
            S == St
            buffered ==
                [S EXCEPT
                    !.buf = Append(@, [p |-> p,
                                       inkey |-> IF t THEN r ELSE NoKey,
                                       w |-> w, ret |-> FALSE, src |-> r,
                                       frag |-> FALSE]),
                    !.live = IF t THEN @ \cup {r} ELSE @,
                    !.cnt = IF t THEN [@ EXCEPT ![r] = 0] ELSE @]
            flushes ==
                /\ w # 0 /\ ~PassThrough(w)
                /\ ~(t /\ Cardinality(S.live) >= cfg.inl)
                /\ SumW(buffered.buf) >= Lower
            S1(bs) ==
                IF w = 0                                     \* 979-987
                THEN (IF t THEN Outcome(S, r, TRUE) ELSE S)
                ELSE IF PassThrough(w)                       \* 991-993
                THEN Send(S, Range(Flatten(p)), NoOut, "pt",
                          IF t THEN r ELSE NoKey)
                ELSE IF t /\ Cardinality(S.live) >= cfg.inl  \* 1008-1020
                THEN Outcome(S, r, FALSE)
                ELSE IF SumW(buffered.buf) >= Lower          \* 1041-1058
                THEN Flush(buffered, "size", err, bs)
                ELSE buffered
        IN \E bs \in IF flushes THEN BatchChoices(buffered.buf, err, fb)
                     ELSE {<<>>} :
           /\ SetSt(S1(bs))
           /\ arrived' = arrived \cup {r}
           /\ trk' = IF t THEN trk \cup {r} ELSE trk
           /\ reqSize' = [reqSize EXCEPT ![r] = w]
           /\ UNCHANGED <<cfg, shutdown>>

(* Wakeup -> flush_signal_impl(Timer) (1424-1445). Clocks are abstracted *)
(* away: a timer flush may happen whenever data is buffered.              *)
TimerFlush ==
    /\ buf # <<>>
    /\ \E err \in ErrChoices, fb \in FbChoices :
        \E bs \in BatchChoices(buf, err, fb) :
            SetSt(Flush(St, "timer", err, bs))
    /\ UNCHANGED <<cfg, arrived, trk, reqSize, shutdown>>

(* Shutdown -> flush_shutdown (756). No input is accepted afterwards.    *)
Shutdown ==
    /\ ~shutdown
    /\ shutdown' = TRUE
    /\ \E err \in ErrChoices, fb \in FbChoices :
        \E bs \in BatchChoices(buf, err, fb) :
            SetSt(Flush(St, "shutdown", err, bs))
    /\ UNCHANGED <<cfg, arrived, trk, reqSize>>

(* Downstream answers one in-flight batch: handle_ack / handle_nack       *)
(* (1320-1368) -> handle (1300). A pass-through batch is answered to its  *)
(* own sender and never reaches the processor's slot maps.                *)
Respond ==
    /\ \E b \in inflight, ok \in BOOLEAN :
        LET S0 == [St EXCEPT !.inflight = @ \ {b},
                             !.confirmed = IF ok THEN @ \cup b.units ELSE @,
                             !.lost = IF ok THEN @ ELSE @ \cup b.units]
            S1 == IF b.kind = "pt"
                  THEN (IF b.pt = NoKey THEN S0 ELSE Outcome(S0, b.pt, ok))
                  ELSE IF b.key # NoOut /\ b.key \in DOMAIN S0.outs
                  THEN HandleParts(S0.outs[b.key], ok,
                                   [S0 EXCEPT !.outs = Remove(@, b.key)])
                  ELSE S0
        IN SetSt(S1)
    /\ UNCHANGED <<cfg, arrived, trk, reqSize, shutdown>>

Next == AcceptPayload \/ TimerFlush \/ Shutdown \/ Respond

Spec == Init /\ [][Next]_vars

-----------------------------------------------------------------------------
(* Properties. See spec.md for their intent and code traces.             *)

TypeOK ==
    /\ live \subseteq Requests
    /\ cnt \in [Requests -> Nat]
    /\ nAck \in [Requests -> Nat] /\ nNack \in [Requests -> Nat]
    /\ Cardinality(live) <= cfg.inl
    /\ Cardinality(DOMAIN outs) <= cfg.outl

(* P1. No premature Ack. *)
NoPrematureAck ==
    \A r \in Requests : nAck[r] > 0 => UnitsOf(r) \subseteq confirmed

(* P2. At most one outcome per request. *)
AtMostOneOutcome ==
    \A r \in Requests : nAck[r] + nNack[r] <= 1

(* P3. Exact reference counts. *)
PortionsIn(s, r) == Cardinality({i \in DOMAIN s : s[i].r = r})
Retained(r) == Cardinality({i \in DOMAIN buf : buf[i].inkey = r /\ buf[i].ret})

ExactCounts ==
    \A r \in live :
        cnt[r] = SumF([k \in DOMAIN outs |-> PortionsIn(outs[k], r)],
                      DOMAIN outs)
                 + Retained(r)

(* P4. A tracked request is never silently dropped: it is live, has an   *)
(* outcome, or is a pass-through batch awaiting downstream.               *)
NoSilentDrop ==
    \A r \in arrived \cap trk :
        \/ r \in live
        \/ nAck[r] + nNack[r] >= 1
        \/ \E b \in inflight : b.pt = r

(* P5. A residual comes only from the last input. *)
ResidualFromLastInput == failed \cap {"1623", "P5"} = {}

(* P6. Drained means empty. *)
DrainedMeansEmpty ==
    (buf = <<>> /\ inflight = {}) =>
        /\ live = {}
        /\ DOMAIN outs = {}
        /\ \A r \in arrived \cap trk : nAck[r] + nNack[r] = 1

(* P7. Guards never fail. 442, 1598-1600 and 1664 hold by construction  *)
(* in this model (min_size is always set; retention needs n > 1 and a    *)
(* non-empty buffer; Drain checks pos before indexing).                  *)
GuardsHold == failed \cap {"1189", "1623", "rescale"} = {}

(* Model sanity: ownership weights add up to the buffered weight, and    *)
(* drain_context never runs out of portions.                              *)
OwnershipConsistent == failed \cap {"own_sum", "drain_short"} = {}

(* P8. Size-0 and pass-through requests never touch the slot maps. *)
NewSize0OrPassThrough ==
    \E r \in arrived' \ arrived :
        reqSize'[r] = 0 \/ PassThrough(reqSize'[r])

PassThroughAnswered ==
    \E b \in inflight \ inflight' : b.kind = "pt"

SlotsUntouched == live' = live /\ cnt' = cnt /\ outs' = outs

SlotFreePaths ==
    [][(NewSize0OrPassThrough \/ PassThroughAnswered) => SlotsUntouched]_vars

=============================================================================
