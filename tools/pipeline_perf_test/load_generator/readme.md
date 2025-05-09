# Load Generator

A simple Python script that continuously sends OTLP logs for a specified
duration. At the end of the run, it outputs the total count of logs sent to
stdout, which can be parsed to determine the number of logs sent.

## Future Enhancements

- Utilize language-specific OpenTelemetry SDKs.
- Integrate load generation tools like Locust or custom telemetry generators.
- Extend the script to support configurable options such as log size and length.
