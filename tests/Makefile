VENV = $(shell readlink -f ./venv)
HOST_PYTHON ?= python3

export PYTHON = $(VENV)/bin/python3
export PIP = $(VENV)/bin/pip
export PYTEST = $(VENV)/bin/pytest

ifneq ($(VERBOSE),)
PYTEST_OPTS += --verbose
else
PYTEST_OPTS += --quiet
endif

test: deps wait-for-rpc
ifdef TEST
	$(V)$(PYTEST) $(PYTEST_OPTS) "$(TEST)"
else
ifdef TESTS
	$(V)$(PYTEST) $(PYTEST_OPTS) -k "$(TESTS)"
else
	$(V)$(PYTEST) $(PYTEST_OPTS)
endif
endif

wait-for-rpc:
	$(V)while ! curl --silent -X POST --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' "$(ETHEREUM_RPC_TARGET)" > /dev/null; do\
		sleep 1s; echo 1>&2 "waiting for: $(ETHEREUM_RPC_TARGET)"; \
	done

deps: .requirements.flag

$(VENV):
	$(HOST_PYTHON) -m venv $@

.requirements%flag: requirements%txt | $(VENV)
	$(PIP) install -r $<
	$(V)touch $@

clean:
	rm -rf .*.flag $(VENV)

.PHONY: test deps clean
