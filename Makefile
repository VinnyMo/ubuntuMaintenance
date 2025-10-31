# Makefile for Ubuntu Server Maintenance Tool (Rust Edition)
# Author: Vincent T. Mossman

# Installation directories
PREFIX ?= /usr
BINDIR = $(PREFIX)/bin
MANDIR = $(PREFIX)/share/man/man1
DOCDIR = $(PREFIX)/share/doc/ubuntu-maintenance

# Program name and paths
PROGRAM = ubuntu-maintenance
RUST_BINARY = target/release/$(PROGRAM)
MANPAGE = ubuntu-maintenance.1

# Version info
VERSION = 3.1.2

.PHONY: all build clean install uninstall man test help

all: build

build:
	@echo "Building Rust binary..."
	cargo build --release

clean:
	@echo "Cleaning build artifacts..."
	-cargo clean 2>/dev/null || true
	rm -f $(MANPAGE).gz
	rm -rf debian/ubuntu-maintenance
	rm -rf debian/.debhelper
	rm -f debian/files
	rm -f debian/*.log
	rm -f debian/*.substvars
	rm -f debian/debhelper-build-stamp

install: build man
	@echo "Installing $(PROGRAM)..."
	# Install binary
	install -D -m 0755 $(RUST_BINARY) $(DESTDIR)$(BINDIR)/$(PROGRAM)

	# Install man page
	install -D -m 0644 $(MANPAGE).gz $(DESTDIR)$(MANDIR)/$(PROGRAM).1.gz

	# Install documentation
	install -D -m 0644 README.md $(DESTDIR)$(DOCDIR)/README.md

	# Create log directory (will be owned by root)
	install -d -m 0755 $(DESTDIR)/var/log

uninstall:
	@echo "Uninstalling $(PROGRAM)..."
	rm -f $(DESTDIR)$(BINDIR)/$(PROGRAM)
	rm -f $(DESTDIR)$(MANDIR)/$(PROGRAM).1.gz
	rm -rf $(DESTDIR)$(DOCDIR)

man: $(MANPAGE).gz

$(MANPAGE).gz: $(MANPAGE)
	gzip -9 -c $(MANPAGE) > $(MANPAGE).gz

# Development targets
test: build
	@echo "Running tests..."
	@echo ""
	@echo "Testing help command:"
	./$(RUST_BINARY) --help
	@echo ""
	@echo "Testing version command:"
	./$(RUST_BINARY) --version

format:
	@echo "Formatting Rust code..."
	cargo fmt

check:
	@echo "Running Rust checks..."
	cargo check
	cargo clippy

.SILENT: help
help:
	@echo "Ubuntu Server Maintenance Tool - Makefile (Rust Edition)"
	@echo ""
	@echo "Targets:"
	@echo "  all        - Build the program (default)"
	@echo "  build      - Build Rust binary with cargo"
	@echo "  clean      - Remove built files"
	@echo "  install    - Install to system (requires root)"
	@echo "  uninstall  - Remove from system (requires root)"
	@echo "  man        - Generate compressed man page"
	@echo "  test       - Run basic tests"
	@echo "  format     - Format Rust code"
	@echo "  check      - Run Rust checks and lints"
	@echo "  help       - Show this help message"
	@echo ""
	@echo "Installation directories:"
	@echo "  PREFIX     = $(PREFIX)"
	@echo "  BINDIR     = $(BINDIR)"
	@echo "  MANDIR     = $(MANDIR)"
	@echo "  DOCDIR     = $(DOCDIR)"
	@echo ""
	@echo "Examples:"
	@echo "  make                    # Build the program"
	@echo "  sudo make install       # Install system-wide"
	@echo "  make PREFIX=/usr/local  # Use different prefix"
	@echo "  make test               # Run tests"
