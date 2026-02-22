build:
	cargo build --workspace

build-client:
	cd encoder-client && cargo build

build-firmware:
	cd encoder-firmware && cargo build

build-firmware-release:
	cd encoder-firmware && cargo build --release

build-uf2: build-firmware-release
	# Converts the compiled ELF to UF2 format for simple drag-and-drop programming on the RP2040 boot disk.
	cd encoder-firmware && elf2uf2-rs target/thumbv6m-none-eabi/release/encoder-firmware target/thumbv6m-none-eabi/release/encoder-firmware.uf2
	@echo "\nUF2 built successfully at: encoder-firmware/target/thumbv6m-none-eabi/release/encoder-firmware.uf2"

flash: build-firmware
	# We use probe-rs to flash the ELF without attaching a logger/debugger.
	cd encoder-firmware && probe-rs download --chip RP2040 --speed 10000 target/thumbv6m-none-eabi/debug/encoder-firmware
	# Reset the chip to let it boot and run normally.
	cd encoder-firmware && probe-rs reset --chip RP2040

flash-release: build-firmware-release
	# We use probe-rs to flash the release ELF without attaching a logger/debugger.
	cd encoder-firmware && probe-rs download --chip RP2040 --speed 10000 target/thumbv6m-none-eabi/release/encoder-firmware
	# Reset the chip to let it boot and run normally.
	cd encoder-firmware && probe-rs reset --chip RP2040

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
