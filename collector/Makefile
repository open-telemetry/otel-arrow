GO_FILES := $(shell find gen -name '*.go')

# Function to execute a command. Note the empty line before endef to make sure each command
# gets executed separately instead of concatenated with previous one.
# Accepts command to execute as first parameter.
define exec-command
$(1)

endef

all:
	echo "Run 'make gen'"

DIR_NAMES := gen/internal gen/exporter gen/receiver
SOURCE_DIR := ../../arrow-collector

generate:
	rm -rf ${DIR_NAMES}
	$(foreach dir, $(DIR_NAMES),$(call exec-command,mkdir -p $(dir)))
	cp -r ${SOURCE_DIR}/exporter/otlpexporter gen/exporter
	rm gen/exporter/otlpexporter/go.{sum,mod}
	cp -r ${SOURCE_DIR}/receiver/otlpreceiver gen/receiver
	rm gen/receiver/otlpreceiver/go.{sum,mod}
	cp -r ${SOURCE_DIR}/internal/netstats gen/internal
	cp -r ${SOURCE_DIR}/internal/testutil gen/internal
	cp -r ${SOURCE_DIR}/internal/testdata gen/internal
	cp -r ${SOURCE_DIR}/internal/sharedcomponent gen/internal
	$(MAKE) subgen

subgen:
	$(foreach file, $(GO_FILES),$(call exec-command,sed -i '' -f patch.sed $(file)))
	go mod tidy
	go build ./...
	go test ./...
