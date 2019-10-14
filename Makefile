CARGO ?= cargo
SOLC ?= solc
DOCKER_COMPOSE ?= docker-compose

BUILD ?= $(shell pwd)/build
export GIT_AUDIT_EXE ?= $(shell pwd)/git-audit

export ETHEREUM_RPC_PORT ?= 18545
export ETHEREUM_RPC_TARGET ?= http://localhost:$(ETHEREUM_RPC_PORT)

build: build-evm build-exe

build-exe: $(GIT_AUDIT_EXE)
.PHONY: $(GIT_AUDIT_EXE)
$(GIT_AUDIT_EXE):
	@mkdir -p "$(BUILD)"
	$(CARGO) build --target-dir="$(BUILD)"
	install "$(BUILD)/debug/git-audit" "$@"

build-evm:
	@mkdir -p "$(BUILD)/evm"
	$(SOLC) --optimize --overwrite --abi --bin -o "$(BUILD)/evm" evm/GitAudit.sol

test:
	$(MAKE) -C tests

docs: build
	./readme.sh > README.md

clean:
	rm -rf "$(BUILD)"
	$(MAKE) -C tests clean

SERVICES ?= ethereum
run-services:
	$(DOCKER_COMPOSE) up --force-recreate $(SERVICES)

test-compose:
	$(DOCKER_COMPOSE) build
	$(DOCKER_COMPOSE) run tests

stop:
	$(DOCKER_COMPOSE) stop
	yes | $(DOCKER_COMPOSE) rm

.PHONY: build build-exe build-evm test docs clean
.PHONY: run-services stop
