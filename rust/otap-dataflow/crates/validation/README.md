# Validation Framework

<!-- markdownlint-disable MD013 -->

End-to-end harness for standing up a **system-under-validation (SUV)**
pipeline, driving OTLP/OTAP traffic into it, capturing the output, and
asserting invariants.

This crate is one layer in the broader OTAP-dataflow testing strategy. For
guidance on when to use validation scenarios versus unit tests, node harnesses,
small pipeline liveness tests, or deterministic simulation testing, see the
[Testing Guide](../../docs/testing-guide.md).

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

   Scenario::new()
       .pipeline(pipeline)                      // add your system under validation pipeline
       .add_generator("traffic_gen", generator)       // add your configured generator pipeline
       .add_capture("validate", capture)          // add your configured capture pipeline
       .expect_within(30) // optional timeout; default 25
       .run()
       .expect("validation scenario failed");
   ```

## Scenario

- **Scenario example**

  ```rust
  use otap_df_validation::scenario::Scenario;

  Scenario::new()
      .pipeline(pipeline)               // required: rewired Pipeline
      .add_generator("traffic_gen", generator)                 // required: Generator config
      .add_capture("validate", capture)                 // required: Capture config
      .expect_within(30) // optional; default 25s
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
- `add_container("label", ContainerConfig)` - add Docker container to run
  - optional; used for test container scenarios
  - the label is referenced by `ContainerConnection` and `PipelineContainerConnection`
  - see [Test Containers](#test-containers) for details
- `expect_within(u64)` - set max runtime in seconds
  - optional; default: 25s
- `run()` - renders template, launches pipelines, waits for readiness
  - required to run the validation stage
  - when containers are configured, they are started before the pipeline
    runs and stopped after it shuts down
  - returns `Result<(), ValidationError>` if invalid or timeout

## Pipeline

- **Pipeline example**

  ```rust
  use otap_df_validation::pipeline::Pipeline;

  let pipeline = Pipeline::from_file("./validation_pipelines/your_pipeline.yaml")
      .expect("load pipeline")
  ```

- `Pipeline::from_file(path)` / `from_yaml(str)` - load the SUV pipeline YAML.
  - returns `Result<Self, ValidationError>`
- `Pipeline::from_file_with_vars(path, vars)` - load the SUV pipeline YAML with
  `${VAR}` placeholder substitution.
  - `vars` is a `&[(&str, &str)]` slice of `(key, value)` pairs
  - each `${KEY}` in the YAML is replaced with the corresponding value
  - returns an error if any `${...}` placeholders remain unresolved
  - useful for injecting TLS cert/key paths at test time (see [TLS / mTLS](#tls--mtls))
- `with_transport_headers_policy(TransportHeadersPolicy)` - set a transport
  headers policy on the SUV pipeline from a typed struct
  - controls header capture at receivers and header propagation at exporters
  - optional; see [Transport Headers](#transport-headers) for details
- `with_transport_headers_policy_yaml(yaml_str)` - set a transport headers
  policy from a YAML string
  - returns `Result<Self, ValidationError>` (YAML parsing can fail)
  - alternative to the typed struct method
- `connect_container(PipelineContainerConnection)` - declare a connection between
  a node in the SUV pipeline and a test container
  - the framework rewrites the specified config key with the container's allocated
    host port at runtime
  - see [Test Containers](#test-containers) for details

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
  - required (unless using `to_container`)
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
  - uses the built-in TLS support
- `with_transport_headers(headers)` - configure transport headers to inject
  into generated traffic
  - each key is a header name; the value is an optional fixed string
  - when the value is `None`, the traffic generator assigns a random value
    at startup
  - only meaningful when the pipeline uses OTLP receivers/exporters
- `to_container(ContainerConnection)` - use a custom exporter that sends to a
  test container instead of directly to the SUV pipeline receiver
  - mutually exclusive with `otlp_grpc()` / `otap_grpc()`
  - see [Test Containers](#test-containers) for details

> NOTE: When using `otlp_grpc()` / `otap_grpc()`, the node names must match
the keys under `nodes:` in your pipeline YAML.

### TLS / mTLS

> Uses the built-in TLS support.

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
    .expect_within(30)
    .run()
    .expect("TLS validation scenario failed");
```

#### Example: mTLS scenario

```rust
use otap_df_validation::pipeline::Pipeline;
use otap_df_validation::scenario::Scenario;
use otap_df_validation::traffic::{Capture, Generator, TlsConfig};

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
    .expect_within(30)
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
       .idle_timeout(5)     // optional; seconds to wait for messages before validating; default 3
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
  - required (unless using `from_container`)
  - specifies which exporter in suv pipeline to receive data from
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
- `idle_timeout(u8)` - set how long (in seconds) the validation exporter
  waits without receiving any messages before declaring the data stream
  settled and performing the final validation
  - default: 3 seconds
- `with_capture_header_keys(keys)` - configure transport header keys to capture
  from inbound signals
  - each key becomes a `match_names` rule in the capture pipeline's
    `header_capture` policy
  - required when using transport header validation instructions
- `from_container(ContainerConnection)` - use a custom receiver that reads from
  a test container instead of directly from the SUV pipeline exporter
  - mutually exclusive with `otlp_grpc()` / `otap_grpc()`
  - see [Test Containers](#test-containers) for details

> NOTE: When using `otlp_grpc()` / `otap_grpc()`, the node names must match
the keys under `nodes:` in your pipeline YAML.

### How `idle_timeout` works

The validation exporter tracks the timestamp of the last received message.
On each periodic telemetry tick (every 1 second), it checks whether the
elapsed time since the last message exceeds the configured idle timeout. If
so, it considers the data stream settled, performs the final validation, and
signals completion.

A shorter timeout makes tests complete faster but may trigger validation
prematurely if there are natural pauses in message delivery. A longer
timeout is safer for pipelines with bursty or delayed output but increases
overall test runtime.

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
- `TransportHeaderRequireKey { keys }`: specified transport header keys must be
  present on every SUV message. Fails if any message is missing transport
  headers entirely.
- `TransportHeaderRequireKeyValue { pairs }`: specified transport header
  key/value pairs must be present on every SUV message. Values are compared as
  UTF-8 text (case-sensitive). When duplicate header keys exist, the check
  passes if any entry with that key matches the expected value.
  - `pairs` accepts `Vec<TransportHeaderKeyValue>`
- `TransportHeaderDeny { keys }`: specified transport header keys must NOT
  appear on any SUV message. Messages without transport headers are acceptable
  for this check.

> See [Transport Headers](#transport-headers) for setup and a full example.

(see `validation_types::attributes`, `validation_types::transport_headers`,
and `validation_types`)

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

### Transport Headers

Validate that transport headers survive the full pipeline chain from generator
through the system-under-validation to the capture pipeline.

> **NOTE:** Only OTLP (`otlp_grpc`) receivers and exporters currently support
> transport header capture and propagation. OTAP receivers/exporters do **not**
> yet support transport headers.

For these headers to flow through the pipeline chain, each
stage needs to be appropriately configured:

1. **Generator pipeline** -- automatically adds a `header_propagation` policy
   (propagate all headers) when `with_transport_headers` is configured.
2. **SUV pipeline** -- requires a `transport_headers` policy with both
   `header_capture` (to extract headers from inbound gRPC metadata) and
   `header_propagation` (to re-emit headers on the outbound gRPC connection).
   Configure via `Pipeline::with_transport_headers_policy_yaml()` or
   `Pipeline::with_transport_headers_policy()`.
3. **Capture pipeline** -- requires `with_capture_header_keys` to specify which
   headers to extract from inbound gRPC metadata for validation.

#### Transport header validation instructions

| Instruction | Behavior |
| ----------- | -------- |
| `TransportHeaderRequireKey { keys }` | Assert specified header keys exist on every SUV message |
| `TransportHeaderRequireKeyValue { pairs }` | Assert specified key/value pairs exist on every SUV message |
| `TransportHeaderDeny { keys }` | Assert specified header keys do NOT exist on any SUV message |

For `RequireKey` and `RequireKeyValue`, every SUV message must carry transport
headers (`Some`). A single message without headers causes immediate failure.

For `Deny`, messages without transport headers are acceptable -- a signal that
never received headers cannot contain a forbidden key.

#### Example: transport headers validation

```rust
use otap_df_validation::ValidationInstructions;
use otap_df_validation::pipeline::Pipeline;
use otap_df_validation::scenario::Scenario;
use otap_df_validation::traffic::{Capture, Generator};
use otap_df_validation::validation_types::transport_headers::TransportHeaderKeyValue;

Scenario::new()
    .pipeline(
        Pipeline::from_file("./validation_pipelines/no-processor.yaml")
            .expect("load pipeline")
            .with_transport_headers_policy_yaml(r#"
header_capture:
  headers:
    - match_names: ["x-tenant-id"]
header_propagation:
  default:
    selector:
      type: all_captured
    action: propagate
"#)
            .expect("parse policy"),
    )
    .add_generator(
        "traffic_gen",
        Generator::logs()
            .fixed_count(500)
            .otlp_grpc("receiver")
            .static_signals()
            .with_transport_headers([("x-tenant-id", Some("test-tenant"))]),
    )
    .add_capture(
        "validate",
        Capture::default()
            .otlp_grpc("exporter")
            .with_capture_header_keys(["x-tenant-id"])
            .validate(vec![
                ValidationInstructions::TransportHeaderRequireKey {
                    keys: vec!["x-tenant-id".into()],
                },
                ValidationInstructions::TransportHeaderRequireKeyValue {
                    pairs: vec![TransportHeaderKeyValue::new("x-tenant-id", "test-tenant")],
                },
                ValidationInstructions::TransportHeaderDeny {
                    keys: vec!["x-should-not-exist".into()],
                },
            ])
            .control_streams(["traffic_gen"]),
    )
    .run()
    .expect("transport headers validation failed");
```

### Test Containers

Run Docker containers alongside your validation scenario using the
[testcontainers](https://docs.rs/testcontainers) crate. This is useful when
the system under validation interacts with external services (e.g. LocalStack
for S3-compatible storage, Redis) that you want to spin up on-demand during
tests.

Containers are managed by the `Scenario`: they start **before** the pipeline
runs and stop **after** it shuts down. Port mappings between the container and
the host are allocated automatically by the framework.

#### ContainerConfig

Describes a Docker container to run alongside the scenario.

```rust
use otap_df_validation::ContainerConfig;

let localstack = ContainerConfig::new("localstack/localstack", "3.4")
    .env("SERVICES", "s3")
    .env("DEFAULT_REGION", "us-east-1")
    .wait_for_log("Ready.");
```

- `ContainerConfig::new(image, tag)` - create a container config for the given
  Docker image and tag
- `.env(key, value)` - set an environment variable on the container
  - can be chained multiple times
- `.env_host_port(key, value_template, internal_port)` - set an environment
  variable whose value is a Jinja2 template; `{{ host_port }}` is replaced
  with the host port mapped to `internal_port` after port allocation
  - can be chained multiple times
  - if no connection maps the given `internal_port`, the framework
    auto-allocates a host port during config wiring
  - the resolved host port is consistent with the port used for Docker
    port mapping and any `ContainerConnection` or
    `PipelineContainerConnection` referencing the same internal port
- `.entrypoint(cmd)` - override the container's entrypoint
  - optional
- `.wait_for_nothing()` - do not wait for any readiness condition (default)
- `.wait_for_log(message)` - wait for the given message to appear on stdout or
  stderr before considering the container ready
- `.wait_for_log_stdout(message)` - wait for the message on stdout only
- `.wait_for_log_stderr(message)` - wait for the message on stderr only
- `.wait_for_duration(length)` - wait for the specified `Duration` before
  considering the container ready
- `.wait_for_seconds(seconds)` - convenience wrapper for waiting a number of
  seconds
- `.wait_for_millis(millis)` - convenience wrapper for waiting a number of
  milliseconds
- `.wait_for_healthcheck()` - wait for the container's Docker health check to
  report healthy
- `.wait_for_http(path)` - wait for an HTTP GET request to the given path to
  return a 2xx response
- `.wait_for_http_with_status(path, status_code)` - wait for an HTTP GET
  request to return the specified status code
- `.wait_for_exit()` - wait for the container to exit (any exit code)
- `.wait_for_exit_with_code(expected_code)` - wait for the container to exit
  with the specified exit code

##### Templated environment variables

Some containers require environment variables that reference the host port
assigned to one of their exposed ports. For example, Kafka's advertised
listeners must contain the host-reachable address so that clients outside
the container can connect. Use `env_host_port` for this:

```rust
use otap_df_validation::ContainerConfig;

let kafka = ContainerConfig::new("confluentinc/cp-kafka", "7.5.0")
    .env("KAFKA_NODE_ID", "1")
    .env_host_port(
        "KAFKA_ADVERTISED_LISTENERS",
        "PLAINTEXT://127.0.0.1:{{ host_port }}",
        9092,
    );
```

After config wiring, if host port 54321 was allocated for container port
9092, the container starts with `KAFKA_ADVERTISED_LISTENERS` set to
`PLAINTEXT://127.0.0.1:54321`.

If a `PipelineContainerConnection` or `ContainerConnection` also
references internal port 9092 on the same container, they all share the
same allocated host port.

Register the container on the scenario with `add_container`:

```rust
Scenario::new()
    .add_container("localstack", localstack)
    // ...
```

#### ContainerConnection

Describes a connection between a **Generator** or **Capture** and a test
container. The connection carries a Jinja2 template for a custom node
configuration that is rendered with `{{ port }}` set to the allocated host
port.

```rust
use otap_df_validation::traffic::ContainerConnection;

let conn = ContainerConnection::new("localstack")
    .internal_port(4566)
    .node_template(r#"
type: "exporter:parquet"
config:
  storage:
    s3:
      base_uri: "s3://otel-test-bucket/telemetry"
      region: "us-east-1"
      endpoint: "http://127.0.0.1:{{ port }}"
      allow_http: true
      virtual_hosted_style_request: false
      auth:
        type: static_credentials
        access_key_id: test
        secret_access_key: test
"#);
```

- `ContainerConnection::new(label)` - create a connection referencing a
  container by its label (must match a label passed to `add_container`)
- `.internal_port(port)` - the container's internal port to map
  - required
- `.node_template(template)` - Jinja2 template for the custom exporter or
  receiver node config; `{{ port }}` is the allocated host port
  - required

Use the connection on a Generator or Capture:

- `Generator::to_container(conn)` - the generator uses a custom exporter
  defined by the template instead of `otlp_grpc` / `otap_grpc`
- `Capture::from_container(conn)` - the capture uses a custom receiver
  defined by the template instead of `otlp_grpc` / `otap_grpc`

#### PipelineContainerConnection

Describes a connection between a **node in the SUV pipeline** and a test
container. Instead of providing a full node template, this rewrites a
specific config key in the pipeline YAML with the rendered address.

```rust
use otap_df_validation::pipeline::PipelineContainerConnection;

let conn = PipelineContainerConnection::new("localstack")
    .internal_port(4566)
    .node("parquet_exporter")
    .config_key("storage.s3.endpoint")
    .address_template("http://127.0.0.1:{{ port }}");
```

- `PipelineContainerConnection::new(label)` - create a connection referencing
  a container by its label
- `.internal_port(port)` - the container's internal port to map
  - required
- `.node(name)` - the node name in the SUV pipeline YAML whose config will
  be rewritten
  - required; must match a key under `nodes:` in the pipeline YAML
- `.config_key(path)` - dot-separated path to the config key relative to
  `nodes.<node>.config`
  - required; e.g. `"storage.s3.endpoint"` or `"protocols.grpc.listening_addr"`
- `.address_template(template)` - Jinja2 template for the address value;
  `{{ port }}` is the allocated host port
  - required; e.g. `"127.0.0.1:{{ port }}"` or `"http://127.0.0.1:{{ port }}"`

Attach the connection to a Pipeline:

```rust
let pipeline = Pipeline::from_file("./validation_pipelines/parquet_pipeline.yaml")?
    .connect_container(conn);
```

#### Connection patterns

There are three patterns for wiring containers into a scenario:

##### Pattern A: Generator -> Container (custom exporter)

The generator sends signals to the container via a custom exporter. Useful
when the container provides a service that the generator writes to directly
(e.g. an S3-compatible object store where parquet files are written).

```rust
let generator = Generator::logs()
    .fixed_count(500)
    .to_container(
        ContainerConnection::new("localstack")
            .internal_port(4566)
            .node_template(r#"
type: "exporter:parquet"
config:
  storage:
    s3:
      base_uri: "s3://otel-test-bucket/telemetry"
      region: "us-east-1"
      endpoint: "http://127.0.0.1:{{ port }}"
      allow_http: true
      virtual_hosted_style_request: false
      auth:
        type: static_credentials
        access_key_id: test
        secret_access_key: test
"#),
    );
```

##### Pattern B: Container -> Capture (custom receiver)

The capture reads signals from the container via a custom receiver. Useful
when the container acts as the exit point of the system under validation
(e.g. an S3-compatible object store where parquet files are read back from).

```rust
let capture = Capture::default()
    .from_container(
        ContainerConnection::new("localstack")
            .internal_port(4566)
            .node_template(r#"
type: "receiver:parquet"
config:
  storage:
    s3:
      base_uri: "s3://otel-test-bucket/telemetry"
      region: "us-east-1"
      endpoint: "http://127.0.0.1:{{ port }}"
      allow_http: true
      virtual_hosted_style_request: false
      auth:
        type: static_credentials
        access_key_id: test
        secret_access_key: test
"#),
    )
    .validate(vec![ValidationInstructions::Equivalence])
    .control_streams(["gen"]);
```

##### Pattern C: Pipeline node -> Container (config key rewrite)

A node in the SUV pipeline connects to the container. The framework
rewrites a specific config key with the allocated address. Useful when
the SUV pipeline itself needs to talk to the container (e.g. a parquet
exporter node needs an S3 endpoint).

```rust
let pipeline = Pipeline::from_file("./validation_pipelines/parquet_pipeline.yaml")?
    .connect_container(
        PipelineContainerConnection::new("localstack")
            .internal_port(4566)
            .node("parquet_exporter")
            .config_key("storage.s3.endpoint")
            .address_template("http://127.0.0.1:{{ port }}"),
    );
```

### Full example

```rust
use otap_df_validation::ContainerConfig;
use otap_df_validation::ValidationInstructions;
use otap_df_validation::pipeline::{Pipeline, PipelineContainerConnection};
use otap_df_validation::scenario::Scenario;
use otap_df_validation::traffic::{Capture, ContainerConnection, Generator};

Scenario::new()
    .pipeline(
        Pipeline::from_file("./validation_pipelines/parquet_pipeline.yaml")?
            .connect_container(
                PipelineContainerConnection::new("localstack")
                    .internal_port(4566)
                    .node("parquet_exporter")
                    .config_key("storage.s3.endpoint")
                    .address_template("http://127.0.0.1:{{ port }}"),
            ),
    )
    .add_container(
        "localstack",
        ContainerConfig::new("localstack/localstack", "3.4")
            .env("SERVICES", "s3")
            .env("DEFAULT_REGION", "us-east-1")
            .env_host_port(
                "LOCALSTACK_HOST",
                "127.0.0.1:{{ host_port }}",
                4566,
            ),
    )
    .add_generator(
        "gen",
        Generator::logs()
            .fixed_count(500)
            .otlp_grpc("receiver"),
    )
    .add_capture(
        "cap",
        Capture::default()
            .otlp_grpc("exporter")
            .validate(vec![ValidationInstructions::Equivalence])
            .control_streams(["gen"]),
    )
    .expect_within(30)
    .run()?;
```
