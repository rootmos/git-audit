CARGO ?= cargo

export GIT_AUDIT_EXE = $(shell pwd)/target/debug/git-audit

build:
	$(CARGO) build

test: build
	$(MAKE) -C tests

clean:
	$(CARGO) clean
	$(MAKE) -C tests clean

.PHONY: build test clean
