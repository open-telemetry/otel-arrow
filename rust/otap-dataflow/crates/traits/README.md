# otap-df-traits

Core traits for the OTAP (OpenTelemetry Arrow Protocol) dataflow system.

## Purpose

This crate defines the fundamental traits that delineate the interfaces for various components in the OTAP dataflow pipeline. By maintaining these traits in a distinct crate, we eliminate circular dependencies and promote a clear separation between interface definitions and their implementations.

## Key Characteristics

- **Dependency Clarity**: Solely consists of traits with no external dependencies.
- **No Circular Dependencies**: Traits are maintained independently and imported as needed.
- **Clean Separation**: Focused on interface definitions, detached from concrete implementations.
- **Testability**: Facilitates testing by allowing any implementation of the traits without complex dependencies.

## Included Traits

### Retryable

Defines capabilities for data items that can be retried within the pipeline, emphasizing cloneability, thread safety, and a stable lifetime.

#### Required Methods

- `id()`: Returns a unique, deterministic identifier for ACK/NACK correlation.
- `deadline()`: Optional method to establish a processing deadline, enhancing deadline-aware strategies.

