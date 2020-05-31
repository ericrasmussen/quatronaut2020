.PHONY: build
build:
	cargo build -Z features=itarget

.PHONY: check
check:
	cargo check -Z features=itarget --all-features --all-targets --benches --bins --examples --tests --workspace

.PHONY: clean
clean:
	cargo clean

.PHONY: clippy
clippy:
	cargo clippy -Z features=itarget --all-features --all-targets --benches --bins --examples --tests --workspace -- -D warnings

.PHONY: doc
doc:
	cargo +nightly doc -Z features=itarget --all-features --no-deps --package benitron3000

.PHONY: fmt
format:
	cargo +nightly fmt --all

.PHONY: run
run:
	cargo run -Z features=itarget

.PHONY: test
test:
	cargo test -Z features=itarget --all-features --all-targets --benches --bins --examples --tests --workspace
