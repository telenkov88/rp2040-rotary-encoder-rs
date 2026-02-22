build:
	cargo build

build-encoder:
	cd encoder-firmware && cargo build

run:
	cd encoder-firmware && cargo run
