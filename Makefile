run-dev:
	RUST_LOG=debug cargo run

fmt:
	cargo fmt

build-dev:
	cargo build

build-release:
	cargo build --release
