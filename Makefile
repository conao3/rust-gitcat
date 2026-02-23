.PHONY: build
build:
	cargo build

.PHONY: run
run:
	cargo run

.PHONY: check
check:
	cargo check

.PHONY: test
test:
	cargo test

.PHONY: clean
clean:
	cargo clean
