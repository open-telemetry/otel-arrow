# Design Principles and Constraints

## Design Principles

- **Design Complexity and Predictability**: Prioritize designs patterns that
  minimize complexity and enhance predictability in system behavior. Opt for
  solutions that are easier to understand, debug, and maintain.
- **Manage User-Facing Complexity**: The user-facing design should allow for
  advanced used to have fine-grain and nuanced management concepts but not force
  less advanced users or simple use-cases to be aware of them.
- **Scalability**: Optimize for modern multi-core systems and, when present and
  appropriate, specialized hardware (e.g. SIMD). Ensure designs support both
  vertical and horizontal scaling.
- **Stability Under Load/Graceful Degradation**: Ensure stability under heavy
  load by employing backpressure or shedding load when downstream systems are
  slow or backlogged. This prevents memory blowups and latency spikes.
- **Minimal Overhead in Synchronization**: Reduce excessive lock contention and
  avoid complex scheduling. Use techniques like CPU affinity and localized data
  structures to improve cache locality and minimize jitter.
- **Minimal Overhead in Context Switching**: Limit unnecessary system calls to
  reduce context switches. Leverage event-driven design patterns, asynchronous
  runtimes and non-blocking IO to minimize latency, improve throughput, and
  enhance scalability.
- **Minimal Memory Allocation**: Utilize zero-copy mechanisms (e.g. between user
  space and kernel space or buffers) and avoid excessive memory allocations.
  This minimizes CPU usage, reduces memory bandwidth bottlenecks, and enhances
  throughput.
- **Serviceability**: Design systems to support live reconfiguration and
  upgrades without service interruption, while maintaining scalability.
- **Modularity, Extensibility and Plugin Architecture**: Enable the system to be
  easily extended and new capabilities exposed, without modifying the core
  system, either by the core team or by third parties
- **Programmability**: The system should provide in-situ programming
  capabilities so that users are able to customize the processing for their
  specific needs, without resorting to plug-ins.
- **Resource Observability and Autoscaling**: Monitor resource usage in
  real-time to enable automated scaling decisions. Support both horizontal and
  vertical scaling.
- **Debuggability**: Design with functional and performance debuggability as a
  primary consideration, covering both developer and user use-case
  scenarios. Use statistics, logs, and traces as appropriate, with controls to
  selectively enable or disable them. Have a means to bootstrap debug locally,
  for cases where the failure mode is the network stack.
- **Memory Safety**: Favor programming languages and techniques that mitigate or
  eliminate memory-related issues to improve security and stability.
- **Continuous Integration and Continuous Benchmarking**: Incorporate robust
  CI/CD pipelines and benchmarking practices to ensure ongoing quality and
  performance improvements.
- **Security First**: Make security a fundamental consideration in every design
  and implementation decision.
- **Privacy First**: Design with user privacy and data protection as fundamental
  priorities.

## Constraints

- **Architectures**: Provide excellent support for x86/x86-64 and ARM
  architectures.
- **Operating Systems**: Provide excellent support for Linux and Windows with
  reasonable support for macOS.
- **Programming Language**: Implement the system core in Rust to leverage its
  performance and memory safety guaranties.
- **OpenTelemetry Compatibility**: Design the system to seamlessly integrate
  with the existing Go-based OTEL Collector.
- **Data Processing Pipelines**: Design the pipeline engine to natively and
  efficiently support OTLP and OTAP pipelines.
- **Configurable Quality of Service**: Handle telemetry data streams with
  optional loss tolerance, ensuring consistent data quality. The pipeline engine
  must be able to support deployments where all incoming requests are processed
  without discarding or dropping data.
