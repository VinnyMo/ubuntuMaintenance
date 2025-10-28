# Makefile for Ubuntu Server Maintenance Tool
# Author: Vincent T. Mossman

# Compiler and flags
CC = gcc
CFLAGS = -Wall -Wextra -O2 -D_FORTIFY_SOURCE=2
LDFLAGS = -Wl,-z,relro,-z,now

# Installation directories
PREFIX ?= /usr
BINDIR = $(PREFIX)/bin
MANDIR = $(PREFIX)/share/man/man1
DOCDIR = $(PREFIX)/share/doc/ubuntu-maintenance

# Program name
PROGRAM = system_update
SOURCES = system_update.c utility_functions.c
HEADERS = utility_functions.h
OBJECTS = $(SOURCES:.c=.o)

# Version info
VERSION = 2.0.0

.PHONY: all clean install uninstall man

all: $(PROGRAM)

$(PROGRAM): $(OBJECTS)
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $^

%.o: %.c $(HEADERS)
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(PROGRAM) $(OBJECTS)
	rm -f system_update.1.gz
	rm -rf debian/ubuntu-maintenance
	rm -rf debian/.debhelper
	rm -f debian/files
	rm -f debian/*.log
	rm -f debian/*.substvars
	rm -f debian/debhelper-build-stamp

install: $(PROGRAM) man
	# Install binary
	install -D -m 0755 $(PROGRAM) $(DESTDIR)$(BINDIR)/$(PROGRAM)

	# Install man page
	install -D -m 0644 system_update.1.gz $(DESTDIR)$(MANDIR)/system_update.1.gz

	# Install documentation
	install -D -m 0644 README.md $(DESTDIR)$(DOCDIR)/README.md

	# Create log directory (will be owned by root)
	install -d -m 0755 $(DESTDIR)/var/log

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/$(PROGRAM)
	rm -f $(DESTDIR)$(MANDIR)/system_update.1.gz
	rm -rf $(DESTDIR)$(DOCDIR)

man: system_update.1.gz

system_update.1.gz: system_update.1
	gzip -9 -c system_update.1 > system_update.1.gz

# Development targets
test: $(PROGRAM)
	@echo "Running dry-run test..."
	./$(PROGRAM) --help

format:
	@echo "Code formatting (if clang-format is available)"
	@command -v clang-format >/dev/null 2>&1 && \
		clang-format -i $(SOURCES) $(HEADERS) || \
		echo "clang-format not found, skipping"

.SILENT: help
help:
	@echo "Ubuntu Server Maintenance Tool - Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  all        - Build the program (default)"
	@echo "  clean      - Remove built files"
	@echo "  install    - Install to system (requires root)"
	@echo "  uninstall  - Remove from system (requires root)"
	@echo "  man        - Generate compressed man page"
	@echo "  test       - Run basic tests"
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
