build:
	cargo build --release

# Run a specific app
run app *args:
	cargo run --release --bin {{app}} -- {{args}}

# Runs a clippy check
check *args:
    cargo clippy {{args}}
