# OTAP Dataflow Engine Controller

A controller takes a pipeline configuration and initiates one dataflow engine
per core (or less if the number of CPUs or the percentage of CPUs is
specified).

Each engine is started on a dedicated CPU core (via thread pinning).

## Roadmap

- [ ] Basic controller that can start/stop engines (stop is not implemented yet)
- [X] Support for metrics collection and aggregation
- [ ] HTTP admin interface to
  - [X] view telemetry
  - [ ] manager pipelines
- [ ] NUMA awareness and support for pinning engines to specific NUMA nodes
- [ ] Support for multiple pipeline groups
- [ ] Support for dynamic reconfiguration of pipelines
- [ ] Support for high availability and failover
- [ ] Support for logging and tracing
- [ ] Support for custom plugins and extensions
- [ ] Support for configuration validation and schema management
