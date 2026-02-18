# Validation Framework

End-to-end harness for standing up a **system-under-validation (SUV)** pipeline, driving OTLP/OTAP traffic into it, capturing the output, and asserting invariants. 

## Framework components
- `Scenario`: orchestrates end-to-end runs (render → run → validate).
- `Pipeline`: loads your SUV pipeline YAML and overrides endpoints.
- `Generator` / `Capture`: configure traffic emission and capture/validation.

## Quick setup (end to end)
1) **Author your SUV pipeline YAML** (the thing you want to validate). Use logical node names for the receiver/exporter you intend to rewire, e.g. `receiver`, `exporter`.
2) **Wire it dynamically** in the test:
   ```rust
   use otap_df_validation::pipeline::Pipeline;

   let pipeline = Pipeline::from_file("./validation_pipelines/your_pipeline.yaml")
       .expect("load pipeline")
       .wire_otlp_grpc_receiver("receiver")   // node name in your YAML
       .wire_otlp_grpc_exporter("exporter");  // node name in your YAML
   ```
3) **Configure traffic generation**:
   ```rust
   use otap_df_validation::traffic::Generator;

   let generator = Generator::logs()                // logs(), metrics(), traces()
       .fixed_count(500)                           // total signals to emit
       .max_batch_size(50)                         // optional
       .otlp_grpc();                               // or .otap_grpc()
   ```
   Available knobs on `Generator`:
   - `logs()`, `metrics()`, `traces()` choose what signal type to emit
   - `fixed_count(usize)` sets max signals to emit before completion (optional; defaults to 2000).
   - `max_batch_size(usize)` controls batch size (optional; defaults to 100).
   - `otlp_grpc()` / `otap_grpc()` choose export protocol (optional; OTLP by default).

4) **Configure capture & validations**:
   ```rust
   use otap_df_validation::traffic::Capture;
   use otap_df_validation::ValidationInstructions;
   use otap_df_validation::validation_types::attributes::{AttributeDomain, KeyValue, AnyValue};

   let capture = Capture::default()
       .otlp_grpc() // or .otap_grpc()
       .validate(vec![
           ValidationInstructions::Equivalence, // control vs SUV outputs match
           ValidationInstructions::SignalDrop {
               min_drop_ratio: Some(0.2),
               max_drop_ratio: Some(0.8),
           },
           ValidationInstructions::AttributeRequireKeyValue {
               domains: vec![AttributeDomain::Signal],
               pairs: vec![KeyValue::new(
                   "ios.app.state".into(),
                   AnyValue::String("active".into()),
               )],
           },
       ]);
   ```

5) **Build and run the scenario**:
   ```rust
   use otap_df_validation::scenario::Scenario;
   use std::time::Duration;

   Scenario::new()
       .pipeline(pipeline)
       .input(generator)
       .observe(capture)
       .expect_within(Duration::from_secs(200)) // optional timeout; default 140 
       .run()
       .expect("validation scenario failed");
   ```



## Pipeline
- **Pipeline example**
  ```rust
  use otap_df_validation::pipeline::Pipeline;

  let pipeline = Pipeline::from_file("./validation_pipelines/your_pipeline.yaml")
      .expect("load pipeline")
      .wire_otlp_grpc_receiver("receiver")   // rewires protocols.grpc.listening_addr
      .wire_otlp_grpc_exporter("exporter");  // rewires grpc_endpoint
  ```
- `Pipeline::from_file(path)` / `from_yaml(str)` – load the SUV pipeline YAML.
- `wire_otlp_grpc_receiver(node_name)` – mark the node whose `protocols.grpc.listening_addr` will be rewritten.
- `wire_otlp_grpc_exporter(node_name)` – mark the exporter whose `grpc_endpoint` will be rewritten.
- `wire_otap_grpc_receiver(node_name)` / `wire_otap_grpc_exporter(node_name)` – OTAP variants.

> NOTE: The node names you pass to `wire_*` must match the keys under `nodes:` in your pipeline YAML.

## Generator & Capture API reference (builder-style)
- **Generator example**
  ```rust
  use otap_df_validation::traffic::Generator;

  let generator = Generator::logs()
      .fixed_count(1000)   // optional; default 2000
      .max_batch_size(64)  // optional; default 100
      .otap_grpc();        // optional; default OTLP
  ```

- `Generator::logs()`, `metrics()`, `traces()` – convenience constructors for weights (other fields keep defaults unless you override).
- Chainable setters:
  - `fixed_count(usize)` (default 2000)
  - `max_batch_size(usize)` (default 100)
  - `otlp_grpc()` / `otap_grpc()` (default OTLP)

- **Capture example (with validations)**
  ```rust
  use otap_df_validation::traffic::Capture;
  use otap_df_validation::ValidationInstructions;
  use otap_df_validation::validation_types::attributes::{AttributeDomain, KeyValue, AnyValue};

  let capture = Capture::default()
      .otlp_grpc()
      .validate(vec![
          ValidationInstructions::Equivalence,
          ValidationInstructions::SignalDrop { min_drop_ratio: None, max_drop_ratio: Some(0.5) },
          ValidationInstructions::AttributeRequireKeyValue {
              domains: vec![AttributeDomain::Signal],
              pairs: vec![KeyValue::new("env".into(), AnyValue::String("prod".into()))],
          },
      ]);
  ```

- `Capture::otlp_grpc()`, `Capture::otap_grpc()` – optional protocol switch. (default OTLP)
- `Capture::validate(Vec<ValidationInstructions>)` – optional override of validations. (default [Equivalence])

### Validation instructions (used with `Capture::validate`)
- `Equivalence`: control and SUV outputs are semantically equal (uses pdata equivalence).
- `SignalDrop { min_drop_ratio, max_drop_ratio }`: asserts the SUV emitted fewer signals within optional ratio bounds.
- `BatchItems { min_batch_size, max_batch_size, timeout }`: bounds the item count per message; `min/max` optional; `timeout` optional
- `BatchBytes { min_bytes, max_bytes, timeout }`: bounds encoded message size; `min/max` optional; `timeout` optional
- `AttributeDeny { domains, keys }`: specified keys must not appear.
- `AttributeRequireKey { domains, keys }`: specified keys must appear.
- `AttributeRequireKeyValue { domains, pairs }`: specified key/value pairs must appear.
- `AttributeNoDuplicate`: check that there are no duplicate attributes

`domains` accepts `AttributeDomain::Resource`, `Scope`, or `Signal` (see `validation_types::attributes`).

## Troubleshooting
- **Missing wire**: Ensure both `wire_*_receiver` and `wire_*_exporter` are called before `Scenario::run()`.
