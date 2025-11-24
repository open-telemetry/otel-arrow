# Test Suites

This directory holds declarative test-suite configurations. These can
be invoked from the pipeline_perf_test directory with:

```shell
python -m orchestrator/run_orchestrator.py -c path_to_config_file
```

## Contents

See each directory for additional README.md files with information specific to
the specific test suite(s).

```shell
.
|_ integration         # Integration suite(s) that are run daily or on commit.
```
