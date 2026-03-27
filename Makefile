.PHONY: setup dev build install test test-all lint fmt clean

setup:
	cargo build

dev:
	cargo run

build:
	cargo build --release

install: build
	cp target/release/fastread /usr/local/bin/fastread

test:
	cargo test

test-all:
	cargo test

lint:
	cargo clippy -- -D warnings

fmt:
	cargo fmt

clean:
	cargo clean
