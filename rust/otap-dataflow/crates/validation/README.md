# Validation

The validation test validates a OTLP or OTAP messages after
experiencing various processes such as encoding/decoding
or going through a pipeline.

## Encoding/Decoding Validation

To validate whether encoding/decoding is working properly
we comparing the input and output to check that they are equivalent.

## Pipeline Validation

To validate pipelines we create a pipeline group that
has three pipelines:

- traffic-gen -> Generate traffic to use for validation
- suv -> System under validation the pipeline being validated
- validate -> Validate the messages received from the suv

The `traffic-gen` pipeline has a fan out processor that will
send message via two exporters one that connects to the `suv`
pipeline and one that connects to the `validate` pipeline.
The `validate` pipeline has a fan in connection with two
receivers which will take in messages from both the `suv`
pipeline and `traffic-gen` pipeline and compare both messages
against each other to determine the validity of the `suv` pipeline

### Adding pipelines to the validation process

Define your pipeline nodes in a yaml file, save the
configuration under the `validation_pipelines` directory.
After adding your pipeline update the `pipeline_validation_scenarios.yaml`
file. There are already some validation scenarios defined in the
`pipeline_validation_scenarios.yaml` file feel free to use
these as a reference when making your additions.
Here is an example scenario definition.

```yaml
  - name: "Debug Processor"
    scenario_config_path: ./validation_pipelines/debug-processor.yaml
    traffic_generation_config:
      suv_exporter_type: otlp
      suv_endpoint: "http://127.0.0.1:4317"
      control_exporter_type: otlp
      control_endpoint: "http://127.0.0.1:4316"
      max_signal_count: 2000
      max_batch_size: 100
      signals_per_second: 100
    traffic_capture_config:
      suv_receiver_type: otap
      suv_listening_addr: "127.0.0.1:4318"
      control_receiver_type: otlp
      control_listening_addr: "127.0.0.1:4316"
      transformative: false
```

notice the `traffic_generation_config` and `traffic_capture_config`
these define the `traffic-gen` and `validate` pipelines the values

#### Traffic Generation Config

- suv_exporter_type: The exporter to use to send messages to the `suv` pipeline
  - default = otlp
- suv_endpoint: The endpoint to send messages to the `suv` pipeline
  - default = "http://127.0.0.1:4317"
- control_exporter_type: The exporter to use to send messages to the `validate` pipeline
  - default = otlp
- control_endpoint: The endpoint to send messages to the `validate` pipeline
  - default = "http://127.0.0.1:4316"
- max_signal_count: The max signals to use for the validation 
  - default = 2000
- max_batch_size: The max batch size to use for signals 
  - default = 100
- signals_per_second: How often the signals are sent through the pipeline 
  - default = 100


#### Traffic Capture Config

- suv_receiver_type: The receier to use to get messages from the `suv` pipeline
  - default = otlp
- suv_listening_addr: The endpoint to get messages from the `suv` pipeline
  - default = "127.0.0.1:4318"
- control_receiver_type: The receier to use to get messages from the `traffic-gen` pipeline
  - default = otlp
- control_listening_addr: The endpoint to send messages to the `traffic-gen` pipeline
  - default = "127.0.0.1:4316"
- transformative: Is the `suv` pipeline going to transform the data
  - default = false

## Future directions
- Extend the validation exporter to support more complex validation procedure.
