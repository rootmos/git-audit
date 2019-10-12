CARGO ?= cargo
SOLC ?= solc
BUILD ?= $(shell pwd)/build

export GIT_AUDIT_EXE = $(BUILD)/debug/git-audit

build: build-rust build-evm

build-rust:
	@mkdir -p "$(BUILD)"
	$(CARGO) build --target-dir="$(BUILD)"

build-evm:
	@mkdir -p "$(BUILD)/evm"
	$(SOLC) --optimize --overwrite --abi --bin -o "$(BUILD)/evm" evm/Mock.sol

test:
	$(MAKE) -C tests

clean:
	rm -rf "$(BUILD)"
	$(MAKE) -C tests clean

.PHONY: build build-rust build-evm test clean
