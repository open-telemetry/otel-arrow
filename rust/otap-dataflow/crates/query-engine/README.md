# OTAP Query Engine

This crate provides implementation of "query engine" for transforming telemetry
pipeline data in OTAP format.

## Motivation

- Enable efficient, programmable transformations of telemetry data
- Support various various data transformation languages
<!-- 
TODO there are probably other motivations and design principles we could
include such as extensibility if/when we allow for custom pipeline planners,
UDFs and pipeline stages
-->

## Status

**Under Development** This crate is actively being developed.

## Concepts

### Pipeline Stages

The "query engine" drives data through a pipeline of stages that may transform
OTAP Batches. Stages implement the `crate::pipeline::PipelineStage` trait

Although no particular implementation solution is required, generally these
stages will use [DataFusion](https://docs.rs/datafusion/latest/datafusion/),
[arrow-rs compute functions](https://docs.rs/arrow/latest/arrow/compute/index.html),
or some combination of the two to efficiently transform the data.

Stages under active development:

- Filtering

### Pipeline Definition

The query engine's pipeline receives the definition of transforms to be applied
in the from the intermediate abstract language defined in the
[`data_engine_expressions` crate](../../../experimental/query_engine/README.md#intermediate-language-abstraction).

This means that transformations can be defined in any higher level language
that can be transpiled into this IL.

## Example Usage

See the [examples folder](./examples/).
