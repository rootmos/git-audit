CARGO ?= cargo
SOLC ?= solc
DOCKER_COMPOSE ?= docker-compose

ifneq ($(VERBOSE),)
export V :=
else
export V := @
endif

BUILD ?= $(shell pwd)/build
export GIT_AUDIT_EXE ?= $(shell pwd)/git-audit

export ETHEREUM_RPC_PORT ?= 18545
export ETHEREUM_RPC_TARGET ?= http://localhost:$(ETHEREUM_RPC_PORT)

build: build-evm build-exe

build-exe: $(GIT_AUDIT_EXE)
.PHONY: $(GIT_AUDIT_EXE)
$(GIT_AUDIT_EXE):
	$(V)mkdir -p "$(BUILD)"
	$(V)$(CARGO) build --target-dir="$(BUILD)"
	$(V)install "$(BUILD)/debug/git-audit" "$@"

build-evm:
	$(V)mkdir -p "$(BUILD)/evm"
	$(V)$(SOLC) --optimize --overwrite --abi --bin -o "$(BUILD)/evm" evm/GitAudit.sol

test:
	$(V)$(MAKE) --no-print-directory -C tests

docs: build
	$(V)./readme.sh > README.md

clean:
	$(V)rm -rf "$(BUILD)"
	$(V)$(MAKE) -C tests clean

SERVICES ?= ethereum
run-services:
	$(V)$(DOCKER_COMPOSE) up --force-recreate $(SERVICES)

test-compose:
	$(V)$(DOCKER_COMPOSE) build
	$(V)$(DOCKER_COMPOSE) run tests

stop:
	$(V)$(DOCKER_COMPOSE) stop
	$(V)yes | $(DOCKER_COMPOSE) rm

.PHONY: build build-exe build-evm test docs clean
.PHONY: run-services stop
