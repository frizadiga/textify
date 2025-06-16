# chill Makefile for Rust — yeah, it's not a thing in Rust project, but whatever 🤘

.PHONY: all build run test clean release

all: dev

build:
	cargo build

dev:
	cargo run

test:
	cargo test

clean:
	cargo clean

release:
	cargo build --release
