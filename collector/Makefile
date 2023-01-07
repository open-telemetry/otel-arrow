GO_FILES := $(shell find . -name '*.go')

# Function to execute a command. Note the empty line before endef to make sure each command
# gets executed separately instead of concatenated with previous one.
# Accepts command to execute as first parameter.
define exec-command
$(1)

endef

all:
	echo "Run 'make gen'"

DIR_NAMES := internal exporter receiver cmd
SOURCE_DIR := ../../arrow-collector

gen:
	rm -rf ${DIR_NAMES}
	$(foreach dir, $(DIR_NAMES),$(call exec-command,mkdir $(dir)))
	cp -r ${SOURCE_DIR}/exporter/otlpexporter exporter
	rm exporter/otlpexporter/go.{sum,mod}
	cp -r ${SOURCE_DIR}/receiver/otlpreceiver receiver
	rm receiver/otlpreceiver/go.{sum,mod}
	cp -r ${SOURCE_DIR}/internal/testutil internal
	cp -r ${SOURCE_DIR}/internal/testdata internal
	cp -r ${SOURCE_DIR}/internal/sharedcomponent internal
	cp -r ${SOURCE_DIR}/cmd/otelcorecol cmd/otelarrowcol
	rm cmd/otelarrowcol/go.{sum,mod}
	rm cmd/otelarrowcol/Makefile
	rm cmd/otelarrowcol/builder-config.yaml
	$(MAKE) subgen

subgen:
	$(foreach file, $(GO_FILES),$(call exec-command,sed -i '' -f patch.sed $(file)))
	go mod tidy
	go build ./...
	go test ./...
