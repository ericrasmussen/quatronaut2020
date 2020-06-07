.PHONY: build
build:
	cargo build

.PHONY: check
check:
	cargo check --features "vulkan" --all-targets --benches --bins --examples --tests --workspace

.PHONY: clean
clean:
	cargo clean

.PHONY: clippy
clippy:
	cargo clippy --features "vulkan" --all-targets --benches --bins --examples --tests --workspace -- -D warnings

.PHONY: doc
doc:
	cargo +nightly doc --features "vulkan" --no-deps --package benitron3000

.PHONY: fmt
format:
	cargo +nightly fmt --all

.PHONY: run
run:
	cargo run

.PHONY: test
test:
	cargo test --features "vulkan" --all-targets --benches --bins --examples --tests --workspace
