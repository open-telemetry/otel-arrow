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



# Install OTC's builder at the version WHICH MUST MATCH collector/otelarrowcol-build.yaml
BUILDER = builder
.PHONY: $(BUILDER)
builder:
	$(GOCMD) install go.opentelemetry.io/collector/cmd/builder@v0.133.0

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
