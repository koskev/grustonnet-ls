build:
	cargo build --release

test:
	cargo test

clippy:
	cargo clippy

install-hooks:
	pre-commit install

conform:
	conform enforce --base-branch main~1

check: test conform
