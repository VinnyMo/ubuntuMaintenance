# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Ubuntu Maintenance Tool - A production-ready CLI utility for automated Ubuntu/Debian server maintenance, rewritten in Rust for memory safety and security. Provides safe, logged, and auditable system updates with multiple operation modes.

**Current Version:** 3.1.6 (Rust Edition)
**Target Platform:** Ubuntu 24.04 (Noble) only
**Distribution:** Launchpad PPA at `ppa:vinny-mossman/ubuntumaintenance`

## Building and Development

### Local Development

```bash
# Build release binary
cargo build --release

# Format code
cargo fmt

# Run lints and checks
cargo check
cargo clippy

# Build with Make (also builds man page)
make

# Install locally for testing
sudo make install
```

### Testing

```bash
# Run basic tests
make test

# Test the binary directly
./target/release/ubuntu-maintenance --help
./target/release/ubuntu-maintenance --version

# Test dry-run mode (safe, no changes)
sudo ./target/release/ubuntu-maintenance --dry-run -a
```

## PPA Packaging and Launchpad Builds

### Critical Requirements for Launchpad Builds

**IMPORTANT:** Launchpad build environment has **NO network access** and uses **older Rust toolchains**. This requires:

1. **Vendored Dependencies** - All Rust crates must be vendored into the source package
2. **Cargo.lock Version 3** - Launchpad's Rust doesn't support v4 lockfiles
3. **Clean Git Tree** - No local modifications or IDE files (`.vscode`, etc.)

### Building for PPA Upload

```bash
# 1. Ensure Cargo.lock is version 3 (not v4)
rm Cargo.lock
cargo +1.76 generate-lockfile  # Use Rust 1.76 to generate v3 format

# 2. Vendor all dependencies (required for offline builds)
cargo vendor

# 3. Configure Cargo to use vendored sources
mkdir -p .cargo
cat > .cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

# 4. Update version numbers in:
#    - Cargo.toml
#    - build-ppa.sh (VERSION variable)
#    - debian/changelog (add new entry at top)

# 5. Regenerate Cargo.lock with version 3 after version bump
rm Cargo.lock && cargo +1.76 generate-lockfile

# 6. Commit all changes (vendor/, .cargo/, Cargo.lock, version bumps)
git add -A
git commit -m "Version X.Y.Z: Description"
git push

# 7. Remove any local IDE files that aren't .gitignored
rm -rf .vscode  # If present

# 8. Build source package
./build-ppa.sh noble  # or ./build-ppa.sh -n noble for no-upload

# 9. Sign and upload (if not using -n flag, script will prompt)
# Files created in parent directory: ../*.changes, ../*.dsc, etc.
```

### Build Script Options

```bash
./build-ppa.sh noble           # Build for Noble, prompt before upload
./build-ppa.sh -n noble        # Build only, don't upload
./build-ppa.sh -r 2 noble      # Set Debian revision to 2
./build-ppa.sh --help          # Show all options
```

### Common Build Issues

**Issue: `Cargo.lock version 4 requires -Znext-lockfile-bump`**
- **Cause:** Cargo.lock was generated with newer Rust (1.77+)
- **Fix:** Regenerate with Rust 1.76: `rm Cargo.lock && cargo +1.76 generate-lockfile`

**Issue: `Couldn't resolve host: index.crates.io`**
- **Cause:** Missing vendored dependencies
- **Fix:** Run `cargo vendor` and ensure `.cargo/config.toml` exists

**Issue: `local changes detected: ubuntuMaintenance/.vscode/...`**
- **Cause:** IDE files in working directory but not in git tarball
- **Fix:** `rm -rf .vscode` before building

**Issue: `Unexpected end of input` when extracting tarball**
- **Cause:** Tarball creation still in progress when build starts
- **Fix:** Recreate tarball: `rm ../ubuntu-maintenance_*.orig.tar.xz` and rebuild

### Version Management

When bumping versions:

1. Update `Cargo.toml` version
2. Update `VERSION` in `build-ppa.sh`
3. Add new entry to `debian/changelog`:
   ```
   ubuntu-maintenance (X.Y.Z-1~noble1) noble; urgency=medium

     * Change description here
     * Another change

    -- Vincent T. Mossman <vincent.mossman@gmail.com>  Date +0000
   ```
4. Regenerate Cargo.lock with version 3
5. Commit and tag: `git tag vX.Y.Z && git push --tags`

## Code Architecture

### Module Structure

```
src/
├── main.rs      - CLI argument parsing, interactive menu, main loop
├── utils.rs     - System utilities (sudo check, formatted output, user input)
├── logger.rs    - Logging to /var/log/ubuntu_maintenance.log
└── schedule.rs  - Cron schedule management for automated updates
```

### Key Design Patterns

**Update Modes:**
- `force_update()` - Full upgrade + autoremove + 5-min delayed reboot
- `all_update()` - Full upgrade + autoremove, no reboot
- `critical_update()` - Security updates only

**Dry-Run System:**
- All update functions check `state.dry_run`
- When enabled, prints commands instead of executing
- Uses `utils::tell_system_with_progress()` for actual execution

**Logging:**
- All operations logged to `/var/log/ubuntu_maintenance.log`
- Falls back to `/tmp/ubuntu_maintenance.log` if no write access
- Uses `logger::log_command()` to track all system commands

**Safe Reboots:**
- Uses `shutdown -r +5` for 5-minute delay
- Interactive countdown with cancellation via 'c' key (crossterm)
- Can be cancelled with `sudo shutdown -c`

