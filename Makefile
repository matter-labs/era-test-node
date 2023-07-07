build-precompiles:
	cd etc/system-contracts && yarn; yarn install; yarn build; yarn build-bootloader
	./scripts/refresh_precompiles.sh

build-contracts:
	cd etc/system-contracts && yarn; yarn install; yarn build; yarn preprocess; yarn build-bootloader
	./scripts/refresh_contracts.sh

clean-contracts:
	cd etc/system-contracts && yarn clean
	rm -rf src/deps/contracts

rebuild-contracts:
	cd etc/system-contracts && yarn build; yarn preprocess; yarn build-bootloader
	./scripts/refresh_contracts.sh

rust-build:
	cargo build --release

all: build-contracts rust-build
clean: clean-contracts