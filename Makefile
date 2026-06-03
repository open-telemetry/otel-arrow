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

# Image name for the OTAP-enabled collector used by the pipeline perf tests and
# as the source of the collector binary for the Rust validation test harness.
# The image is built from the thin Dockerfile, which pins the upstream
# OpenTelemetry Collector Contrib distribution (kept current by Renovate).
OTELARROWCOL_IMAGE ?= otelarrowcol

# Build the collector image from the thin Dockerfile.
.PHONY: docker-otelarrowcol
docker-otelarrowcol:
	docker build . -t $(OTELARROWCOL_IMAGE)

# Extract the collector binary from the image into bin/otelarrowcol, where the
# Rust validation test harness expects it (see
# rust/otap-dataflow/crates/pdata/src/validation/collector.rs). The binary is a
# static (CGO-disabled) Go executable, so it runs directly on the CI host.
.PHONY: otelarrowcol
otelarrowcol: docker-otelarrowcol
	mkdir -p bin
	docker rm -f otelarrowcol-extract >/dev/null 2>&1 || true
	docker create --name otelarrowcol-extract $(OTELARROWCOL_IMAGE)
	docker cp otelarrowcol-extract:/otelcol-contrib bin/otelarrowcol
	docker rm -f otelarrowcol-extract

# Install chloggen at a pinned version. Used by both contributors (via
# `make chlog-new-{go,rust}`) and the release workflows (via
# `make chlog-update`).
CHLOGGEN_VERSION = v0.30.0
CHLOGGEN = chloggen
CHLOGGEN_GO_CONFIG = go/.chloggen/config.yaml
CHLOGGEN_RUST_CONFIG = rust/otap-dataflow/.chloggen/config.yaml
CHANGELOG_GO = go/CHANGELOG.md
CHANGELOG_RUST = rust/otap-dataflow/CHANGELOG.md

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
	@echo "=== $(CHANGELOG_GO) ==="
	$(CHLOGGEN) update --config $(CHLOGGEN_GO_CONFIG) --dry
	@echo "=== $(CHANGELOG_RUST) ==="
	$(CHLOGGEN) update --config $(CHLOGGEN_RUST_CONFIG) --dry

# Render pending entries into the configured CHANGELOG files for VERSION
# and delete the consumed entry files. Invoked by the release workflow.
.PHONY: chlog-update
chlog-update:
	$(CHLOGGEN) update --config $(CHLOGGEN_GO_CONFIG) --version $(VERSION)
	$(CHLOGGEN) update --config $(CHLOGGEN_RUST_CONFIG) --version $(VERSION)
	# chloggen's indent function leaves trailing whitespace on blank sub-text
	# lines and the template emits consecutive blank lines; fix both so the
	# sanity check and markdownlint pass.
	for f in $(CHANGELOG_GO) $(CHANGELOG_RUST); do \
		sed -i.bak 's/[[:space:]]*$$//' $$f && rm -f $$f.bak; \
		cat -s $$f > $$f.tmp && mv $$f.tmp $$f; \
	done
