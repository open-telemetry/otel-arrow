# Load Balancing: Challenges & Solutions

## Context

OpenTelemetry Protocol with Apache Arrow (OTAP) maintains per-stream state (schemas, dictionaries, other compression metadata) to reduce wire size; longer-lived streams generally compress better but accumulate memory and require coordination when recycled.

In thread-per-core architectures using `SO_REUSEPORT`, the kernel creates an independent accept queue per listening socket and selects a socket based on a hash of the connection 4-tuple (source/destination IP, source/destination port). This approach reduces accept contention but makes load distribution dependent on connection diversity.

Since gRPC multiplexes multiple logical calls (including bidirectional streaming RPCs) over a single HTTP/2 (and thus single TCP) connection, too few client connections can cause traffic to concentrate on a single reuseport bucket (one core) behind an L4 balancer. Fine-grained distribution requires either L7 (HTTP/2-aware) load balancing or deliberate client fan-out.

Addressing these interactions is crucial for reliable scalability. This document outlines the associated challenges, trade-offs, and practical mitigations for both exporters and servers.


## Key Challenges

| #   | Challenge                               | Symptoms                                                                                                                       | Root Cause                                                                                                                                                                 |
| --- | --------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | **Skewed listener utilization**         | All (or most) exporter TCP connections land on the same reuseport listener => one core saturated, others idle                  | Too few distinct TCP 4-tuples (e.g. single long-lived gRPC channel) means the kernel’s reuseport hash has little entropy; distribution collapses statistically.            |
| 2   | **In-stream vs connection imbalance**   | Recycling an OTAP *stream* clears state but stays on the same TCP connection, so work remains pinned to the same listener/core | gRPC stream lifetime ≠ TCP connection lifetime; reuseport selection happens once per connection handshake. To move work you must rotate the connection or rebalance at L7. |
| 3   | **Dictionary / stream-state growth**    | High-cardinality attributes inflate Arrow dictionary & per-stream memory                                                       | Stateful OTAP encoding accumulates per-stream dictionaries until recycled or bounded by receiver memory/admission limits.                                                  |
| 4   | **Single accept listener anti-pattern** | One thread does all `accept()` => CPU hotspot, lock contention, cache/NUMA misses, global back-pressure                        | Shared accept queues can become contended and distribute unevenly; `SO_REUSEPORT` (per-socket queues) improves scalability but has fairness/latency trade-offs.            |


