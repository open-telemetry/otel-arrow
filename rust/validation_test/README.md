# Validation Tests

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
When defining your pipeline have the receiver listen to
`127.0.0.1:4317` and exporter export to `http://127.0.0.1:4318`
After adding your pipeline update the `pipeline_validation_configs.yaml`
file. There are already some pipelines defined in the
`pipeline_validation_configs.yaml` file feel free to use
these as a reference when making your additions.
Below are what each required key is used for

- name -> Validation test name for your pipeline
- pipeline_config_path -> Path to your pipeline config
- loadgen_exporter_type -> What receiver type are you using (`otlp` or `otap`)
- backend_receiver_type -> What exporter type are you using (`otlp` or `otap`)
- transformative -> Does your pipeline modify or alter the data

## Future directions

- Automatically trigger the validation process when a PR becomes "Ready for review".
- Manually trigger the validation process when a comment containing `@validation` 
  is added to a PR (strecht goal).
- Extend the validation exporter to support more complex validation procedure.
