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

# Multimod can be installed using:
#
#   go install github.com/open-telemetry/opentelemetry-go-build-tools/multimod@latest
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
	$(MAKE) gotidy

COMMIT?=HEAD
REMOTE?=git@github.com:open-telemetry/opentelemetry-collector.git
.PHONY: push-tags
push-tags: $(MULTIMOD)
	$(MULTIMOD) verify
	set -e; for tag in `$(MULTIMOD) tag -m ${MODSET} -c ${COMMIT} --print-tags | grep -v "Using" `; do \
		echo "pushing tag $${tag}"; \
		git push ${REMOTE} $${tag}; \
	done;

.PHONY: check-changes
check-changes: $(YQ)
ifndef MODSET
	@echo "MODSET not defined"
	@echo "usage: make check-changes PREVIOUS_VERSION=<version eg 0.52.0> MODSET=beta"
	exit 1
endif
ifndef PREVIOUS_VERSION
	@echo "PREVIOUS_VERSION not defined"
	@echo "usage: make check-changes PREVIOUS_VERSION=<version eg 0.52.0> MODSET=beta"
	exit 1
else
ifeq (, $(findstring v,$(PREVIOUS_VERSION)))
NORMALIZED_PREVIOUS_VERSION="v$(PREVIOUS_VERSION)"
else
NORMALIZED_PREVIOUS_VERSION="$(PREVIOUS_VERSION)"
endif
endif
	@all_submods=$$($(YQ) e '.module-sets.*.modules[] | select(. != "go.opentelemetry.io/collector")' versions.yaml | sed 's/^go\.opentelemetry\.io\/collector\///'); \
	mods=$$($(YQ) e '.module-sets.$(MODSET).modules[]' versions.yaml | sed 's/^go\.opentelemetry\.io\/collector\///'); \
	changed_files=""; \
	for mod in $${mods}; do \
		if [ "$${mod}" == "go.opentelemetry.io/collector" ]; then \
			changed_files+=$$(git diff --name-only $(NORMALIZED_PREVIOUS_VERSION) -- $$(printf '%s\n' $${all_submods[@]} | sed 's/^/:!/' | paste -sd' ' -) | grep -E '.+\.go$$'); \
		elif ! git rev-parse --quiet --verify $${mod}/$(NORMALIZED_PREVIOUS_VERSION) >/dev/null; then \
			echo "Module $${mod} does not have a $(NORMALIZED_PREVIOUS_VERSION) tag"; \
			echo "$(MODSET) release is required."; \
			exit 0; \
		else \
			changed_files+=$$(git diff --name-only $${mod}/$(NORMALIZED_PREVIOUS_VERSION) -- $${mod} | grep -E '.+\.go$$'); \
		fi; \
	done; \
	if [ -n "$${changed_files}" ]; then \
		echo "The following files changed in $(MODSET) modules since $(NORMALIZED_PREVIOUS_VERSION): $${changed_files}"; \
	else \
		echo "No $(MODSET) modules have changed since $(NORMALIZED_PREVIOUS_VERSION)"; \
		echo "No need to release $(MODSET)."; \
		exit 1; \
	fi

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
	# check if any modules have changed since the previous release
	$(MAKE) check-changes
	# update files with new version
	sed -i.bak 's/$(PREVIOUS_VERSION)/$(RELEASE_CANDIDATE)/g' versions.yaml
	sed -i.bak 's/$(PREVIOUS_VERSION)/$(RELEASE_CANDIDATE)/g' ./cmd/builder/internal/builder/config.go
	sed -i.bak 's/$(PREVIOUS_VERSION)/$(RELEASE_CANDIDATE)/g' ./cmd/builder/test/core.builder.yaml
	sed -i.bak 's/$(PREVIOUS_VERSION)/$(RELEASE_CANDIDATE)/g' ./cmd/otelcorecol/builder-config.yaml
	sed -i.bak 's/$(PREVIOUS_VERSION)/$(RELEASE_CANDIDATE)/g' examples/k8s/otel-config.yaml
	find . -name "*.bak" -type f -delete
	# commit changes before running multimod
	git add .
	git commit -m "prepare release $(RELEASE_CANDIDATE)"
	$(MAKE) multimod-prerelease
	# regenerate files
	$(MAKE) -C cmd/builder config
	$(MAKE) genotelcorecol
	git add .
	git commit -m "add multimod changes $(RELEASE_CANDIDATE)" || (echo "no multimod changes to commit")
