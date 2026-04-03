# Ubuntu Server Maintenance Tool

A production-ready command-line utility for automated Ubuntu/Debian server maintenance. Designed for safe, logged, and auditable system updates with multiple operation modes.

**Author:** Vincent T. Mossman
**Version:** 3.1.10 (Rust Edition)
**License:** MIT
**Platform:** Ubuntu 24.04 (Noble) only

## Features

### Core Functionality
- **Multiple Update Modes**: Force (with reboot), All (no reboot), Critical (security only)
- **Automated Schedule Management**: Set up cron jobs for automated updates
- **Dry-Run Mode**: Preview changes before applying them
- **System Information**: View comprehensive system and package status
- **Automatic Logging**: All operations logged to `/var/log/ubuntu_maintenance.log`
- **Safe Reboots**: 5-minute delay with interactive cancellation
- **Interactive & CLI Modes**: Use interactively or script with command-line options

### Rust Edition Benefits (v3.x)
- ✅ **Memory Safety**: Compile-time guarantees, no leaks or buffer overflows
- ✅ **Modern CLI**: Type-safe argument parsing with `clap`
- ✅ **Cross-Platform Terminal**: Interactive features with `crossterm`
- ✅ **Robust Error Handling**: `Result` and `anyhow` for comprehensive error handling
- ✅ **Zero Runtime Overhead**: Performance equivalent to C with better safety
- ✅ **Production Ready**: Extensively tested on Ubuntu 24.04 (Noble)

## Installation

### Option 1: Install from PPA (Recommended)

The easiest way to install on Ubuntu:

```bash
# Add the PPA
sudo add-apt-repository ppa:vinny-mossman/ubuntumaintenance

# Update package lists
sudo apt update

# Install the package
sudo apt install ubuntu-maintenance
```

**If you get a GPG key error**, import the signing key first:

```bash
# Import the PPA signing key
sudo gpg --keyserver keyserver.ubuntu.com --recv-keys 7D382EA0DFF37F99
sudo gpg --export 7D382EA0DFF37F99 | sudo apt-key add -

# Then update and install
sudo apt update
sudo apt install ubuntu-maintenance
```

**Benefits of PPA installation:**
- Automatic updates via `apt upgrade`
- Properly installed man pages and documentation
- Automatic log file setup
- Easy removal with `apt remove`

### Option 2: Build from Source

