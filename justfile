prefix := "/usr"

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

install:
	install -Dm755 -t {{prefix}}/bin target/release/dotbackup
	install -Dm755 -t {{prefix}}/bin target/release/dotsetup
	install -Dm644 -t {{prefix}}/share/man/man1 docs/dotbackup.1
	install -Dm644 -t {{prefix}}/share/man/man1 docs/dotsetup.1
	install -Dm644 -t {{prefix}}/share/man/man5 docs/dotbackup.5

uninstall:
	rm -f {{prefix}}/bin/dotbackup
	rm -f {{prefix}}/bin/dotsetup
	rm -f {{prefix}}/share/man/man1/dotbackup.1
	rm -f {{prefix}}/share/man/man1/dotsetup.1
	rm -f {{prefix}}/share/man/man5/dotbackup.5
