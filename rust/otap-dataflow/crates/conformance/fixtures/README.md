# Live reconfiguration fixtures

[fixtures.yaml](fixtures.yaml) contains one entry per component under test,
identified by its full registered URN. Each entry gives the runner a DFE node
configuration, a way to exercise it, and an observable result to check after
a live rollout.

## Read a fixture

The runner starts two pipelines in one engine from the
[baseline template](baseline-pipeline.yaml). Each has its own syslog receiver
(`input`). In `main`, the runner adds the fixture's component as `subject`,
changes its configuration, then removes it. During and after each rollout,
it sends marked records through `observer` and checks that its file exporter
still receives them on the original generation.

By default, `main` also starts with a file exporter named `out`. For
[`otlp-http-header` fixtures](#bind-an-extension-to-a-node), `main.out` starts
as the fixture's OTLP HTTP exporter, with the binding to `subject` omitted.
The runner fills in `{{ input_address }}` and `{{ output_file }}` separately
for each copy of the template. A file exporter expands `{signal}`,
`{core_id}`, and `{generation}` when it writes.

![Live rollout of the attribute processor in main while observer keeps delivering.][attribute-rollout]

To adapt this example to your node, choose what a fresh record must show at
`out` after each rollout. In the YAML below, `urn` selects the node named `subject`,
`probe.connections` places it in the route, and `config` changes what it does.
`probe.expect` describes what the exporter must see with each configuration
and after the subject is removed.

```yaml
- urn: urn:otel:processor:attribute
  probe:
    kind: pipeline-node
    role: processor
    connections:
      - { from: input, to: subject }
      - { from: subject, to: out }
    expect:
      kind: log-attribute
      key: guarantee.phase
      values:
        add: { equals: add }
        change: { equals: change }
        remove: absent
  config:
    actions:
      - action: upsert
        key: guarantee.phase
        value: '{{ phase }}'
```

### Run the attribute fixture

From `rust/otap-dataflow`, run the example by its full URN.

```sh
cargo xtask conformance --component urn:otel:processor:attribute
```

The rollout results from a passing run look like this.

```text
add: passed
change: passed
remove: passed
reconfiguration-noop: passed
```

The runner renders `config` separately for add and change. It sends a fresh
syslog record through the graph and reads the attribute at the exporter. On
removal, the baseline route `input -> out` must deliver a fresh record without
the attribute.

### Diagnose a change that never reaches the engine

In the attribute fixture in [fixtures.yaml](fixtures.yaml), change just the
value of the `guarantee.phase` action. Rerun the same focused command.

```diff
-        value: '{{ phase }}'
+        value: 'add'
```

Add passes because the exported record has `guarantee.phase: add`. When the
runner prepares change, the subject still has the configuration it already
uses. The command exits with the diagnostic
`fixture must change the subject configuration or its node policy`.

```text
add: passed
change: failed
remove: unexercised
reconfiguration-noop: unexercised
```

The runner checks the subject before submitting the change. A different
peer, marker, or expectation alone cannot satisfy that check. Restore
`value: '{{ phase }}'` and rerun to exercise change, no-op, and removal.

To replace the entire node configuration on change, supply
`change_config` alongside `config`. This is useful for changing typed values
such as a batch size. The configuration-free noop exporter uses
`probe.change_policies` to enable node-level message telemetry. The runner
reapplies each changed configuration and checks that this is a no-op.

## Choose a probe and its input

Start with `kind: pipeline-node` for a receiver, processor, or exporter.
Choose a specialized probe when you need a stage-specific observation, such
as checking that a retired TCP listener has closed.

| `probe.kind` | What the fixture can check |
| --- | --- |
| `pipeline-node` | Wire the subject to [peers](#wire-a-pipeline-node-probe-with-peers) and check the exported signal, record, response, or sink. The default graph is `input -> subject -> out` and the default expectation is a marked log. |
| `syslog-tcp` | For each stage, `delivers_on` names the listener that must deliver the marker. Optional `closed` names the retired listener that must refuse a new TCP connection. |
| `jsonl-file` | The stage marker must appear at `delivers_to` and be absent from each destination in `absent_from` when the probe checks. |
| `otlp-http-header` | Declare the consuming exporter and its capability binding. On the marked outbound request, the named header must match the stage's `equals` value or be `absent`. |
| `otlp-authorization` | Built-in checks exercise accepted and denied requests at an OTLP receiver and verify unauthenticated delivery after unbinding. |

For `pipeline-node`, set `role` to the registered role.
Choose `input: syslog` (the default), `otlp-logs`, or `otlp-metrics` to send
traffic. Use `autonomous` when the subject generates its own telemetry.

With `input: otlp-logs`, the runner sends one protobuf log via OTLP/HTTP.
Here is the **add** request decoded and shown as YAML.

```yaml
resourceLogs:
  - resource:
      attributes:
        - { key: service.name, value: { stringValue: add } }
    scopeLogs:
      - logRecords:
          - body: { stringValue: fixture-add-record }
            severityText: ERROR
            severityNumber: 17
            attributes:
              - { key: syslog.message, value: { stringValue: fixture-add-record } }
              - { key: guarantee.input, value: { stringValue: add } }
```

On change, the phase becomes `change` and the marker becomes
`fixture-change-record`. The content router selects on the resource's
`service.name`. The filter selects on the record's `guarantee.input`.
The default `marked-log` expectation finds the delivered log by `syslog.message`.

Use live-reconfigurable settings for the change stage. For example, the
console fixture uses pretty output because structured stdout must be claimed
at startup. Topic fixtures keep the same broker implementation across stages
because changing it requires an engine restart.

## Choose what a pipeline-node probe must observe

For `probe.kind: pipeline-node`, `probe.expect.kind` selects the evidence required
after add and change. Omitting `expect` selects `marked-log`. Choose an
observation that would fail if the old configuration were still active.
For example, the batch fixture checks delivery across a change in batch size.
Measuring the resulting batch size would require a separate assertion.

| `probe.expect.kind` | What a pass establishes for this stage | Example in [fixtures.yaml](fixtures.yaml) |
| --- | --- | --- |
| `marked-log` (default) | A log in the stage's file output has a `syslog.message` attribute equal to the fresh marker. | Content router |
| `log-attribute` | That marked log has the string attribute named by `key` with the stage's `equals` value, or the key is `absent`, as specified in `values`. | Attribute processor |
| `log-body` | An exported log has a string body exactly equal to `value`. Use `{{ phase }}` to distinguish add from change. | Log parser |
| `signal` | The file contains a log record or metric for the selected `signal` (`logs` or `metrics`). Optional `contains` requires an exact string value somewhere in that record or metric. It accepts `{{ marker }}` and `{{ phase }}`. | Journald receiver |
| `metrics` | One exported metrics payload contains every name in `present`. The metric named by `sum.name` has an integer sum data point equal to `sum.add` or `sum.change` for this stage. | Host metrics receiver |
| `ack` | With `otlp-logs` or `otlp-metrics` input, the client receives a successful OTLP response with zero rejected log records or metric points. | Noop exporter |
| `nack` | With either OTLP input, the client receives HTTP 503 and a decoded RPC status with code 14 and a message containing `reason`. The reason can include `{{ phase }}`. | Error exporter |
| `console` | The captured engine log contains the stage marker. | Console exporter |
| `parquet` | A decoded row in a Parquet file under the stage directory contains the marker. | Parquet exporter |
| `http` | The local server captures a POST whose URL path contains the configured `path` and whose body contains the marker, after decompression if gzipped. The path can include `{{ phase }}`. | Azure Monitor exporter |
| `scraped-metric` | The capture at `/v1/metrics` receives an OTLP gauge named `guarantee_<phase>` with at least one data point. | Prometheus receiver |
| `clickhouse` | The capture receives a nonempty POST with an `INSERT INTO` query that names the stage table (`logs_<phase>`) and specifies `ArrowStream`. | ClickHouse exporter |
| `geneva-packet` | The capture receives a nonempty POST at `/packets/<phase>` with the stage's bearer token and the `application/x-mepacket` content type. | Geneva metrics exporter |
| `geneva-upload` | The capture receives a nonempty POST at `/api/v1/ingestion/ingest` with the stage's ingestion bearer token. | Geneva exporter |

On **remove**, the pipeline-node probe sends a fresh syslog marker through the
restored baseline and checks delivery at the file exporter. `log-attribute`
also checks `values.remove` on that record. See the [pipeline-node probe](../src/pipeline_node.rs)
for the Rust checks. The [specialized probes](#choose-a-probe-and-its-input)
have their own expectations for each operation.

## Wire a pipeline-node probe with peers

A peer is a helper node that runs in the same DFE pipeline as the node under
test (`subject`). Add a peer when you need a producer to feed your receiver
or a consumer to observe your exporter's output. For a processor that can
use the default `input -> subject -> out` route, omit `peers`.

The Kafka receiver needs a producer to put a marked log on its topic. Its
[catalog entry](fixtures.yaml) declares the producer in the `probe` block.

```yaml
probe:
  kind: pipeline-node
  role: receiver
  peers:
    writer:
      type: exporter:kafka
      config:
        brokers: '{{ brokers }}'
        client_id: write-{{ phase }}
        logs:
          topic: '{{ phase }}'
  connections:
    - { from: input, to: writer }
    - { from: subject, to: out }
```

Choose a distinct node ID such as `writer`, give it a normal DFE `type` and
`config`, and wire that ID into `probe.connections`. The first edge feeds
`writer` from `input`. Kafka carries the record from `writer` to `subject`,
and the second edge feeds `out` from `subject`. The runner renders the peer's config
for add and change, then removes the peer with the subject when it restores
the baseline pipeline.

For the reverse example, see the Kafka exporter in [fixtures.yaml](fixtures.yaml).
The exporter is `subject`, and a Kafka receiver peer named `reader` delivers
the exported record to `out`.

## Bind an extension to a node

A capability binding tells a node which pipeline extension provides a named
capability. The **key** is the capability the node consumes. The **value** is
the extension's ID in this pipeline. The fixture's `urn` determines which
component the runner names `subject`.

![Side-by-side bindings. Azure Monitor is the exporter subject that consumes a bearer token. The flat-file API key extension is the subject that supplies a key to the OTLP HTTP exporter.][capability-bindings]

**When testing a node that consumes a capability**, declare the provider in
`probe.extensions` and bind it to `subject` with `probe.bindings`. The Azure
Monitor exporter fixture uses these fields inside its `pipeline-node` probe.

```yaml
extensions:
  auth:
    type: extension:oauth2_client_auth
    config:
      token_url: '{{ http_url }}/token/{{ phase }}'
      client_id: fixture
      client_secret: fixture-secret
bindings:
  bearer_token_provider: auth
```

Here `input -> subject` carries the marked log. The runner installs `auth`
under the pipeline's `extensions` and copies the binding to
`nodes.subject.capabilities`. The HTTP probe checks the POST body for the
marker and the path for `Custom-add` or `Custom-change`. On removal, a fresh
marker must reach the baseline file exporter.

**When testing an extension**, use a consumer that makes its effect observable.
The flat-file API key fixture uses this `otlp-http-header` probe.

```yaml
probe:
  kind: otlp-http-header
  consumer:
    type: exporter:otlp_http
    config: { endpoint: '{{ http_url }}', client_pool_size: 1, http: {} }
    capabilities: { api_key_provider: subject }
  header: x-test-key
  expect:
    add: { equals: 'add' }
    change: { equals: 'change' }
    remove: absent
```

Here the provider is `extensions.subject` and the consumer is `nodes.out`.
On add and change the exporter reads its key through the binding. The probe
finds the marked POST to `/v1/logs` and checks `x-test-key` on that request.
On removal the exporter must still deliver the new marker with the header absent.

## Route records through named outputs

The content-router fixture declares `hit` and `miss` in `probe.outputs` and
selects `input: otlp-logs`. During add and change, `main` has these connections.

| Declared in `probe.connections` | Deployed in `main` | What flows through it |
| --- | --- | --- |
| `input -> subject` | `feed -> subject` | The runner creates an OTLP receiver named `feed` and sends it the marked log with `service.name` set to the phase. |
| `subject["hit"] -> out` | `subject["hit"] -> out` | Matching records reach the file exporter where the probe looks for the marker. |
| `subject["miss"] -> discard` | `subject["miss"] -> discard` | Unmatched records go to the noop exporter peer. |
| Added by the runner | `input -> baseline` | The original syslog receiver keeps a route to a file exporter while `feed` drives the router. |

Removal restores `input -> out` in `main`. The independent `observer`
pipeline keeps its own receiver and exporter throughout.

![A matching OTLP log follows hit to the file. An unmatched service name would follow miss to the discard peer.][router-outputs]

On change, the route now matches `service.name = change`. With the old
configuration still active, that record would go to `miss` and the probe
would time out waiting for `fixture-change-record` at `out`. Both injected
OTLP records exercise `hit`. Verifying delivery on `miss` would require an
unmatched test record and an observable destination on that branch.

## Use a fixture service

Declare a top-level `services` entry when the runner needs to set up a resource
for your fixture. For example, the Kafka receiver declares `services: [kafka]`.
Before the first rollout, the runner creates one in-process mock Kafka broker
with two topics, `add` and `change`. It makes the broker's address available
as `{{ brokers }}` and keeps the broker alive until the fixture finishes.

![A Kafka exporter peer publishes a marked log to the runner's broker. The receiver under test consumes it and sends it to the file exporter.][kafka-service]

Both the `writer` peer and the receiver under test use `{{ brokers }}` and
select the log topic with `{{ phase }}`. Finding the change marker in the file
proves the receiver can read from the new topic. On removal, the runner checks
delivery of a fresh marker through the restored `input -> out` route.

Run this example from `rust/otap-dataflow`.

```sh
cargo xtask conformance --component urn:otel:receiver:kafka
```

Other supported services are `topics` for two engine topics declared at startup
with `impl_selection: force_mixed`, `azure-identity` for a local identity
endpoint, and `geneva` for local Geneva endpoints. Use `requires` to declare a
prerequisite that the environment must already provide.

## Prepare files and assets

The runner creates a temporary directory for each fixture. Before applying
add or change, it prepares files under `{{ dir }}/{{ phase }}`. Each key in
`prepare.files` is a destination relative to that stage directory. An
`asset` value selects a file under [assets/](assets/) to copy byte for byte.
A `text` value writes a rendered string, as in the API-key fixture's
`test.key: { text: '{{ phase }}' }`. The driver embeds the assets at build time.

The Linux host-metrics fixture uses these two mappings.

```yaml
prepare:
  files:
    proc/cpuinfo: { asset: 'host-proc/{{ phase }}/cpuinfo' }
    proc/stat: { asset: 'host-proc/{{ phase }}/stat' }
```

On add, the runner copies [the one-CPU inputs](assets/host-proc/add/) to
`{{ dir }}/add/proc/`. On change, it copies [the two-CPU inputs][two-cpus] to
`{{ dir }}/change/proc/`. Setting `config.host_view.root_path` to
`'{{ dir }}/{{ phase }}'` makes the receiver read that stage's files. With
`input: autonomous`, the receiver collects on its timer. Its `subject -> out`
route must emit `system.cpu.time` and a `system.cpu.logical.count` of one,
then two. The different counts prove that the receiver switched roots.
The graph also keeps `input -> baseline` for syslog delivery. Removing the
subject restores `input -> out` in `main`.
Separate host roots also let the old and new generations overlap during a
rollout without claiming the same root.

Two fixtures use specialized preparation. `prepare: journal` unpacks
[sample.journal.gz](sample.journal.gz) and writes its matching machine ID.
The two records have identifiers `add` and `change` and bodies
`fixture-add-record` and `fixture-change-record`. To inspect them, decompress
the asset and run `journalctl --file=sample.journal -o export`. The receiver
needs `libsystemd.so.0` at runtime. `prepare: wasm` compiles [pass.wat](pass.wat)
to `pass.wasm` in the stage directory.

## Template variables by field

During add and change, the runner renders string values in `config`,
`change_config`, and the `asset` or `text` values in `prepare.files`.
For a `pipeline-node` probe it also renders string values in `peers`,
`connections`, `outputs`, `bindings`, and `extensions`. YAML mapping keys
and prepared-file destination names are literal. These fields share the
following variables.

| Variable | Value during add or change |
| --- | --- |
| `{{ phase }}` | `add` or `change`. Use it when a setting should differ across rollouts. |
| `{{ address }}` | Reserved loopback `host:port` for this stage. A listener and its client can share it. |
| `{{ dir }}` | Fixture temporary directory. Prepared files for this stage are under `{{ dir }}/{{ phase }}`. |
| `{{ output_file }}` | Stage-specific file exporter path pattern, including the exporter's signal, core, and generation placeholders. |
| `{{ http_url }}` | Base URL of the fixture's local HTTP capture server. |
| `{{ brokers }}` | Bootstrap address for the mock broker created by `services: [kafka]`. |
| `{{ https_url }}` | Local TLS endpoint created by the `geneva` service. |

The baseline and the probe fields below have their own variable scopes.

| Where you write the value | Variables available there |
| --- | --- |
| [Baseline template](baseline-pipeline.yaml) | `{{ input_address }}` and `{{ output_file }}` for that pipeline at startup. |
| `pipeline-node`, `expect.kind: signal` | `contains` accepts `{{ phase }}` and `{{ marker }}` (for example, `fixture-add-record`). |
| `pipeline-node`, other expectations with substitutions | `{{ phase }}` in `value` for `log-body`, `path` for `http`, or `reason` for `nack`. |
| `otlp-http-header` consumer | `{{ http_url }}` sets the export destination. |
| `otlp-http-header` expected `equals` value | `{{ phase }}` names the operation. `{{ basic }}` is the Base64 encoding of `ops:<phase>` for a Basic authorization header. |

In `pipeline-node` expectations, spell the placeholders exactly as shown,
including the spaces. For example, `contains: '{{ marker }}'` matches the
injected marker in a `signal` expectation. Using `{{ marker }}` in the
subject's `config` produces an undefined-variable error.

## Add or update coverage

1. Choose a registered URN in the [support table][support]. Add its entry to
   [fixtures.yaml](fixtures.yaml), or edit the existing entry. Pick a change
   with an observable effect and a probe that can distinguish the two stages.
2. Declare any [fixture services](#use-a-fixture-service) and prerequisites.
   Use the [template variable reference](#template-variables-by-field) to
   connect the node to the runner's listeners, services, or files.
   `requires` controls when the fixture runs. Linux enables `linux`
   automatically. Use `--with NAME` for a declared prerequisite you have
   provisioned.
3. If the component still needs an environment or implementation before it
   can be tested, record its URN in [prerequisites.yaml](prerequisites.yaml).
   Use `needs` for what is missing and `verify` for the behavior to demonstrate.
   Remove that entry when the executable fixture covers it.
4. [Run the focused check](#run-the-attribute-fixture) with your component's
   URN. From `rust/otap-dataflow`, regenerate the support table after editing
   either catalog and review the full inventory with `--base` to catch lost coverage.

```sh
cargo xtask conformance --write-support
cargo xtask conformance --base origin/main
```

See the [conformance guide][guide] for report options and engine-wide checks.

[attribute-rollout]: attribute-processor-rollout.svg
[capability-bindings]: capability-bindings.svg
[router-outputs]: content-router-outputs.svg
[kafka-service]: kafka-receiver-service.svg
[two-cpus]: assets/host-proc/change/
[support]: ../../../docs/conformance/live-support.md
[guide]: ../../../docs/conformance/README.md
