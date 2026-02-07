# Validation

The validation test validates a OTLP or OTAP messages after
experiencing various processes such as encoding/decoding
or going through a pipeline.

## Encoding/Decoding Validation

To validate whether encoding/decoding is working properly
we comparing the input and output to check that they are equivalent.

## Pipeline Validation

To validate pipelines we create a pipeline group that has three pipelines:

- `traffic-gen` generates traffic for the validation.
- `suv` is the system under validation (the pipeline being tested).
- `validate` compares control vs. suv outputs using the validation exporter.

The framework now prefers programmatic scenarios defined in tests; it handles
wiring ports and running the group end-to-end.

### How to validate your pipelines

You can define scenarios directly inside your Rust tests by utilizing the
validation framework.

- `Pipeline`: loads a pipeline YAML and lets you wire logical endpoints
(receiver/exporter) that will be rewritten to free ports for each test run.
- `Scenario`: orchestrates the end-to-end run: rewires the pipeline, spins up
the validation group, drives traffic, waits for metrics, and returns Ok on success

Example:

```rust
use otap_df_validation::{pipeline::Pipeline, scenario::Scenario, traffic};
use std::time::Duration;

#[test]
fn no_processor() {
    Scenario::new()
        .pipeline(
            Pipeline::from_file("./validation_pipelines/no-processor.yaml")
                .expect("failed to read in pipeline yaml")
                .wire_otlp_grpc_receiver("receiver")
                .wire_otlp_grpc_exporter("exporter"),
        )
        .input(traffic::Generator::logs().fixed_count(500).otlp_grpc())
        .observe(traffic::Capture::default().otlp_grpc())
        .expect_within(Duration::from_secs(140))
        .run()
        .expect("validation scenario failed");
}
```

The wired nodes (e.g., `receiver`, `exporter`) are automatically rewritten to
free ports by the framework.

## Future directions

- Extend the validation exporter to support more complex validation procedure.
