CARGO ?= cargo
SOLC ?= solc
BUILD ?= $(shell pwd)/build

export GIT_AUDIT_EXE ?= $(shell pwd)/git-audit

build: build-exe build-evm

build-exe: $(GIT_AUDIT_EXE)
$(GIT_AUDIT_EXE):
	@mkdir -p "$(BUILD)"
	$(CARGO) build --release --target-dir="$(BUILD)"
	install "$(BUILD)/release/git-audit" "$@"

build-evm:
	@mkdir -p "$(BUILD)/evm"
	$(SOLC) --optimize --overwrite --abi --bin -o "$(BUILD)/evm" evm/Mock.sol

test:
	$(MAKE) -C tests

clean:
	rm -rf "$(BUILD)"
	$(MAKE) -C tests clean

.PHONY: build build-exe build-evm test clean
