FROM alpine:3.10 as evm
COPY --from=ethereum/solc:0.5.12 /usr/bin/solc /usr/bin/solc
RUN apk add --update make
WORKDIR /git-audit
ADD Makefile ./
ADD evm evm
RUN make build-evm

FROM rust:1.38.0-slim-buster as rust
RUN apt-get update && apt-get install -y make libssl-dev pkg-config
WORKDIR /git-audit
COPY --from=evm /git-audit/build/evm build/evm
ADD Makefile Cargo.toml Cargo.lock ./
ADD src src
RUN make build-exe

FROM python:3.7.4-slim-buster as tests
RUN apt-get update && apt-get install -y make gcc
WORKDIR /git-audit
ADD Makefile ./
COPY --from=rust /git-audit/git-audit .
WORKDIR /git-audit/tests
ADD tests/requirements.txt tests/Makefile ./
RUN make deps
ADD tests /git-audit/tests
ENTRYPOINT ["make", "test", "GIT_AUDIT_EXE=/git-audit/git-audit"]
