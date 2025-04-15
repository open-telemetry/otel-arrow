# Project Roadmap

This document outlines the planned features and milestones for this project.

---

## Current Status

**Status:** Work-In-Progress

---

## Milestones

For the moment, these milestones are very high-level and are subject to
significant changes.

### CI/CD

- [ ] Github action setup
- [ ] Linting (Clippy)
- [ ] Formatting
- [ ] Unit tests
- [ ] Integration tests
- [ ] Code coverage (target: 80%)
- [ ] Documentation generation
- [ ] Continuous benchmarking and performance tracking
  - [ ] Comparison with Go Collector
  
### Dataflow Engine Foundations

- [ ] Receiver, Processor, and Exporter traits
- [ ] Dataflow Engine
- [ ] Initial benchmarks and documentation

### OTLP Dataflow

- [ ] OTLP Receiver, Batch Processor, and OTLP Exporter
- [ ] Comparison with Go OTLP Pipeline

### OTAP Dataflow

- [ ] OTAP Message
- [ ] OTAP Receiver, Batch Processor, and OTLP Exporter
- [ ] Comparison with Go OTLP Pipeline

### Dataflow Engine Advanced Features

- [ ] Admission control
- [ ] Failover and Retry
- [ ] Live reconfiguration
- [ ] Backpressure
- [ ] Shutdown
- [ ] Memory usage control
- [ ] Acknowledgment and guarantee of delivery

---

## Future Plans (Ideas & Discussions)

- [ ] KQL Processor
- [ ] OTTL Processor
- [ ] Integration with Go Collector

---

## Completed Milestones

- ✅ Basic project structure setup
- ✅ Creation of xtask to check project conformity against the controls we want
  to systematically apply.
- ✅ MPMC Channel implementation
- ✅ MPSC Channel implementation

---

## Contributing

If you would like to suggest features, discuss improvements, or contribute
directly, please open an issue or PR!

---