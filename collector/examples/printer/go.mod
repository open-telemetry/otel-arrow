module github.com/open-telemetry/otel-arrow/collector/examples/printer

go 1.21

require go.opentelemetry.io/collector/pdata v1.0.0-rcv0018

require (
	github.com/gogo/protobuf v1.3.2 // indirect
	github.com/golang/protobuf v1.5.3 // indirect
	github.com/google/go-cmp v0.6.0 // indirect
	github.com/json-iterator/go v1.1.12 // indirect
	github.com/modern-go/concurrent v0.0.0-20180306012644-bacd9c7ef1dd // indirect
	github.com/modern-go/reflect2 v1.0.2 // indirect
	go.uber.org/multierr v1.11.0 // indirect
	golang.org/x/net v0.17.0 // indirect
	golang.org/x/sys v0.14.0 // indirect
	golang.org/x/text v0.13.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20231012201019-e917dd12ba7a // indirect
	google.golang.org/grpc v1.59.0 // indirect
	google.golang.org/protobuf v1.31.0 // indirect
)

replace github.com/open-telemetry/otel-arrow/collector/cmd/otelarrowcol => ../../../collector/cmd/otelarrowcol

replace github.com/open-telemetry/otel-arrow/collector/connector/validationconnector => ../../../collector/connector/validationconnector

replace github.com/open-telemetry/otel-arrow/collector/exporter/fileexporter => ../../../collector/exporter/fileexporter

replace github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter => ../../../collector/exporter/otelarrowexporter

replace github.com/open-telemetry/otel-arrow/collector => ../../../collector

replace github.com/open-telemetry/otel-arrow/collector/processor/concurrentbatchprocessor => ../../../collector/processor/concurrentbatchprocessor

replace github.com/open-telemetry/otel-arrow/collector/processor/experimentprocessor => ../../../collector/processor/experimentprocessor

replace github.com/open-telemetry/otel-arrow/collector/processor/obfuscationprocessor => ../../../collector/processor/obfuscationprocessor

replace github.com/open-telemetry/otel-arrow/collector/receiver/filereceiver => ../../../collector/receiver/filereceiver

replace github.com/open-telemetry/otel-arrow/collector/receiver/otelarrowreceiver => ../../../collector/receiver/otelarrowreceiver

replace github.com/open-telemetry/otel-arrow => ../../..
