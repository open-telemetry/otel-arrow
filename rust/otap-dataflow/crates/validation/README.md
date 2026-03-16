# Validation Framework

End-to-end harness for standing up a **system-under-validation (SUV)**
pipeline, driving OTLP/OTAP traffic into it, capturing the output, and
asserting invariants.

## Framework components

- `Scenario`: orchestrates end-to-end runs
  - wire connections -> render pipeline group config -> validate
- `Pipeline`: loads your SUV pipeline YAML and overrides endpoints.
- `Generator` / `Capture`: configure traffic emission and capture/validation.

## Quick setup (end to end)

1) **Author your SUV pipeline YAML** (the thing you want to validate). Use
logical node names for the receiver/exporter you intend to rewire,
e.g. `receiver`, `exporter`.

2) **Wire it dynamically** in the test:

   ```rust
   use otap_df_validation::pipeline::Pipeline;

   let pipeline = Pipeline::from_file("./validation_pipelines/your_pipeline.yaml") // path to your pipeline yaml
       .expect("load pipeline");
   ```

3) **Configure traffic generation**:

   ```rust
   use otap_df_validation::traffic::Generator;

   let generator = Generator::logs()                // logs(), metrics(), traces()
       .fixed_count(500)                           // total signals to emit
       .max_batch_size(50)                         // optional
       .otlp_grpc("receiver")                      // or .otap_grpc()
       .static_signals()                           // or semantic_signals()
       .core_range(2, 2);                          // set core range to use
   ```

4) **Configure capture & validations**:

   ```rust
   use otap_df_validation::traffic::Capture;
   use otap_df_validation::ValidationInstructions;
   use otap_df_validation::validation_types::attributes::{AttributeDomain, KeyValue, AnyValue};

   let capture = Capture::default()
       .otlp_grpc("exporter") // or .otap_grpc()
       .control_streams(["traffic_gen"]) // generator whose unmodified signals this capture should receive
       .validate(vec![
           ValidationInstructions::Equivalence, // control vs SUV outputs match
           ValidationInstructions::SignalDrop { // detect signal drop
               min_drop_ratio: Some(0.2),
               max_drop_ratio: Some(0.8),
           },
           ValidationInstructions::AttributeRequireKeyValue { // require suv signals to all have key value pair
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
       .pipeline(pipeline)                      // add your system under validation pipeline
       .add_generator("traffic_gen", generator)       // add your configured generator pipeline
       .add_capture("validate", capture)          // add your configured capture pipeline
       .expect_within(Duration::from_secs(200)) // optional timeout; default 140
       .run()
       .expect("validation scenario failed");
   ```

## Scenario

- **Scenario example**

  ```rust
  use otap_df_validation::scenario::Scenario;
  use std::time::Duration;

  Scenario::new()
      .pipeline(pipeline)               // required: rewired Pipeline
      .add_generator("traffic_gen", generator)                 // required: Generator config
      .add_capture("validate", capture)                 // required: Capture config
      .expect_within(Duration::from_secs(180)) // optional; default 140s
      .run()
      .expect("validation scenario failed");
  ```

- `Scenario::new()` - create a new Scenario
- `pipeline(Pipeline)` - provide the system-under-validation pipeline
  - required
- `add_generator("gen_key", Generator)` - add traffic generation config
  - required, at least one generator must be configured
  - add support multiple if your pipeline has multiple receivers
- `add_capture("cap_key", Capture)` - add capture/validation config
  - required, at least one capture must be configured
  - can support multiple if your pipeline has multiple exporters
- `expect_within(Duration)` - set max runtime
  - optional; default: 140s
- `run()` - renders template, launches pipelines, waits for readiness
  - required to run the validation stage
  - returns `Result<(), ValidationError>` if invalid or timeout

## Pipeline

- **Pipeline example**

  ```rust
  use otap_df_validation::pipeline::Pipeline;

  let pipeline = Pipeline::from_file("./validation_pipelines/your_pipeline.yaml")
      .expect("load pipeline")
  ```

