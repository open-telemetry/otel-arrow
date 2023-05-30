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

.PHONY: all gotidy test build fmt

all: gotidy test build

test:
	for dir in $(GODIRS); do (cd $${dir} && go test ./...); done

fmt:
	for dir in $(GODIRS); do (cd $${dir} && go fmt ./...); done

build:
	for dir in $(GODIRS); do (cd $${dir} && go build ./...); done

gotidy:
	for dir in $(GODIRS); do (cd $${dir} && go mod tidy); done

doc:
	go run tools/data_model_gen/main.go