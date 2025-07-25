# justfile

set shell := ["pwsh", "-NoProfile", "-Command"]

test:
    cargo test --all -- --test-threads=1