#### Prerequisites
- **Ubuntu 24.04 (Noble) only** - Earlier versions not supported
- Rust toolchain 1.70+ (Noble's default rustc)
- Cargo build system
- Root/sudo privileges for system updates

**Why Noble only?** The Rust edition uses `clap 4.5` which requires Rust 1.70+. Only Ubuntu 24.04 ships with this version by default.

#### Using Make (Recommended)

```bash
cd ubuntuMaintenance
make
sudo make install
```

This installs:
- Binary to `/usr/bin/ubuntu-maintenance`
- Man page to `/usr/share/man/man1/ubuntu-maintenance.1.gz`
- Creates log file at `/var/log/ubuntu_maintenance.log`

#### Manual Compilation

```bash
cd ubuntuMaintenance
cargo build --release
sudo cp target/release/ubuntu-maintenance /usr/local/bin/
sudo chmod 755 /usr/local/bin/ubuntu-maintenance
```

#### Development Build

```bash
cargo build          # Debug build
cargo build --release # Optimized build
cargo fmt            # Format code
cargo clippy         # Run linter
```

## Usage

### Interactive Mode

Run without arguments for a guided full-screen menu:

```bash
sudo ubuntu-maintenance
```

The interactive flow is organized around five areas:
```
Main Menu
- Run Updates
- Manage Schedule
- System Information
- View Logs
- Help
- Exit
```

Navigation uses the arrow keys and `Enter`. `Esc` or `q` returns to the previous menu.

### Command-Line Mode

#### Available Options

| Option | Long Form | Description |
|--------|-----------|-------------|
| `-f` | `--force` | Full update with automatic reboot (5 min delay) |
| `-a` | `--all` | Full update without reboot |
| `-c` | `--critical` | Security updates only |
| `-i` | `--info` | Display system information |
| `-d` | `--dry-run` | Preview updates without applying |
| `-h` | `--help` | Display help message |

#### Examples

**Recommended for Production Servers:**
```bash
# Preview updates first (safe, read-only)
sudo ubuntu-maintenance --dry-run -a

# Apply all updates without rebooting
sudo ubuntu-maintenance --all

# Critical security updates only
sudo ubuntu-maintenance --critical
```

**For Maintenance Windows:**
```bash
# Full update with scheduled reboot
sudo ubuntu-maintenance --force

# Preview force update first
sudo ubuntu-maintenance --dry-run -f
```

**System Monitoring:**
```bash
# View system status
ubuntu-maintenance --info
```

## Update Modes Explained

### 1. Force Update (`-f`)
**Use when:** Performing complete system maintenance during a scheduled maintenance window.

**What it does:**
1. Updates package lists (`apt update`)
2. Installs all available upgrades (`apt full-upgrade -y`)
3. Removes obsolete packages (`apt autoremove -y`)
4. Cleans package cache (`apt autoclean`)
5. **Schedules system reboot in 5 minutes** (cancellable)

**Safety:** Reboot can be cancelled with `sudo shutdown -c` within 5 minutes.

### 2. All Update (`-a`)
**Use when:** Keeping servers up-to-date during normal operations. **Recommended for most cases.**

**What it does:**
1. Updates package lists (`apt update`)
2. Installs all available upgrades (`apt full-upgrade`)
3. Removes obsolete packages (`apt autoremove`)
4. Cleans package cache (`apt autoclean`)
5. **No automatic reboot** - you control when to restart

**Best for:** Web servers, database servers, production systems that need controlled reboots.

### 3. Critical Update (`-c`)
**Use when:** Applying only security patches between major updates.

**What it does:**
1. Updates package lists (`apt update`)
2. Installs security/critical upgrades only (`apt upgrade -y`)
3. Cleans package cache (`apt autoclean`)
4. **No automatic reboot**

**Best for:** Conservative update strategy, minimal downtime risk.

### 4. Dry-Run Mode (`-d`)
**Use when:** You want to preview changes before applying them. **Always use this first in production!**

**What it does:**
- Shows what commands would be executed
- Makes **no changes** to the system
- Safe to run anytime

**Example workflow:**
```bash
# 1. Preview the changes
sudo ubuntu-maintenance --dry-run -a

# 2. Review the output

# 3. If acceptable, run for real
sudo ubuntu-maintenance -a
```

## Production Deployment

### Recommended Setup

#### 1. Initial Installation
```bash
# Build optimized release binary
cargo build --release

# Test in dry-run mode first
sudo ./target/release/ubuntu-maintenance --dry-run -a

# Run actual updates
sudo ./target/release/ubuntu-maintenance -a

# Install system-wide
sudo make install
```

#### 2. Automated Updates (Cron)

For **automated weekly updates** without reboots:

```bash
sudo crontab -e
```

Add:
```cron
# Run updates every Sunday at 3 AM
0 3 * * 0 /usr/bin/ubuntu-maintenance -a >> /var/log/automated_updates.log 2>&1
```

For **automated security updates only**:
```cron
# Run security updates daily at 2 AM
0 2 * * * /usr/bin/ubuntu-maintenance -c >> /var/log/security_updates.log 2>&1
```

**Important:** Only use `-f` (force with reboot) in cron if you have redundancy/load balancing!

#### 3. Systemd Timer (Alternative to Cron)

Create `/etc/systemd/system/ubuntu-maintenance.service`:
```ini
[Unit]
Description=Ubuntu Maintenance Tool
After=network-online.target

[Service]
Type=oneshot
ExecStart=/usr/bin/ubuntu-maintenance -a
StandardOutput=journal
StandardError=journal
```

Create `/etc/systemd/system/ubuntu-maintenance.timer`:
```ini
[Unit]
Description=Weekly Ubuntu Maintenance Timer

[Timer]
OnCalendar=Sun 03:00
Persistent=true

[Install]
WantedBy=timers.target
```

Enable:
```bash
sudo systemctl enable ubuntu-maintenance.timer
sudo systemctl start ubuntu-maintenance.timer
```

### Logging

The tool writes two logs:

- `/var/log/ubuntu_maintenance.log` for high-level actions and command status
- `/var/log/ubuntu_maintenance_verbose.log` for captured command output used by the built-in log browser

**View recent activity:**
```bash
sudo tail -f /var/log/ubuntu_maintenance.log
```

**View detailed command output:**
```bash
sudo tail -f /var/log/ubuntu_maintenance_verbose.log
```

**Log format:**
```
[Fri Oct 31 19:45:00 2025] === Ubuntu Maintenance Tool Started ===
[Fri Oct 31 19:45:00 2025] CMD: apt update
[Fri Oct 31 19:45:32 2025] === ALL UPDATE completed ===
```

**Log rotation** (recommended):

Create `/etc/logrotate.d/ubuntu-maintenance`:
```
/var/log/ubuntu_maintenance.log {
    weekly
    rotate 12
    compress
    delaycompress
    notifempty
    create 0640 root root
}
```

## Best Practices for Production

### For Web Servers (Apache, Nginx, etc.)

1. **Always preview first:**
   ```bash
   sudo ubuntu-maintenance --dry-run -a
   ```

2. **Use the `-a` (all) mode** to control reboot timing:
   ```bash
   sudo ubuntu-maintenance -a
   ```

3. **Schedule reboots during low-traffic periods:**
   ```bash
   # After updates complete, schedule reboot for off-hours
   sudo shutdown -r 02:00  # Reboot at 2 AM
   ```

### For Database Servers

1. **Use `-c` (critical) for minimal risk:**
   ```bash
   sudo ubuntu-maintenance -c
   ```

2. **Schedule full updates during maintenance windows:**
   ```bash
   # During scheduled maintenance
   sudo ubuntu-maintenance -a
   ```

3. **Never use `-f` (force with reboot)** without backups and redundancy.

### For Accounting Applications

Since this server hosts accounting applications, follow these guidelines:

1. **Backup first:**
   ```bash
   # Backup databases and application data
   sudo ./backup_script.sh

   # Then update
   sudo ubuntu-maintenance -a
   ```

2. **Test in dry-run mode:**
   ```bash
   sudo ubuntu-maintenance --dry-run -a
   ```

3. **Schedule during off-hours** (weekends, late night):
   ```bash
   # Use cron for automated weekend updates
   0 2 * * 6 /usr/bin/ubuntu-maintenance -a
   ```

4. **Monitor logs after updates:**
   ```bash
   sudo tail -f /var/log/ubuntu_maintenance.log
   ```

## Troubleshooting

### Permission Denied
```
ERROR: This program must be run with sudo privileges.
```
**Solution:** Run with `sudo`:
```bash
sudo ubuntu-maintenance -a
```

### Cannot Open Log File
If you see warnings about log files:
```
WARNING: Cannot open log file
```
**Solution:** The tool falls back to `/tmp/ubuntu_maintenance.log`. To fix permanently:
```bash
sudo touch /var/log/ubuntu_maintenance.log
sudo chmod 640 /var/log/ubuntu_maintenance.log
```

### APT Lock Issues
If updates fail due to another process using apt:
```bash
# Check what's using apt
sudo lsof /var/lib/dpkg/lock-frontend

# Wait for other process to finish, then retry
sudo ubuntu-maintenance -a
```

### Cancel Scheduled Reboot
If you used `-f` and need to cancel the reboot:
```bash
sudo shutdown -c
```

## Monitoring & Alerts

### Check for Available Updates
```bash
sudo ubuntu-maintenance -i
```

Shows:
- Packages with updates available
- Security updates available
- Disk usage
- System uptime
- Last reboot time

### Email Alerts (Optional)

Modify cron to send email on completion:
```cron
MAILTO=admin@yourdomain.com
0 3 * * 0 /usr/bin/ubuntu-maintenance -a
```

## Security Considerations

### Rust Edition Benefits (v3.x)

1. **Memory Safety:** Compile-time guarantees eliminate memory leaks and buffer overflows
2. **Type Safety:** Strong typing prevents entire classes of bugs
3. **Modern Error Handling:** `Result` type forces explicit error handling
4. **No Undefined Behavior:** Rust's borrow checker prevents data races and null pointer dereferences
5. **Safe Concurrency:** Ownership system ensures thread safety
6. **Auditable:** All system commands logged for compliance

### Safe Defaults

- Reboot delay: 5 minutes with interactive cancellation
- Dry-run mode available for preview
- Sudo checking prevents accidental non-root execution
- All system commands logged to `/var/log/ubuntu_maintenance.log`
- Error conditions reported and logged
- Input validation on all user interactions

## Version History

| Feature | Original C (2019) | C v2.0 (2025) | Rust v3.x (2025) |
|---------|-------------------|---------------|------------------|
| Memory safety | ❌ 6 leaks | ✅ Fixed | ✅ Compile-time guaranteed |
| Buffer overflows | ❌ Vulnerable | ✅ Protected | ✅ Impossible by design |
| Input validation | ❌ None | ✅ Basic | ✅ Type-safe |
| Error handling | ❌ None | ✅ C-style | ✅ Result types |
| Logging | ❌ None | ✅ Basic | ✅ Structured |
| Dry-run mode | ❌ No | ✅ Yes | ✅ Yes |
| Interactive menu | ✅ Basic | ✅ Basic | ✅ Raw terminal mode |
| Schedule management | ❌ No | ✅ Added | ✅ Improved |
| Reboot safety | ❌ Immediate | ✅ 5-min delay | ✅ Interactive countdown |
| Platform support | Multiple | Multiple | Noble only (Rust 1.70+) |

## Support

### Resources
- **PPA:** `ppa:vinny-mossman/ubuntumaintenance`
- **GitHub:** https://github.com/VinnyMo/ubuntuMaintenance
- **Launchpad:** https://launchpad.net/~vinny-mossman/+archive/ubuntu/ubuntumaintenance

### Logs Location
- Primary: `/var/log/ubuntu_maintenance.log`
- Fallback: `/tmp/ubuntu_maintenance.log`
- Automated updates: `/var/log/automated_updates.log` (if using cron)

### Bug Reports
File issues on GitHub with:
- Log excerpts from `/var/log/ubuntu_maintenance.log`
- Command used
- Ubuntu version (`lsb_release -a`)
- Expected vs. actual behavior

## Changelog

### Version 3.1.10 (2026-04-03) - QOL Refresh
- Reworked the interactive navigation so menus share one consistent full-screen style
- Reformatted system information into a faster, easier-to-scan summary with short-lived caching
- Improved schedule viewing so cron jobs are explained in plain language
- Cleaned up the verbose log browser for more readable paging and controls
- Updated bundled help text, man pages, and packaging metadata to match the current app
- Release follow-up: finish the next PPA packaging and publish workflow after interactive validation

### Version 3.1.7 (2025-10-31) - Rust Edition
- Fix postinst/postrm scripts for proper log file handling
- Vendor all Rust dependencies for offline Launchpad builds
- Improved PPA build process with Cargo.lock v3 compatibility

### Version 3.1.0 (2025-10-29) - Rust Rewrite
- **Complete rewrite in Rust** for memory safety and security
- Compile-time memory safety guarantees (no leaks, no buffer overflows)
- Type-safe CLI parsing with `clap 4.5`
- Interactive terminal features with `crossterm`
- Robust error handling with `Result` and `anyhow`
- Target platform: Ubuntu 24.04 (Noble) only
- Binary renamed: `ubuntu-maintenance` (was `system_update`)
- Log file: `/var/log/ubuntu_maintenance.log` (was `system_update.log`)

### Version 2.1 (2025-10-28) - C Edition
- Added schedule management features
- Cron job automation support
- View and remove schedules

### Version 2.0 (2025-10-28) - C Edition
- Complete security hardening
- Fixed all memory leaks
- Added buffer overflow protection
- Implemented dry-run mode
- Added comprehensive logging
- Implemented system information page
- Added safe reboot with 5-minute delay

### Version 1.0 (2019-07-04) - Original C
- Initial release
- Basic update functionality
- Interactive menu

## License

MIT License - Open source, use and modify as needed. Attribution appreciated.

---

**Author:** Vincent T. Mossman
**Repository:** https://github.com/VinnyMo/ubuntuMaintenance
**PPA:** ppa:vinny-mossman/ubuntumaintenance
**Platform:** Ubuntu 24.04 (Noble)
**Language:** Rust (2025 rewrite from C)

**Production Ready:** Deployed on web servers hosting accounting applications, Node.js services, and personal projects.
