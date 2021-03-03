.PHONY: all
all: fmt lint build test schema

.PHONY: clean
clean:
	@cargo clean

.PHONY: fmt
fmt:
	@cargo fmt --all -- --check

.PHONY: lint
lint:
	@cargo clippy -- -D warnings

.PHONY: build
build:
	@cargo wasm

.PHONY: test
test:
	@RUST_BACKTRACE=1 cargo unit-test

.PHONY: schema
schema:
	@cargo schema

.PHONY: optimize
optimize:
	@docker run --rm -v $(CURDIR):/code \
		--mount type=volume,source=bilateral-exchange_cache,target=/code/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		cosmwasm/rust-optimizer:0.10.7

.PHONY: install
install: optimize
	@cp artifacts/bilateral_exchange.wasm $(PIO_HOME)