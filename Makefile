run-dev:
	RUST_LOG=debug cargo run

run-tests:
	cargo test

run-fmt:
	cargo fmt

build-dev:
	cargo build

build-release:
	cargo build --release