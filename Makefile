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
	for dir in $(GODIRS); do (cd $${dir}; $(GOCMD) test -race --tags=assert ./...) || exit 1; done

fmt:
	for dir in $(GODIRS); do (cd $${dir}; $(GOCMD) fmt ./...) || exit 1; done

build:
	for dir in $(GODIRS); do (cd $${dir}; $(GOCMD) build ./...) || exit 1; done

gotidy:
	for dir in $(GODIRS); do (cd $${dir}; GOWORK="off" $(GOCMD) mod tidy) || exit 1; done

doc:
	$(GOCMD) run tools/data_model_gen/main.go

# Multimod can be installed using:
#
#   $(GOCMD) install github.com/open-telemetry/opentelemetry-go-build-tools/multimod@latest
#
# TODO install this locally
MULTIMOD := multimod
.PHONY: $(MULTIMOD)

.PHONY: multimod-verify
multimod-verify:
	@echo "Validating versions.yaml"
	$(MULTIMOD) verify

MODSET?=beta
.PHONY: multimod-prerelease
multimod-prerelease: $(MULTIMOD)
	$(MULTIMOD) prerelease -s=true -b=false -v ./versions.yaml -m ${MODSET}
# this is a hack to sync the otelarrowreceiver during this process
# to avoid gomod ambigious imports.
	(cd collector/receiver/otelarrowreceiver/ && $(GOCMD) work sync)
	$(MAKE) gotidy

COMMIT?=HEAD
REMOTE?=git@github.com:open-telemetry/otel-arrow.git
.PHONY: push-release
push-release: $(MULTIMOD)
	$(MULTIMOD) verify
	set -e; for tag in `$(MULTIMOD) tag -m ${MODSET} -c ${COMMIT} --print-tags | grep -v "Using" `; do \
		echo "pushing tag $${tag}"; \
		git push ${REMOTE} $${tag}; \
	done;

.PHONY: prepare-release
prepare-release:
ifndef MODSET
	@echo "MODSET not defined"
	@echo "usage: make prepare-release RELEASE_CANDIDATE=<version eg 0.53.0> PREVIOUS_VERSION=<version eg 0.52.0> MODSET=beta"
	exit 1
endif
ifdef PREVIOUS_VERSION
	@echo "Previous version $(PREVIOUS_VERSION)"
else
	@echo "PREVIOUS_VERSION not defined"
	@echo "usage: make prepare-release RELEASE_CANDIDATE=<version eg 0.53.0> PREVIOUS_VERSION=<version eg 0.52.0> MODSET=beta"
	exit 1
endif
ifdef RELEASE_CANDIDATE
	@echo "Preparing ${MODSET} release $(RELEASE_CANDIDATE)"
else
	@echo "RELEASE_CANDIDATE not defined"
	@echo "usage: make prepare-release RELEASE_CANDIDATE=<version eg 0.53.0> PREVIOUS_VERSION=<version eg 0.52.0> MODSET=beta"
	exit 1
endif
	# ensure a clean branch
	git diff -s --exit-code || (echo "local repository not clean"; exit 1)
	# update files with new version
	sed -i.bak 's/$(PREVIOUS_VERSION)/$(RELEASE_CANDIDATE)/g' versions.yaml
	sed -i.bak 's/$(PREVIOUS_VERSION)/$(RELEASE_CANDIDATE)/g' collector/otelarrowcol-build.yaml
	sed -i.bak 's/$(PREVIOUS_VERSION)/$(RELEASE_CANDIDATE)/g' collector/cmd/otelarrowcol/main.go
	find . -name "*.bak" -type f -delete
	$(GOCMD) run ./tools/replacer fix
	# commit changes before running multimod
	git add .
	git commit -m "prepare release $(RELEASE_CANDIDATE)"
	$(MAKE) multimod-prerelease
	git add .
	git commit -m "multimode changes $(RELEASE_CANDIDATE)"
	# regenerate files
	$(MAKE) gotidy
	$(GOCMD) run ./tools/replacer unfix
	git add .
	git commit -m "go mod tidy $(RELEASE_CANDIDATE)"
	# ensure a clean branch (that was a test--gotidy should be idempotent and should not change the working dir again)
	git diff -s --exit-code || (echo "local repository not clean"; exit 1)
	git add .
	git commit -m "remove replace statements $(RELEASE_CANDIDATE)" || (echo "no multimod changes to commit")

# Install OTC's builder at the version WHICH MUST MATCH collector/otelarrowcol-build.yaml
BUILDER = builder
.PHONY: $(BUILDER)
builder:
	$(GOCMD) install go.opentelemetry.io/collector/cmd/builder@v0.98.0

.PHONY: genotelarrowcol
genotelarrowcol: builder
	rm -f collector/cmd/otelarrowcol/*
	GOWORK="off" $(GOCMD) run ./tools/replacer fix
	GOWORK="off" $(BUILDER) --skip-compilation --config collector/otelarrowcol-build.yaml
	GOWORK="off" $(GOCMD) run ./tools/replacer fix
	$(GOCMD) work sync
	$(MAKE) gotidy
	GOWORK="off" $(GOCMD) run ./tools/replacer unfix
	$(MAKE) gotidy

.PHONY: otelarrowcol
otelarrowcol:
	(cd collector/cmd/otelarrowcol && \
		GO111MODULE=on CGO_ENABLED=0 \
		$(GOCMD) build -trimpath -o ../../../bin/otelarrowcol $(BUILD_INFO) .)

.PHONY: docker-otelarrowcol
docker-otelarrowcol:
	docker build . -t otelarrowcol
