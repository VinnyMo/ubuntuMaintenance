# Ubuntu Maintenance

A Rust command-line tool for Ubuntu server maintenance, with a full-screen menu, repeatable update modes, cron scheduling, and logs you can inspect afterward.

**Current source version:** 3.1.11 · **Target platform:** Ubuntu 24.04 (Noble) · **License:** MIT

[Getting started](#getting-started) · [Update modes](#update-modes) · [Safety notes](#before-running-maintenance) · [Source guide](#source-guide)

## What it does

- Runs package updates and cleanup through APT
- Offers full-upgrade modes with or without a scheduled reboot
- Previews update commands with an explicit command-line dry run
- Shows system information, including update counts, storage, memory, and uptime
- Adds, displays, and removes maintenance cron schedules
- Keeps summary logs and captured command output, with a built-in verbose log browser

The current application is the Rust edition. Earlier C sources remain in the repository as project history.

## Before running maintenance

This tool can upgrade and remove packages, change cron entries, and reboot a machine. Review the commands and back up important data before using it on a server you depend on.

Important details in the current implementation:

- **`--critical` is not a security-only filter.** It runs `apt upgrade -y` against the machine's configured APT sources. The menu and CLI label describe security updates, but the implementation does not restrict upgrades to security packages.
- **Dry run previews commands, not APT's proposed package changes.** Use it with an update flag, such as `--dry-run --all`. It skips the update commands but still writes application logs. `--dry-run` alone enters the interactive menu, whose update actions do not retain that flag.
- **Check the logs for failures.** The current update routines continue after individual command errors, so a completion message alone does not establish that every step succeeded.
- **Force mode schedules a reboot after five minutes.** Use it only during a planned maintenance window. A pending reboot can be cancelled with `sudo shutdown -c`.

## Getting started

### Build from source

Use Ubuntu 24.04 with Git and a Rust/Cargo toolchain compatible with the checked-in dependencies. The repository includes vendored crates and Cargo configuration for the packaging workflow.

```bash
git clone https://github.com/VinnyMo/ubuntuMaintenance.git
cd ubuntuMaintenance
cargo build --release
./target/release/ubuntu-maintenance --help
```

To install the binary, man page, and README system-wide:

```bash
sudo make install
```

The Makefile's default prefix is `/usr`. Installation changes system files and requires appropriate privileges.

### PPA distribution

The project's PPA is [vinny-mossman/ubuntumaintenance on Launchpad](https://launchpad.net/~vinny-mossman/+archive/ubuntu/ubuntumaintenance). Check its published packages and Ubuntu series before installing; the version in Git may be newer than the packaged release.

```bash
sudo add-apt-repository ppa:vinny-mossman/ubuntumaintenance
sudo apt update
sudo apt install ubuntu-maintenance
```

Adding the PPA changes your system's package sources. If signature verification fails, inspect the PPA's current instructions rather than bypassing verification.

## Usage

Inspect the available options and system information:

```bash
ubuntu-maintenance --help
ubuntu-maintenance --version
ubuntu-maintenance --info
```

Preview a full update without applying its package commands:

```bash
sudo ubuntu-maintenance --dry-run --all
```

When you're ready to apply updates without an automatic reboot:

```bash
sudo ubuntu-maintenance --all
```

For the interactive menu, run `sudo ubuntu-maintenance`. Use the arrow keys and Enter to navigate; Esc or `q` returns from menus. The main areas are Run Updates, Manage Schedule, System Information, View Logs, and Help.

### Update modes

| Flag | Commands performed | Reboot |
| --- | --- | --- |
| `-a`, `--all` | `apt update`, `apt full-upgrade -y`, `apt autoremove -y`, `apt autoclean` | No automatic reboot |
| `-f`, `--force` | The same full update and cleanup sequence | Scheduled after five minutes |
| `-c`, `--critical` | `apt update`, `apt upgrade -y`, `apt autoclean` | No automatic reboot |
| `-d`, `--dry-run` | Prints the selected update mode's commands instead of executing them | None when paired with an update flag |

The `--critical` name is retained here to match the CLI; see the security-filter limitation above. Full upgrades and automatic removal can remove packages.

### Scheduling

The interactive Manage Schedule menu supports daily, weekly, and weekday jobs, with a choice of update mode and a confirmation before writing the schedule. Review the selected mode carefully, especially force mode's automatic reboot. Removing schedules removes the tool's entries from the current user's crontab.

### Logs and troubleshooting

- Summary log: `/var/log/ubuntu_maintenance.log`, with a `/tmp/ubuntu_maintenance.log` fallback
- Captured command output: `/var/log/ubuntu_maintenance_verbose.log`
- Scheduled runs: `/var/log/automated_updates.log`

For an APT lock error, let the other package operation finish before retrying. Don't remove lock files while a package manager is running. If a log cannot be opened, check its path and permissions.

When reporting a problem, include the Ubuntu version, command used, expected behavior, and relevant log excerpts. Remove hostnames, account details, or other private information before posting.

## Source guide

| Path | Purpose |
| --- | --- |
| [`src/main.rs`](src/main.rs) | CLI parsing, menus, update modes, and reboot flow |
| [`src/utils.rs`](src/utils.rs) | Terminal interaction, command execution, and system information |
| [`src/schedule.rs`](src/schedule.rs) | Cron schedule management |
| [`src/logger.rs`](src/logger.rs) | Summary and verbose logging |
| [`ubuntu-maintenance.1`](ubuntu-maintenance.1) | Man page |
| [`debian/`](debian/) | Debian packaging and release history |
| [`system_update.c`](system_update.c) | Earlier C implementation |

### Development checks

```bash
cargo fmt --check
cargo check
cargo clippy
make test
```

`make test` builds the release binary and checks its help/version output. It is not an end-to-end maintenance test. Do system-changing validation in a disposable Ubuntu environment before relying on a release.

## Project history

- **Original C edition:** early command-line and interactive update workflows
- **Rust rewrite:** CLI parsing, terminal interaction, logging, dry-run support, and delayed reboot handling
- **3.1.11:** consistent full-screen menus, a clearer system summary, readable schedule descriptions, and improvements to the verbose log browser

## License

MIT, as declared in [`Cargo.toml`](Cargo.toml). Packaging copyright and license information is in [`debian/copyright`](debian/copyright).
