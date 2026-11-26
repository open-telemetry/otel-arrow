# Live reconfiguration fixtures

[fixtures.yaml](fixtures.yaml) contains one entry per component under test,
identified by its full registered URN. Each entry gives the runner a DFE node
configuration, a way to exercise it, and an observable result to check after
a live rollout.

## Read a fixture

The runner starts two pipelines in one engine from the
[baseline template](baseline-pipeline.yaml). Each has its own syslog receiver
(`input`) and file exporter (`out`). The runner adds the component as
`subject` in `main`, changes its configuration, then removes it. During and
after each rollout it sends marked records through `observer` and verifies
delivery on the original generation. Fixtures only describe changes to `main`.
The baseline's `{{ input_address }}` and `{{ output_file }}` are filled in
before startup. The file exporter expands `{signal}`, `{core_id}`, and
`{generation}` when it writes each signal.

![Live rollout of the attribute processor in main while observer keeps delivering.][attribute-rollout]

To adapt this example to your node, choose what a fresh record must show at
`out` after each rollout. In the YAML below, `urn` selects the node named `subject`,
`probe.connections` places it in the route, and `config` changes what it does.
`probe.expect` describes what the exporter must see with each configuration
and after the subject is removed.

```yaml
- urn: urn:otel:processor:attribute
  probe:
    kind: graph
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

The runner renders `config` separately for add and change. It sends a fresh
syslog record through the graph and reads the attribute at the exporter. On
removal, the baseline route `input -> out` must deliver a fresh record without
the attribute. To replace the entire node configuration on change, supply
`change_config` alongside `config`. This is useful for changing typed values
such as a batch size. A change must alter the subject's configuration or node
policy. The configuration-free noop exporter uses `probe.change_policies` to
enable node-level message telemetry. The runner reapplies each changed
configuration and checks that this is a no-op.

## Choose a probe and its input

| `probe.kind` | What the fixture can check |
| --- | --- |
| `graph` | Wire the subject to peers and check the exported signal, record, response, or sink. The default graph is `input -> subject -> out` and the default expectation is a marked log. |
| `syslog-tcp` | Name the listener that must deliver a record and any retired listener that must refuse connections at each stage. |
| `jsonl-file` | Name the file destinations that must receive the marker or remain free of it at each stage. |
| `otlp-http-header` | Declare the consuming exporter and its capability binding. Check the header on a captured request when the extension is added, changed, and unbound. |
| `otlp-authorization` | Exercise accepted and denied requests at an OTLP receiver and verify unauthenticated delivery after unbinding. |

For a graph probe, set `role` to the registered role. Use `peers` to declare
additional DFE nodes and `connections` to wire them by name. For example, the
Kafka receiver fixture uses an exporter peer to write a record to its topic.
Choose `input: syslog` (the default), `otlp-logs`, or `otlp-metrics` to send
traffic. Use `autonomous` when the subject generates its own telemetry.

Use live-reconfigurable settings for the change stage. For example, the
console fixture uses pretty output because structured stdout must be claimed
at startup. Topic fixtures keep the same broker implementation across stages
because changing it requires an engine restart.

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

## Add or update coverage

1. Choose a registered URN in the [support table][support]. Add its entry to
   [fixtures.yaml](fixtures.yaml), or edit the existing entry. Pick a change
   with an observable effect and a probe that can distinguish the two stages.
2. Supply any local services and prerequisites. `services` asks the runner
   to provision `kafka`, `topics`, `azure-identity`, or `geneva`. The topic
   fixtures use two topics declared at startup with
   `impl_selection: force_mixed`. In node configurations, use `{{ address }}`
   for the stage's reserved listener, `{{ output_file }}` for its export path,
   `{{ http_url }}` for the HTTP capture, and `{{ brokers }}` for the mock
   Kafka broker.
   `requires` controls when the fixture runs. Linux enables `linux`
   automatically. Use `--with NAME` for a declared prerequisite you have
   provisioned.
3. If the component still needs an environment or implementation before it
   can be tested, record its URN in [prerequisites.yaml](prerequisites.yaml).
   Use `needs` for what is missing and `verify` for the behavior to demonstrate.
   Remove that entry when the executable fixture covers it.
4. From `rust/otap-dataflow`, run the focused check for your URN. Regenerate
   the support table after editing either catalog. Review the full inventory
   with `--base` to catch lost coverage.

```sh
cargo xtask conformance --component urn:otel:processor:attribute
cargo xtask conformance --write-support
cargo xtask conformance --base origin/main
```

The report gives separate add, change, and remove results. A failed stage
leaves later stages unexercised. See the [conformance guide][guide] for report
options and engine-wide checks.

[attribute-rollout]: attribute-processor-rollout.svg
[two-cpus]: assets/host-proc/change/
[support]: ../../../docs/conformance/live-support.md
[guide]: ../../../docs/conformance/README.md
