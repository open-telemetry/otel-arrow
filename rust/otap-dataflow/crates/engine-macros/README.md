# Pipeline Engine Macros

The linked-based plugin system requires the creation of a pipeline factory for
each type of PData. The creation of these factories is managed by the
`pipeline_factory` macro, which allows declaring factories for specific PData
types.

The following declaration is an example of a factory declaration for the
`OTAPData` type:

```rust
#[pipeline_factory(OTAP, OTAPData)]
static OTAP_PIPELINE_FACTORY: PipelineFactory<OTAPData> = build_factory();
```
