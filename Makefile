NAME := wacko


.PHONY: run build docs probe flash-softdevice

build:
	cargo build --release

run:
	cargo run --release

docs:
	cargo doc --open

flash-softdevice:
	probe-rs erase --chip nrf52833_xxAA --allow-erase-all; 
	probe-rs download --verify --binary-format hex --chip nrf52833_xxAA ./data/s140_nrf52_7.3.0_softdevice.hex


probe:
	probe-rs attach --chip nRF52833_xxAA --log-format '{L} {s}' \
	target/thumbv7em-none-eabihf/release/wacko