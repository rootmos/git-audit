VENV = $(shell readlink -f ./venv)
HOST_PYTHON ?= python3

export PYTHON = $(VENV)/bin/python3
export PIP = $(VENV)/bin/pip
export PYTEST = $(VENV)/bin/pytest

PYTEST_OPTS ?= -v

test: deps wait-for-rpc
ifdef TEST
	$(PYTEST) $(PYTEST_OPTS) "$(TEST)"
else
ifdef TESTS
	$(PYTEST) $(PYTEST_OPTS) -k "$(TESTS)"
else
	$(PYTEST) $(PYTEST_OPTS)
endif
endif

wait-for-rpc:
	@while ! curl --silent -X POST --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' "$(ETHEREUM_RPC_TARGET)" > /dev/null; do\
		sleep 1s; echo 1>&2 "waiting for: $(ETHEREUM_RPC_TARGET)"; \
	done

deps: .requirements.flag

$(VENV):
	$(HOST_PYTHON) -m venv $@

.requirements%flag: requirements%txt | $(VENV)
	$(PIP) install -r $<
	@touch $@

clean:
	rm -rf .*.flag $(VENV)

.PHONY: test deps clean
