FROM alpine:3.10 as evm
COPY --from=ethereum/solc:0.5.12 /usr/bin/solc /usr/bin/solc
RUN apk add --update make
WORKDIR /git-audit
ADD Makefile ./
ADD evm evm
RUN make build-evm

FROM rust:1.38.0-alpine3.10 as rust
RUN apk add --update make
WORKDIR /git-audit
ADD Makefile Cargo.toml Cargo.lock ./
ADD src src
RUN make build-rust

FROM python:3.7.4-alpine3.10 as tests
RUN apk add --update make gcc musl-dev libffi-dev libgit2-dev
WORKDIR /git-audit
ADD Makefile ./
ADD tests tests
ENTRYPOINT ["make", "test"]
