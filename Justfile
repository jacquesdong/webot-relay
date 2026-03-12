run:
    cargo run

test:
    cargo test
    python -m doctest src/main.py -v

build:
    cargo build
