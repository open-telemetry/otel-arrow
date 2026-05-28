# Because this repository uses multiple `go.mod` files, it is helpful
# to have top-level make rules to run `go test`, `go build`, and `go
# mod tidy` on every module.  Use `make all` to ensure that all Go
# code builds, passes tests, and has update-to-date `go.mod`
# dependencies.
#
# The `make fmt` rule is not included in `all`, to allow manually
# formatting the Go sources.

MODULES := $(shell find . -name go.mod)

GODIRS := $(foreach d,$(MODULES),$(shell dirname $d))
GOCMD?= go
BUILD_INFO=-ldflags "-X $(BUILD_INFO_IMPORT_PATH).Version=$(VERSION)"
VERSION=$(shell git describe --always --match "v[0-9]*" HEAD)
BUILD_INFO_IMPORT_PATH=go.opentelemetry.io/collector/internal/version

.PHONY: all gotidy test build fmt

all: gotidy test build

test:
	for dir in $(GODIRS); do (cd $${dir}; $(GOCMD) test --tags=assert ./...) || exit 1; done

fmt:
	for dir in $(GODIRS); do (cd $${dir}; $(GOCMD) fmt ./...) || exit 1; done

build:
	for dir in $(GODIRS); do (cd $${dir}; $(GOCMD) build ./...) || exit 1; done

gotidy:
	for dir in $(GODIRS); do (cd $${dir}; GOWORK="off" $(GOCMD) mod tidy) || exit 1; done

doc:
	$(GOCMD) run go/tools/data_model_gen/main.go



# Install opentelemetry-collector builder at a specific version which should (almost always) match component references in collector/otelarrowcol-build.yaml
# In addition to installing the builder, this command attempts to synchronize this version in both otelarrowcol-build.yaml and Dockerfile
# In the event that Collector and Collector-Contrib references need to be different versions, manual edits may be required after running this command
BUILDER_VERSION = v0.149.0
BUILDER = builder
.PHONY: $(BUILDER)
builder:
	$(GOCMD) install go.opentelemetry.io/collector/cmd/builder@$(BUILDER_VERSION)
	@echo "Updating otelarrowcol-build.yaml gomods to version $(BUILDER_VERSION)..."
	sed -i 's|go.opentelemetry.io/collector/\([^[:space:]]*\) v[0-9][0-9.]*|go.opentelemetry.io/collector/\1 $(BUILDER_VERSION)|g' collector/otelarrowcol-build.yaml
	sed -i 's|github.com/open-telemetry/opentelemetry-collector-contrib/\([^[:space:]]*\) v[0-9][0-9.]*|github.com/open-telemetry/opentelemetry-collector-contrib/\1 $(BUILDER_VERSION)|g' collector/otelarrowcol-build.yaml
	@echo "Updating Dockerfile to use builder@$(BUILDER_VERSION)..."
	sed -i 's|go.opentelemetry.io/collector/cmd/builder@v[0-9.]*|go.opentelemetry.io/collector/cmd/builder@$(BUILDER_VERSION)|g' Dockerfile

.PHONY: genotelarrowcol
genotelarrowcol: builder
	rm -f collector/cmd/otelarrowcol/*
	GOWORK="off" $(BUILDER) --skip-compilation --config collector/otelarrowcol-build.yaml
	$(MAKE) gotidy

.PHONY: otelarrowcol
otelarrowcol:
	(cd collector/cmd/otelarrowcol && \
		GO111MODULE=on CGO_ENABLED=0 \
		$(GOCMD) build -trimpath -o ../../../bin/otelarrowcol $(BUILD_INFO) .)

.PHONY: docker-otelarrowcol
docker-otelarrowcol:
	docker build . -t otelarrowcol

# Install chloggen at a pinned version. Used by both contributors (via
# `make chlog-new-{go,rust}`) and the release workflows (via
# `make chlog-update`).
CHLOGGEN_VERSION = v0.30.0
CHLOGGEN = chloggen
CHLOGGEN_GO_CONFIG = go/.chloggen/config.yaml
CHLOGGEN_RUST_CONFIG = rust/otap-dataflow/.chloggen/config.yaml

.PHONY: chlog-install
chlog-install:
	$(GOCMD) install go.opentelemetry.io/build-tools/chloggen@$(CHLOGGEN_VERSION)

# Generate a new changelog entry. FILENAME defaults to the current git
# branch name so the entry tracks the PR.
FILENAME ?= $(shell git branch --show-current)

.PHONY: chlog-new-go
chlog-new-go:
	$(CHLOGGEN) new --config $(CHLOGGEN_GO_CONFIG) --filename $(FILENAME)

.PHONY: chlog-new-rust
chlog-new-rust:
	$(CHLOGGEN) new --config $(CHLOGGEN_RUST_CONFIG) --filename $(FILENAME)

.PHONY: chlog-validate
chlog-validate:
	$(CHLOGGEN) validate --config $(CHLOGGEN_GO_CONFIG)
	$(CHLOGGEN) validate --config $(CHLOGGEN_RUST_CONFIG)

.PHONY: chlog-preview
chlog-preview:
	@echo "=== go/CHANGELOG.md ==="
	$(CHLOGGEN) update --config $(CHLOGGEN_GO_CONFIG) --dry
	@echo "=== rust/otap-dataflow/CHANGELOG.md ==="
	$(CHLOGGEN) update --config $(CHLOGGEN_RUST_CONFIG) --dry

# Render pending entries into the configured CHANGELOG files for VERSION
# and delete the consumed entry files. Invoked by the release workflow.
.PHONY: chlog-update
chlog-update:
	$(CHLOGGEN) update --config $(CHLOGGEN_GO_CONFIG) --version $(VERSION)
	$(CHLOGGEN) update --config $(CHLOGGEN_RUST_CONFIG) --version $(VERSION)
