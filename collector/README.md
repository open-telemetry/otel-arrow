The source code in this directory is automatically copied from a [fork
of the OpenTelemetry
collector](https://github.com/open-telemetry/experimental-arrow-collector),
which maintains the merge history of each OTLP-Arrow component and
allows repeatedly pulling from the mainline. 

The script to generate the source is in `Makefile` and `patch.sed`.

To re-generate the source code here:

1. Two directories up, execute `git clone https://github.com/open-telemetry/experimental-arrow-collector.git arrow-collector`.  This places the Arrow collector in "../../arrow-collector" relative to the Makefile.  Ensure that repository is set to the intended Arrow collector version.
2. Run `make gen` in this directory.