**Table notes:**
- Reuseport hashing distributes *connections*, not individual gRPC streams; ensure enough connections for statistical balance. See [The SO_REUSEPORT socket option - LWN.net](https://lwn.net/Articles/542629/) and [Performance best practices with gRPC - Microsoft Learn](https://learn.microsoft.com/en-us/aspnet/core/grpc/performance)
- Consider tail-latency implications when moving from a shared accept queue to per-socket queues. See [Why does one NGINX worker take all the load?](https://blog.cloudflare.com/the-sad-state-of-linux-socket-balancing/) and [Perfect locality and three epic SystemTap scripts](https://blog.cloudflare.com/perfect-locality-and-three-epic-systemtap-scripts/)


## Solution Space

### 1. Client‑Side Techniques (Exporter)

| Technique                                                                | Purpose                                                                                                                                                                                                                           | Considerations                                                                                                                        |
| ------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| **Increase connection fan-out** (`num_streams` / multiple gRPC channels) | Create multiple concurrent TCP connections so reuseport has entropy; improves distribution and concurrency headroom. Start near `max(1, CPUs/2)` (current `otelarrowexporter` default) and tune; cap for TLS / resource overhead. | More channels consume sockets, TLS handshakes, and exporter CPU; too many can hurt compression efficiency because state is sharded.   |
| **Client-side gRPC load-balancing policy** (`round_robin`, etc.)         | Ensure channels spread across backend endpoints (or front-end L7 proxies) instead of `pick_first` pinning. OTEL-Arrow exporter defaults to `round_robin`.                                                                         | Requires name resolution / endpoint list; risk of thundering herd on endpoint change if misconfigured.                                |
| **Stream lifetime control** (`max_stream_lifetime`)                      | Periodically recycle OTAP streams to bound dictionary growth and help downstream rebalancers shed load; default 30s in current Go exporter (tunable).                                                                             | Recycling forces resending schemas/dictionaries; shorter lifetimes reduce compression efficiency; coordinate with server `keepalive`. |
| **Back-pressure aware retries**                                          | Honor exporter/receiver back-pressure signals so clients can open additional channels or shed load when a stream saturates.                                                                                                       | Requires instrumentation and retry logic; careless retries can amplify load.                                                          |

> **Important note:** Do not trust uncooperative or malicious clients to recycle streams or limit dictionary growth => enforce server-side caps.


### 2. Server-Side Techniques

#### 2.a. Custom `SO_REUSEPORT` BPF Selection

An eBPF program (`SO_ATTACH_REUSEPORT_EBPF` / `BPF_PROG_TYPE_SK_REUSEPORT`) can be attached to influence kernel listener selection within a reuseport group. Practical uses include adding randomization, weighting sockets, or leveraging additional header information to mitigate distribution skew.

#### 2.b. Front-End **L7 (HTTP/2-aware) Load Balancer**

Deploying an HTTP/2-aware proxy (e.g. Envoy, NGINX) that terminates HTTP/2/TLS and creates new backend connections enables finer distribution of individual gRPC calls/streams, effectively addressing L4 connection pinning. This is the recommended approach for achieving balanced load distribution of long-lived streaming RPCs.

> **Tip:** If you already run a thread-per-core backend behind the proxy, you typically expose *one* service port and rely on reuseport behind the proxy; per-core port sharding is rarely needed.

#### 2.c. eBPF **SK_MSG / SOCKMAP** Stream-Level Experiments (Advanced)

Sockmap/SK_MSG programs can redirect application payloads between established sockets in the kernel and are used in advanced in-kernel proxies and service meshes. In theory we could parse (plaintext) HTTP/2 frames and steer them across worker sockets, but doing so—especially through TLS—is complex and typically unnecessary if an L7 proxy is available. Treat as research / last resort.


## Recommended Baseline Configuration

1. **Per-CPU listener sockets:** Use `SO_REUSEPORT` (one service port; one socket per core/worker) to reduce accept contention and improve CPU locality.  
2. **Exporter connection fan-out:** Begin with `num_streams ≈ max(1, CPUs/2)` (current default); monitor and adjust upward if skew persists and resources permit.  
3. **Stream recycling:** Start with `max_stream_lifetime` set between 30 seconds and 2 minutes; lengthen this interval for improved compression if memory allows, or shorten it to rebalance faster or respect proxy keepalive constraints. Align with server-side settings like `keepalive` and `max_connection_age`.  
4. **Observability:** Monitor per-listener connection counts, QPS, latency, and compression efficiency; alert when deviation exceeds acceptable thresholds (e.g. >20% from median) aligned with your Service Level Objectives (SLOs).


## Future Work

* Adaptive autotuning of stream reset intervals and connection fan-out based on live operational metrics.
* Prototype & benchmark eBPF hash strategies.
* Investigate **hot-stream migration** without a full TCP reconnect (protocol extension): explore methods for shifting active OTAP gRPC streams to new listeners/cores, allowing load rebalancing without forcing exporters to reconnect or resend Arrow dictionaries.


## Appendix - Why *Not* a Single Accept Listener?

* **CPU & lock contention:** A shared accept queue forces synchronization among workers, potentially skewing load distribution particularly when combined with epoll’s LIFO behavior, which preferentially feeds the busiest worker.
* **Locality & scalability:** Per-socket queues improve packet locality and reduce cross-CPU bouncing, aiding multicore scaling (NUMA). 
* **Extra syscalls:** Passing accepted sockets between workers introduces unnecessary syscall overhead.
* **Global back‑pressure**: one slow worker stalls all new connections.
* **Redundant engineering:** Duplicates functionality already efficiently provided by `SO_REUSEPORT`.
* **Failure risk**: single point of failure—if the acceptor crashes, all new connections stall.


## References & Further Reading

- [Linux `socket(7)` manual — `SO_REUSEPORT`, `SO_ATTACH_REUSEPORT_EBPF` options](https://man7.org/linux/man-pages/man7/socket.7.html)  
- [The SO_REUSEPORT socket option* (LWN.net, Kerrisk)](https://lwn.net/Articles/542629/)  
- [NGINX `listen ... reuseport` directive (socket sharding)](https://nginx.org/en/docs/http/ngx_http_core_module.html)  
- [Why does one NGINX worker take all the load? (Cloudflare) — accept queue models & balancing](https://blog.cloudflare.com/the-sad-state-of-linux-socket-balancing/)  
- [Perfect locality and three SystemTap scripts (Cloudflare) — reuseport & locality effects](https://blog.cloudflare.com/perfect-locality-and-three-epic-systemtap-scripts/)  
- [Performance Optimisation using SO_REUSEPORT* (Marten Gartner)](https://medium.com/high-performance-network-programming/performance-optimisation-using-so-reuseport-c0fe4f2d3f88)  
- [eBPF-Powered Load Balancing for SO_REUSEPORT](https://medium.com/all-things-ebpf/ebpf-powered-load-balancing-for-so-reuseport-30acb395e1d6)  
- [eBPF Program Type `SK_MSG` & Sockmap redirection docs](https://docs.ebpf.io/linux/program-type/BPF_PROG_TYPE_SK_MSG/), [BPF_MAP_TYPE_SOCKMAP and BPF_MAP_TYPE_SOCKHASH - Linux Kernel Docs](https://docs.kernel.org/bpf/map_sockmap.html)  
- [gRPC Performance Best Practices — L4 vs L7 load balancing; streaming pinning - Microsoft Learn](https://learn.microsoft.com/en-us/aspnet/core/grpc/performance)  
- [gRPC Custom Load Balancing Policies](https://grpc.io/docs/guides/custom-load-balancing/)  
- [Envoy gRPC proxying & load-balancing overview](https://www.envoyproxy.io/docs/envoy/latest/intro/arch_overview/other_protocols/grpc)  
- [Introducing gRPC Support with NGINX](https://blog.nginx.org/blog/nginx-1-13-10-grpc)  
- [OTel-Arrow in Production (memory limits, tuning, back-pressure)](https://opentelemetry.io/blog/2024/otel-arrow-production/)  
- [Apache Arrow Columnar Format & Dictionary Encoding spec](https://arrow.apache.org/docs/format/Columnar.html)  
- [OTAP Phase-2 Announcement (Rust / thread-per-core direction)](https://opentelemetry.io/blog/2025/otel-arrow-phase-2/)