### Dependencies

**Critical Dependency Versions:**
- `clap = "4.5"` - Requires Rust 1.70+ (this is why only Noble is supported)
- `crossterm = "0.28"` - For interactive terminal features
- `colored = "2.1"` - Terminal color output
- `chrono = "0.4"` - Date/time formatting
- `nix = "0.29"` - Unix system calls (sudo check)

**Debian Build Dependencies:**
- `rustc >= 1.70` - Required for clap 4.x
- Only Ubuntu 24.04 (Noble) has rustc 1.70+
- Earlier Ubuntu versions (22.04, 20.04) not supported

## Debian Packaging

### Package Files

```
debian/
├── changelog       - Version history (must be updated for each release)
├── control         - Package metadata, dependencies
├── rules           - Build instructions (uses cargo build --release)
├── install         - File installation mappings
├── copyright       - License information
├── postinst        - Post-installation script (creates log file)
└── postrm          - Post-removal cleanup
```

### Build Process Flow

1. `debuild` reads `debian/changelog` for version
2. Creates `orig.tar.xz` from git archive (includes vendor/)
3. Runs `debian/rules` which calls `cargo build --release`
4. Man page compressed: `ubuntu-maintenance.1.gz`
5. Signs with GPG key
6. Uploads `.changes`, `.dsc`, `.tar.xz` to Launchpad

### Launchpad Build Environment

- Ubuntu Noble chroot with no network access
- Has rustc 1.70, cargo, debhelper
- Unpacks `orig.tar.xz` and `debian.tar.xz`
- Runs `cargo build --release` using vendored dependencies
- Build logs: https://launchpad.net/~vinny-mossman/+archive/ubuntu/ubuntumaintenance/+packages

## Interactive Features

### Menu System (main.rs)

The interactive menu uses raw terminal mode (crossterm) for immediate key response without Enter. Main loop:

1. Display menu
2. Read single keypress
3. Execute selected function
4. Return to menu

### Countdown System (utils.rs)

The reboot countdown uses:
- `crossterm::event::poll()` for non-blocking key detection
- Raw mode for immediate 'c' key detection
- Updates every second with time remaining
- Executes `sudo shutdown -c` on cancellation

## Logging and Monitoring

```bash
# View logs
sudo tail -f /var/log/ubuntu_maintenance.log

# Check automated update logs (if using cron)
sudo tail -f /var/log/automated_updates.log

# View log path used by tool
ubuntu-maintenance  # Shows log path in header
```

**Log Format:**
```
[Fri Oct 31 19:45:00 2025] === Ubuntu Maintenance Tool Started ===
[Fri Oct 31 19:45:00 2025] CMD: apt update
[Fri Oct 31 19:45:32 2025] === ALL UPDATE completed ===
```

## Production Deployment

The tool is designed for production Ubuntu servers running web apps, databases, etc.

**Common Usage:**
```bash
# Preview before applying (recommended)
sudo ubuntu-maintenance --dry-run -a

# Apply all updates, no reboot
sudo ubuntu-maintenance -a

# Security updates only
sudo ubuntu-maintenance -c

# Full update with reboot (maintenance window)
sudo ubuntu-maintenance -f
```

**Automated Updates via Cron:**
```cron
# Weekly full updates, Sundays at 3 AM
0 3 * * 0 /usr/bin/ubuntu-maintenance -a >> /var/log/automated_updates.log 2>&1

# Daily security updates
0 2 * * * /usr/bin/ubuntu-maintenance -c >> /var/log/security_updates.log 2>&1
```

## Lessons Learned from Launchpad Builds

### Must-Have for Successful Builds

1. **Always vendor dependencies** - `cargo vendor` is non-negotiable
2. **Use Cargo.lock v3** - Generated with `cargo +1.76 generate-lockfile`
3. **Clean working directory** - No `.vscode/`, temporary files, or local changes
4. **Test tarball extraction** - Ensure `git archive` completes before building
5. **Match orig.tar.xz to git state** - Cargo.lock in tarball must match working tree

### Common Pitfalls

- ❌ Forgetting to vendor after adding dependencies
- ❌ Using modern Rust to generate Cargo.lock (creates v4)
- ❌ Building while tarball creation still in progress
- ❌ Having local IDE files that differ from git
- ❌ Not updating all version numbers consistently

### Build Verification Checklist

Before uploading to PPA:
- [ ] Cargo.lock is version 3 (`head -3 Cargo.lock`)
- [ ] vendor/ directory exists and is committed
- [ ] .cargo/config.toml configured for vendored sources
- [ ] All version numbers updated (Cargo.toml, build-ppa.sh, debian/changelog)
- [ ] No .vscode or IDE files in working directory
- [ ] `cargo build --release` works locally with vendored deps
- [ ] Git tree is clean (`git status`)

## Git Repository

**GitHub:** https://github.com/VinnyMo/ubuntuMaintenance
**Branch:** `rust-rewrite` (main development)
**Remote:** SSH (git@github.com:VinnyMo/ubuntuMaintenance.git)

## Historical Context

This is a **Rust rewrite** of an original C version (2019). The C version had memory leaks, buffer overflows, and minimal error handling. The Rust version provides:

- Compile-time memory safety (no leaks, no buffer overflows)
- Modern error handling with `Result` and `anyhow`
- Type-safe command parsing with `clap`
- Cross-platform terminal handling with `crossterm`
- Zero runtime overhead compared to C

The README mentions v2.0 (C) and v2.1 (schedule features), but the actual shipped version is v3.x (Rust edition).
