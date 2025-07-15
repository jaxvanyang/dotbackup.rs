build: doc
	cargo build --release

# Run a specific app
run app *args:
	cargo run --release --bin {{app}} -- {{args}}

# Runs a clippy check
check *args:
	cargo clippy {{args}}

# Build man pages
doc:
	scdoc < docs/dotbackup.1.scdoc > docs/dotbackup.1
	scdoc < docs/dotsetup.1.scdoc > docs/dotsetup.1
	scdoc < docs/dotbackup.5.scdoc > docs/dotbackup.5
