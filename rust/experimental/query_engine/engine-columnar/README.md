# Columnar Query Engine

This folder contains a query engine implementation which operates on columnar
([Arrow
RecordBatch](https://docs.rs/arrow/latest/arrow/record_batch/struct.RecordBatch.html))
data.

## Goals

* Feature parity with the [RecordSet Engine](..\engine-recordset\README.md)
  (including summaries).

## Should I use [OTAP Query Engine](..\..\..\otap-dataflow\crates\query-engine\README.md) or Columnar Query Engine?

* Are your input records OTAP, OTLP, or something else?

  | Engine | OTAP | OTLP | Other |
  | - | - | - | - |
  | OTAP Query Engine | YES | NO* | NO |
  | Columnar Engine | YES (bridge) | NO* | YES |
  | RecordSet Engine | NO* | YES (bridge) | YES |

  \* OTLP and OTAP data can easily be converted back and forth with some CPU
  cost. The chart above is describing native support.

  The fundamental difference between the OTAP Query Engine and
  Columnar\RecordSet Engines is an abstraction layer. Columnar and RecordSet
  Engines are general purpose things designed to run over unknown schemas. They
  should be runnable in any application. OTAP Query Engine is designed to be run
  in the otel-arrow OTAP Dataflow Engine. The OTAP Query Engine is tied directly
  to otel-arrow OTAP (pdata) structure and only understands OTel schema.

  In the table above "bridge" is mentioned. There are OTLP and OTAP "bridge"
  crates. What these do is add the OTel-specific schema handling into the
  general purpose engines. They accept either OTLP or OTAP, invoke the general
  purpose query engine, and then return OTLP or OTAP back to the caller.

  * [OTLP
    Bridge](..\..\..\otap-dataflow\crates\contrib-nodes\src\processors\recordset_kql_processor\otlp_bridge)
  * OTAP Bridge (coming soon)

  Note: All engines should be able to run an expression tree compiled from any
  language that has a parser (KQL, OPL, OTTL, etc.).

* Do you need summaries?

  It is not currently a goal of the OTAP Query Engine to support summaries. Use
  RecordSet Engine or Columnar Engine if you want summary support.
