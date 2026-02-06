build:
	cargo build --release

test:
	cargo test

clippy:
	cargo clippy -- --deny "warnings"

install-hooks:
	prek install

conform:
	conform enforce --base-branch main~1

check: test conform

license:
	reuse annotate --copyright="Kevin Köster" --license="AGPL-3.0-or-later" -t default $$(find crates -type f -not \( -path "crates/name-variant/*" -prune \) -name '*.rs')
