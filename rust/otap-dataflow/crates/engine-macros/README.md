# otel-arrow-dfe-engine-macros

This crate is currently pre-1.0. Its public API may evolve between minor
releases.

The linked-based plugin system requires the creation of a pipeline factory for
each type of PData. The creation of these factories is managed by the
`pipeline_factory` macro, which allows declaring factories for specific PData
types.

The following declaration is an example of a factory declaration for an
`OTAPData` type that supports runtime context materialization:

```rust
#[pipeline_factory(OTAP, OTAPData, OTAPData::materialize_context)]
static OTAP_PIPELINE_FACTORY: PipelineFactory<OTAPData> = build_factory();
```

The optional third argument is a function path with the signature
`fn(&mut PData, &ContextMaterializationPlan)`. It registers how the PData type
stores selected constant and randomness composites. The two-argument form
remains valid for PData types that do not support materialization, but pipeline
construction fails if their configuration selects a materialized composite.

## Usage

```sh
cargo add otel-arrow-dfe-engine-macros
```

Most consumers receive these macros through `otel-arrow-dfe-engine`. Depend on
this crate directly only when implementing engine plugins.
