default:
    @just --list

run:
    cargo run --bin bradley

build:
    cargo build --release

check:
    cargo check

test:
    cargo test

clippy:
    cargo clippy --jobs 1

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

install-please:
    cargo install --path . --bin please

stop:
    cargo run --bin please -- help me