- `Pipeline::from_file(path)` / `from_yaml(str)` - load the SUV pipeline YAML.
- `Pipeline::from_file_with_vars(path, vars)` - load the SUV pipeline YAML with
  `${VAR}` placeholder substitution.
  - `vars` is a `&[(&str, &str)]` slice of `(key, value)` pairs
  - each `${KEY}` in the YAML is replaced with the corresponding value
  - returns an error if any `${...}` placeholders remain unresolved
  - useful for injecting TLS cert/key paths at test time (see [TLS / mTLS](#tls--mtls))

## Generator

- **Generator example**

  ```rust
  use otap_df_validation::traffic::Generator;

  let generator = Generator::logs()
      .fixed_count(1000)   // optional; default 2000
      .max_batch_size(64)  // optional; default 100
      .core_range(2, 2)    // optional; default 2-2
      .otap_grpc("node_name");        // required; must pass node name of the receiver in your system-under-validation pipeline
                                      // tells the generator which receiver to send signals to
  ```

- `Generator::logs()`, `metrics()`, `traces()` - constructors for signal type
- `fixed_count(usize)` - sets max signals to emit before completion
  - default: 2000
- `max_batch_size(usize)` - controls batch size
  - default: 100
- `otlp_grpc("node_name")` / `otap_grpc("node_name")` - connect to receiver
  - required
  - specifies which receiver in suv pipeline to send data to
  - also sets the exporter type of the generator
    - exporter type must match the receiver type
      - OTLP -> OTLP or OTAP -> OTAP
- `static_signals()` / `semantic_signals()` - choose data source
  - default: static
- `core_range(start, end)` - set the core range to use for pipeline
  - default: 2-2
- `with_tls(TlsConfig)` - enable TLS on the generator's exporter
  - optional; see [TLS / mTLS](#tls--mtls) for details
  - requires the `experimental-tls` feature flag

> NOTE: The node names you pass to `otlp_grpc() / otap_grpc()` must match
the keys under `nodes:` in your pipeline YAML.

### TLS / mTLS

> Requires the `experimental-tls` feature flag.

TLS support is configured on the **Generator** side. The generator's exporter
connects to a TLS-enabled receiver in the SUV pipeline. Use `${VAR}` placeholders
in your SUV pipeline YAML so cert/key paths can be injected at test time via
`Pipeline::from_file_with_vars`.

#### TlsConfig

```rust
use otap_df_validation::traffic::TlsConfig;
```

- `TlsConfig::tls_only(ca_cert_path)` - create a TLS config with
CA verification only
  - the generator trusts the server using the provided CA certificate
- `TlsConfig::mtls(ca_cert_path, client_cert_path, client_key_path)`
  - create a mutual TLS config
  - the generator trusts the server via the CA cert
  and presents a client certificate
- `.with_server_name("name")` - override the server name used
for TLS verification
  - default: `"localhost"`

#### SUV pipeline setup

Your SUV pipeline YAML should include TLS configuration on the receiver with
`${VAR}` placeholders for the cert/key paths:

- **TLS** (server-side only):

  ```yaml
  nodes:
    receiver:
      type: otlp.grpc.receiver
      config:
        endpoint: '127.0.0.1:4317'
        tls:
          cert_file: '${TLS_SERVER_CERT}'
          key_file: '${TLS_SERVER_KEY}'
  ```

- **mTLS** (server + client verification):

  ```yaml
  nodes:
    receiver:
      type: otlp.grpc.receiver
      config:
        endpoint: '127.0.0.1:4317'
        tls:
          cert_file: '${TLS_SERVER_CERT}'
          key_file: '${TLS_SERVER_KEY}'
          client_ca_file: '${TLS_CLIENT_CA}'
          include_system_ca_certs_pool: false
  ```

#### Example: TLS scenario

```rust
use otap_df_validation::pipeline::Pipeline;
use otap_df_validation::scenario::Scenario;
use otap_df_validation::traffic::{Capture, Generator, TlsConfig};
use std::time::Duration;

let server_cert_path = "path/to/server.crt";
let server_key_path = "path/to/server.key";
let ca_cert_path = "path/to/ca.crt";

Scenario::new()
    .pipeline(
        Pipeline::from_file_with_vars(
            "./validation_pipelines/tls-no-processor.yaml",
            &[
                ("TLS_SERVER_CERT", server_cert_path),
                ("TLS_SERVER_KEY", server_key_path),
            ],
        )
        .expect("load pipeline"),
    )
    .add_generator(
        "traffic_gen",
        Generator::logs()
            .fixed_count(500)
            .otlp_grpc("receiver")
            .with_tls(TlsConfig::tls_only(ca_cert_path)),
    )
    .add_capture(
        "validate",
        Capture::default()
            .otlp_grpc("exporter")
            .control_streams(["traffic_gen"]),
    )
    .expect_within(Duration::from_secs(140))
    .run()
    .expect("TLS validation scenario failed");
```

#### Example: mTLS scenario

```rust
use otap_df_validation::pipeline::Pipeline;
use otap_df_validation::scenario::Scenario;
use otap_df_validation::traffic::{Capture, Generator, TlsConfig};
use std::time::Duration;

let server_cert_path = "path/to/server.crt";
let server_key_path = "path/to/server.key";
let ca_cert_path = "path/to/ca.crt";
let client_cert_path = "path/to/client.crt";
let client_key_path = "path/to/client.key";

Scenario::new()
    .pipeline(
        Pipeline::from_file_with_vars(
            "./validation_pipelines/mtls-no-processor.yaml",
            &[
                ("TLS_SERVER_CERT", server_cert_path),
                ("TLS_SERVER_KEY", server_key_path),
                ("TLS_CLIENT_CA", ca_cert_path),
            ],
        )
        .expect("load pipeline"),
    )
    .add_generator(
        "traffic_gen",
        Generator::logs()
            .fixed_count(500)
            .otlp_grpc("receiver")
            .with_tls(TlsConfig::mtls(ca_cert_path, client_cert_path, client_key_path)),
    )
    .add_capture(
        "validate",
        Capture::default()
            .otlp_grpc("exporter")
            .control_streams(["traffic_gen"]),
    )
    .expect_within(Duration::from_secs(140))
    .run()
    .expect("mTLS validation scenario failed");
```

## Capture

- **Capture example (with validations)**

  ```rust
  use otap_df_validation::traffic::Capture;
  use otap_df_validation::ValidationInstructions;
  use otap_df_validation::validation_types::attributes::{AttributeDomain, KeyValue, AnyValue};

  let capture = Capture::default()
      .otlp_grpc("node_name")   // required; must pass node name of exporter in your system-under-validation pipeline
      .control_streams(["input"]) // optional; generator labels whose unmodified signals this capture receives
      .core_range(3, 5)    // optional; default 1-1
      .validate(vec![           // required; define your validation instructions
          ValidationInstructions::Equivalence,
          ValidationInstructions::SignalDrop { min_drop_ratio: None, max_drop_ratio: Some(0.5) },
          ValidationInstructions::AttributeRequireKeyValue {
              domains: vec![AttributeDomain::Signal],
              pairs: vec![KeyValue::new("env".into(), AnyValue::String("prod".into()))],
          },
      ]);
  ```

- `Capture::default()` - create a Capture
- `otlp_grpc("node_name")` / `otap_grpc("node_name")` - connect to exporter
  - required
  - specifies which exporter in suv pipeline to send data to
  - also sets the receiver type of the capture
    - receiver type must match the exporter type
      - OTLP -> OTLP or OTAP -> OTAP
- `control_streams(labels)` - declare which generators this capture
should receive control signals from
  - accepts a list of generator labels (e.g. `["input"]` or `["input1", "input2"]`)
  - required for validation methods that compare against unmodified reference signals
    - e.g. `ValidationInstructions::Equivalence`, `ValidationInstructions::SignalDrop`
  - default: [] (no control streams)
- `validate(Vec<ValidationInstructions>)` - define validation instructions
  - default: []
- `core_range(start, end)` - set the core range to use for pipeline
  - default: 1-1

> NOTE: The node names you pass to `otlp_grpc() / otap_grpc()` must match
the keys under `nodes:` in your pipeline YAML.

### Validation instructions (used with `Capture::validate`)

- `Equivalence`: control and SUV outputs are semantically equal
  - requires control signals
- `SignalDrop { min_drop_ratio, max_drop_ratio }`: asserts the SUV emitted
fewer signals within optional ratio bounds.
  - required control signals
- `BatchItems { min_batch_size, max_batch_size, timeout }`: bounds the item
count per message; `min/max` optional; `timeout` optional
- `BatchBytes { min_bytes, max_bytes, timeout }`: bounds encoded message size;
`min/max` optional; `timeout` optional
- `AttributeDeny { domains, keys }`: specified keys must not appear.
  - `domains` accepts `AttributeDomain::Resource`, `Scope`, or `Signal`
- `AttributeRequireKey { domains, keys }`: specified keys must appear.
  - `domains` accepts `AttributeDomain::Resource`, `Scope`, or `Signal`
- `AttributeRequireKeyValue { domains, pairs }`: specified key/value pairs must
  appear.
  - `domains` accepts `AttributeDomain::Resource`, `Scope`, or `Signal`
  - `pairs` accepts `Vec<KeyValue>`
- `AttributeNoDuplicate`: check that there are no duplicate attributes

(see `validation_types::attributes` and `validation_types`)

> NOTE: Some ValidationInstructions require control signals. Use
`control_streams` on the Capture to declare which generator(s) should
provide reference signals.

## Troubleshooting

- **Missing wire**: Ensure generator and capture are connected properly to
your system-under-validation pipeline, the node names must match
- **Invalid Validation**: Ensure the capture has `control_streams` configured with
the appropriate generator labels so validation instructions have control signals
to validate against

## New Feature Update

### Support multiple input/output connections

- You can define multiple Generator(s)/Capture(s) and add them to your Scenario
  - call add_generator("label1", Generator)/add_capture("label2", Capture)
    - labels for each generator/capture should be unique
  - one Generator/Capture per receiver/exporter node in suv pipeline
- Use `control_streams` on each Capture to declare which generators
it should receive control signals from
  - e.g. `Capture::default().control_streams(["traffic_gen1", "traffic_gen2"])`
  to receive from two generators
  - each generator label must match a label passed to `add_generator`

### Test containers (WIP)
