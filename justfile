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
	install -Dm755 target/release/dotbackup {{prefix}}/bin/dotbackup
	install -Dm755 target/release/dotsetup {{prefix}}/bin/dotsetup
	install -Dm644 docs/dotbackup.1 {{prefix}}/share/man/man1/dotbackup.1
	install -Dm644 docs/dotsetup.1 {{prefix}}/share/man/man1/dotsetup.1
	install -Dm644 docs/dotbackup.5 {{prefix}}/share/man/man5/dotbackup.5

uninstall:
	rm -f {{prefix}}/bin/dotbackup
	rm -f {{prefix}}/bin/dotsetup
	rm -f {{prefix}}/share/man/man1/dotbackup.1
	rm -f {{prefix}}/share/man/man1/dotsetup.1
	rm -f {{prefix}}/share/man/man5/dotbackup.5
