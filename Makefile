run-dev:
	RUST_LOG=debug cargo run

tests:
	cargo test

fmt:
	cargo fmt

build-dev:
	cargo build

build-release:
	cargo build --release