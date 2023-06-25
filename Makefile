build-contracts:
	cd etc/system-contracts && yarn; yarn install; yarn build; yarn preprocess; yarn build-bootloader

clean-contracts:
	cd etc/system-contracts && yarn clean

rebuild-contracts:
	cd etc/system-contracts && yarn build; yarn preprocess; yarn build-bootloader

rust-build:
	cargo build --release

all: build-contracts rust-build