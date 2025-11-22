# Attack Vectors and Mitigations

## Scope and Thread Model

This document summarizes the security analysis of the OTEL Receiver component
in the OTAP Dataflow engine. The receiver implements OTLP and OTAP Arrow over a
custom H2 gRPC server. It runs inside a single-threaded runtime per core,
avoids Send and Sync, avoids Arc and Mutex, and minimizes heap allocations.
Because it is internet-facing, it must defend against malformed H2 messages,
flow control abuse, gRPC-level resource exhaustion, compression bombs, and
protocol downgrade attacks.

The component processes untrusted network traffic, including arbitrary HTTP/2
frames, arbitrary gRPC frames, compressed payloads, and long-lived streaming
connections. Therefore, an attacker with network access is assumed capable of
sending intentionally malformed or adversarial payloads.

## Summary of Major Attack Vectors and Mitigations

The table below summarizes each category:

| Vector                                     | Description                                                                   | Mitigation                                                     |
| ------------------------------------------ | ----------------------------------------------------------------------------- | -------------------------------------------------------------- |
| Oversized gRPC frames                      | Huge declared length in 5-byte gRPC prefix that forces unlimited buffering    | Enforced global `max_decoding_message_size` and rejected early |
| Compression bombs                          | zstd, gzip, deflate payloads inflating to large memory footprint              | Decompressors capped at `max_decoding_message_size`            |
| HTTP/2 flow control bypass                 | Releasing capacity on read (not consume) allowed unlimited inflight buffering | Release only on actual consumption, reactivating flow control  |
| Slowloris H2 handshake                     | Idle or extremely slow H2 prefaces tying up unlimited connection slots        | Handshake timeout                                              |
| Infinite idle H2 connections               | Long-lived connections with no active streams                                 | Keepalive and idle cutoff logic                                |
| Ack registry exhaustion                    | Leaked registry slots during pipeline errors                                  | Slots cancelled when enqueue fails                             |
| CPU starvation inside reactor              | Long decode or decompress without preemption                                  | Hard size limits and bounded internal buffers                  |
| Protocol downgrade or invalid content-type | Invalid gRPC Content-Type or compression headers                              | Strict header validation and early rejection                   |
| Unknown or unapproved compression methods  | "zstdarrowX", raw zstd variants, unknown tokens                               | Strict whitelist matching `AcceptedGrpcEncodings`              |
| Stream flooding                            | Opening too many H2 streams                                                   | Per-connection stream admission and `max_in_flight limit`      |
| Pipeline overload                          | Downstream queue saturation                                                   | Ack registry backpressure and overloaded_status                |
| Malformed Arrow data                       | Invalid protobuf or Arrow content stalling stream                             | Immediate error, closing stream safely                         |
