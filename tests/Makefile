VENV = $(shell readlink -f ./venv)
HOST_PYTHON ?= python3

export PYTHON = $(VENV)/bin/python3
export PIP = $(VENV)/bin/pip
export PYTEST = $(VENV)/bin/pytest

PYTEST_OPTS ?= -v

test: deps
ifdef TEST
	$(PYTEST) $(PYTEST_OPTS) "$(TEST)"
else
ifdef TESTS
	$(PYTEST) $(PYTEST_OPTS) -k "$(TESTS)"
else
	$(PYTEST) $(PYTEST_OPTS)
endif
endif

deps: .requirements.flag

$(VENV):
	$(HOST_PYTHON) -m venv $@

.requirements%flag: requirements%txt | $(VENV)
	$(PIP) install -r $<
	@touch $@

clean:
	rm -rf .*.flag $(VENV)

.PHONY: test deps clean
