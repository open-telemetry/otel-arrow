# OTAP Dataflow Engine Controller

A controller takes a pipeline configuration and initiates one dataflow engine per core (or less if the number of CPUs or the percentage of CPUs is specified).

Each engine is started on a dedicated CPU core (via thread pinning).