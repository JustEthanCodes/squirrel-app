default:
    @just --list

run:
    cargo run --bin bradley

build:
    cargo build --release

check:
    cargo check --all-targets

test:
    cargo test

clippy:
    cargo clippy --all-targets

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

install-please:
    cargo install --path . --bin please

stop:
    cargo run --bin please -- help me
