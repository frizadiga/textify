# chill Makefile for Rust — yeah, it's not a thing in Rust project, but whatever 🤘

.PHONY: all build dev test clean format release

all: dev

build:
	cargo build

dev:
	cargo run -- --perf

test:
	cargo test

clean:
	cargo clean

format:
	cargo fmt

release:
	cargo build --release
