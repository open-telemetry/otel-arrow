# Query Engine

This folder contains work in progress to implement an engine which can run
queries over different data shapes.

## Folder structure

|Name            |Description                                                                              |
|----------------|-----------------------------------------------------------------------------------------|
|expressions     |Intermediary language and syntax tree for the query engine                               |
|kql-parser      |Parser to turn KQL queries into query engine expressions (syntax trees)                  |
|engine-columnar |Query engine implementation which takes a syntax tree and runs over columnar data (arrow)|
|engine-recordset|Query engine implementation which takes a syntax tree and runs over set or records (otlp)|
