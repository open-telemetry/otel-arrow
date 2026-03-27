# Experimental control-aware bounded channel prototype

This crate contains a standalone prototype for a control-optimized bounded
channel intended for OTAP dataflow control traffic.

The prototype is not integrated into the engine runtime yet. It exists to make
the queue policy testable in isolation before deciding whether and how to wire
it into node-control delivery.

Current behaviors include:

- reserved lifecycle delivery for `DrainIngress` and `Shutdown`
- batched `Ack` and `Nack` delivery
- deduplicated `TimerTick` and `CollectTelemetry`
- last-write-wins `Config`
- bounded retained `DelayedData`

It exposes both local and shared variants with matching semantics so the queue
behavior can be evaluated independently of engine integration.
