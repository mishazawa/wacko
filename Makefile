NAME := wacko


.PHONY: run build docs lab

build:
	cargo build --release

run:
	cargo run --release

docs:
	cargo doc --open

lab:
	uv run jupyter lab --notebook-dir=.
