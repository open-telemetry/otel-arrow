module github.com/open-telemetry/otel-arrow/collector/processor/obfuscationprocessor

go 1.21

toolchain go1.21.4

require (
	github.com/cyrildever/feistel v1.5.5
	github.com/stretchr/testify v1.8.4
	go.opentelemetry.io/collector/component v0.96.0
	go.opentelemetry.io/collector/consumer v0.96.0
	go.opentelemetry.io/collector/pdata v1.3.0
	go.opentelemetry.io/collector/processor v0.96.0
	go.uber.org/zap v1.27.0
)

require (
	github.com/btcsuite/btcd/btcec/v2 v2.3.2 // indirect
	github.com/cyrildever/go-utls v1.9.7 // indirect
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/decred/dcrd/dcrec/secp256k1/v4 v4.0.1 // indirect
	github.com/ethereum/go-ethereum v1.13.1 // indirect
	github.com/fatih/color v1.15.0 // indirect
	github.com/go-stack/stack v1.8.1 // indirect
	github.com/go-viper/mapstructure/v2 v2.0.0-alpha.1 // indirect
	github.com/gofrs/uuid v4.4.0+incompatible // indirect
	github.com/gogo/protobuf v1.3.2 // indirect
	github.com/golang/protobuf v1.5.3 // indirect
	github.com/holiman/uint256 v1.2.3 // indirect
	github.com/json-iterator/go v1.1.12 // indirect
	github.com/knadh/koanf/maps v0.1.1 // indirect
	github.com/knadh/koanf/providers/confmap v0.1.0 // indirect
	github.com/knadh/koanf/v2 v2.1.0 // indirect
	github.com/mattn/go-colorable v0.1.13 // indirect
	github.com/mattn/go-isatty v0.0.19 // indirect
	github.com/mitchellh/copystructure v1.2.0 // indirect
	github.com/mitchellh/reflectwalk v1.0.2 // indirect
	github.com/modern-go/concurrent v0.0.0-20180306012644-bacd9c7ef1dd // indirect
	github.com/modern-go/reflect2 v1.0.2 // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	github.com/rogpeppe/go-internal v1.11.0 // indirect
	github.com/vmihailenco/msgpack/v5 v5.3.5 // indirect
	github.com/vmihailenco/tagparser/v2 v2.0.0 // indirect
	go.mongodb.org/mongo-driver v1.12.1 // indirect
	go.opentelemetry.io/collector v0.96.0 // indirect
	go.opentelemetry.io/collector/config/configtelemetry v0.96.0 // indirect
	go.opentelemetry.io/collector/confmap v0.96.0 // indirect
	go.opentelemetry.io/otel v1.24.0 // indirect
	go.opentelemetry.io/otel/metric v1.24.0 // indirect
	go.opentelemetry.io/otel/trace v1.24.0 // indirect
	go.uber.org/multierr v1.11.0 // indirect
	golang.org/x/crypto v0.19.0 // indirect
	golang.org/x/net v0.21.0 // indirect
	golang.org/x/sys v0.17.0 // indirect
	golang.org/x/text v0.14.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20240221002015-b0ce06bbee7c // indirect
	google.golang.org/grpc v1.62.0 // indirect
	google.golang.org/protobuf v1.32.0 // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)

replace github.com/open-telemetry/otel-arrow/collector/cmd/otelarrowcol => ../../../collector/cmd/otelarrowcol

replace github.com/open-telemetry/otel-arrow/collector/connector/validationconnector => ../../../collector/connector/validationconnector

replace github.com/open-telemetry/otel-arrow/collector/examples/printer => ../../../collector/examples/printer

replace github.com/open-telemetry/otel-arrow/collector/exporter/fileexporter => ../../../collector/exporter/fileexporter

replace github.com/open-telemetry/otel-arrow/collector/exporter/otelarrowexporter => ../../../collector/exporter/otelarrowexporter

replace github.com/open-telemetry/otel-arrow/collector => ../../../collector

replace github.com/open-telemetry/otel-arrow/collector/processor/concurrentbatchprocessor => ../../../collector/processor/concurrentbatchprocessor

replace github.com/open-telemetry/otel-arrow/collector/processor/experimentprocessor => ../../../collector/processor/experimentprocessor

replace github.com/open-telemetry/otel-arrow/collector/receiver/filereceiver => ../../../collector/receiver/filereceiver

replace github.com/open-telemetry/otel-arrow/collector/receiver/otelarrowreceiver => ../../../collector/receiver/otelarrowreceiver

replace github.com/open-telemetry/otel-arrow => ../../..
