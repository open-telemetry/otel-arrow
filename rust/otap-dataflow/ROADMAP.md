# Project Roadmap

This document outlines the planned features and milestones for this project.

---

## Current Status

**Status:** Work-In-Progress

---

## Milestones

For the moment, these milestones are very high-level and are subject to
significant changes.

### CI/CD Setup

- [ ] Github action setup
- [ ] Linting (Clippy)
- [ ] Formatting
- [ ] Unit tests (Nextest)
- [ ] Integration tests
- [ ] Code coverage (target: 80%)
- [ ] Documentation generation
- [ ] Continuous benchmarking and performance tracking
  - [ ] Comparison with Go Collector

### Pipeline Engine Foundations (phase 1)

- Channels
  - [x] MPMC Channel
  - [x] MPSC Channel
- [x] Receiver trait
- [x] Processor trait
- [x] Exporter trait
- [x] EffectHandler trait
- [ ] Connector trait
- [ ] CPU & Memory Benchmarks
- [ ] Documentation

### Pipeline Engine Foundations (phase 2)

- Channels
  - [ ] SPSC Channel
  - [ ] Broadcast Channel
- [ ] Pipeline Engine
- [ ] Instrumentation
- [ ] Thread pinning
- Benchmarks
  - [ ] CPU benchmarks
  - [ ] Memory benchmarks
- [ ] CPU & Memory Benchmarks
- [ ] Documentation

### OTLP Pipeline

- [ ] OTLP Receiver, Batch Processor, and OTLP Exporter
- [ ] Comparison with Go OTLP Pipeline

### OTAP Pipeline

- [ ] OTAP Message
- [ ] OTAP Receiver, Batch Processor, and OTLP Exporter
- [ ] Comparison with Go OTLP Pipeline

### Pipeline Engine Advanced Features

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

- :white_check_mark: Project structure setup and xtask to check project conformity

---

## Contributing

If you would like to suggest features, discuss improvements, or contribute
directly, please open an issue or PR!

---
