# Test Suites

This directory holds declarative test-suite configurations. These can
be invoked from the pipeline_perf_test directory with:

```shell
python -m orchestrator/run_orchestrator.py -c path_to_config_file
```

## Semantic conventions registry

Suites whose load generator is a `df_engine` `traffic_generator` bind-mount the
vendored semantic conventions registry (`rust/otap-dataflow/vendor/semconv`)
read-only at `/home/nonroot/semconv`, and the loadgen config templates default
`registry_path` to `/home/nonroot/semconv/model`. This means the load generator
doesn't fetch the registry over the network. The mount source is a relative
path (`../../rust/otap-dataflow/vendor/semconv`). Docker resolves it against the
orchestrator's working directory, so run suites from `tools/pipeline_perf_test`,
or from `tools/comparison_dashboard` for comparison dashboard suites.

## Contents

See each directory for additional README.md files with information specific to
the specific test suite(s).

```shell
.
|_ integration         # Integration suite(s) that are run daily or on commit.
```
