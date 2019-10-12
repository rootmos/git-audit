CARGO ?= cargo
SOLC ?= solc
BUILD ?= $(shell pwd)/build

export GIT_AUDIT_EXE = $(BUILD)/debug/git-audit

build:
	@mkdir -p "$(BUILD)/evm"
	$(SOLC) --optimize --overwrite --abi --bin -o "$(BUILD)/evm" evm/Mock.sol
	$(CARGO) build --target-dir="$(BUILD)"

test: build
	$(MAKE) -C tests

clean:
	rm -rf "$(BUILD)"
	$(MAKE) -C tests clean

.PHONY: build test clean
