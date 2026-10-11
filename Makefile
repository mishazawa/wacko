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

flash-softdevice:
	probe-rs erase --chip nrf52833_xxAA --allow-erase-all; 
	probe-rs download --verify --binary-format hex --chip nrf52833_xxAA ./data/s140_nrf52_7.3.0_softdevice.hex