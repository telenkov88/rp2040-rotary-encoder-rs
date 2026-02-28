.PHONY: build build-client build-firmware build-firmware-release build-uf2 flash flash-release run install-tools test test-client test-hardware clean

clean:
	cargo clean --workspace
	cd encoder-firmware && cargo clean

lint:
	cargo clippy --workspace

fmt:
	cargo fmt

build:
	cargo build --workspace

build-client:
	cd encoder-client && cargo build

build-firmware:
	cd encoder-firmware && cargo build

build-firmware-release:
	cd encoder-firmware && cargo build --release

build-uf2: build-firmware-release
	cd encoder-firmware && elf2uf2-rs target/thumbv6m-none-eabi/release/encoder-firmware target/thumbv6m-none-eabi/release/encoder-firmware.uf2
	@echo "\nUF2 built successfully at: encoder-firmware/target/thumbv6m-none-eabi/release/encoder-firmware.uf2"

flash: build-firmware
	cd encoder-firmware && probe-rs download --chip RP2040 --speed 10000 target/thumbv6m-none-eabi/debug/encoder-firmware
	probe-rs reset --chip RP2040

flash-release: build-firmware-release
	cd encoder-firmware && probe-rs download --chip RP2040 --speed 10000 target/thumbv6m-none-eabi/release/encoder-firmware
	probe-rs reset --chip RP2040

run:
	cd encoder-firmware && cargo run

install-tools:
	cargo install probe-rs-tools elf2uf2-rs

test:
	cargo test --workspace


test-client:
	cd encoder-client && cargo test

test-hardware:
	cd encoder-client && cargo test -- --ignored --nocapture
