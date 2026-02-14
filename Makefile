HOOK_BINARY ?= prek
CONFORM ?= conform
REUSE ?= reuse
CARGO ?= cargo
SCHEMA_OUTPUT ?= schema.json

JUNIT_REPORT_FILE ?= report.xml


.PHONY: all
all: build

check-%:
	@which $* > /dev/null 2>&1 || (echo "Could not find '$*' in PATH. Just install Nix and run 'direnv allow' if you are having issues installing the dependencies" && exit 1)

.PHONY: build
build: check-$(CARGO)
	$(CARGO) build --release

build-%: check-$(CARGO)
	$(CARGO) build --release --target $*

.PHONY: clean
clean: check-$(CARGO)
	$(CARGO) clean

.PHONY: test
test: check-$(CARGO)
	$(CARGO) test

.PHONY: clippy
clippy: check-$(CARGO)
	$(CARGO) clippy -- --deny "warnings"

.PHONY: install-hooks
install-hooks: check-$(HOOK_BINARY)
	$(HOOK_BINARY) install

.PHONY: conform
conform: check-$(CONFORM)
	$(CONFORM) enforce --base-branch main

.PHONY: check
check: test conform

.PHONY: license
license: check-$(REUSE)
	$(REUSE) annotate --copyright="Kevin Köster" --license="AGPL-3.0-or-later" -t default $$(find crates -type f -not \( -path "crates/name-variant/*" -prune \) -name '*.rs')

$(SCHEMA_OUTPUT): check-cargo
	$(CARGO) run -- --export-config-schema > $(SCHEMA_OUTPUT)

.PHONY: schema
schema: $(SCHEMA_OUTPUT)

.PHONY: test-ci
test-ci:
	cargo-tarpaulin --tests --out xml --engine llvm -- -Z unstable-options --format=json | cargo2junit > $(JUNIT_REPORT_FILE)

.PHONY: FORCE
FORCE:

crates/jsonnet-bridge/go/gen.go: FORCE
	rust2go-cli --src crates/jsonnet-bridge/src/go.rs --dst crates/jsonnet-bridge/go/gen.go

.PHONY: rust2go
rust2go: crates/jsonnet-bridge/go/gen.go
