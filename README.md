# Ubuntu Server Maintenance Tool

A production-ready command-line utility for automated Ubuntu/Debian server maintenance. Designed for safe, logged, and auditable system updates with multiple operation modes.

**Author:** Vincent T. Mossman
**Version:** 2.1 (Schedule Management)
**License:** MIT

## Features

### Core Functionality
- **Multiple Update Modes**: Force (with reboot), All (no reboot), Critical (security only)
- **Automated Schedule Management** ⭐ NEW: Set up cron jobs without editing crontab
- **Dry-Run Mode**: Preview changes before applying them
- **System Information**: View comprehensive system and package status
- **Automatic Logging**: All operations logged to `/var/log/system_update.log`
- **Safe Reboots**: 5-minute delay with cancellation option
- **Interactive & CLI Modes**: Use interactively or script with command-line options

### Schedule Management (v2.1) ⭐ NEW
- **Easy Cron Setup**: No need to manually edit crontab
- **Multiple Frequencies**: Daily, weekly, or weekdays
- **Flexible Modes**: Choose all updates, critical only, or force with reboot
- **View Schedules**: See all configured update schedules in human-readable format
- **Remove Schedules**: Safely remove automated updates with confirmation
- **Automatic Logging**: Scheduled updates log to `/var/log/automated_updates.log`

### Security Improvements (v2.0)
- ✅ All memory leaks fixed
- ✅ Buffer overflow protection
- ✅ Input validation on all user input
- ✅ Comprehensive error handling
- ✅ Sudo privilege checking
- ✅ Command execution logging
- ✅ Safe string operations throughout

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
- Ubuntu 18.04 LTS or later (or Debian-based distribution)
- GCC compiler
- Root/sudo privileges for system updates

#### Using Make (Recommended)

```bash
cd /home/user/ubuntuMaintenance
make
sudo make install
```

This installs to `/usr/bin/system_update` with man pages.

#### Manual Compilation

```bash
cd /home/user/ubuntuMaintenance
gcc -Wall -Wextra -o system_update system_update.c utility_functions.c
sudo chmod +x system_update
```

#### Optional: Install System-Wide

```bash
sudo cp system_update /usr/local/bin/
sudo chown root:root /usr/local/bin/system_update
```

## Usage

### Interactive Mode

Run without arguments for an interactive menu:

```bash
sudo ./system_update
```

You'll see:
```
=== UBUNTU SERVER MAINTENANCE TOOL ===

1) Force Update (with reboot)
2) All Update (no reboot)
3) Critical Update (security only)
4) System Information
5) Help
0) Exit

Enter choice (0-5):
```

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
sudo ./system_update --dry-run -a

# Apply all updates without rebooting
sudo ./system_update --all

# Critical security updates only
sudo ./system_update --critical
```

**For Maintenance Windows:**
```bash
# Full update with scheduled reboot
sudo ./system_update --force

# Preview force update first
sudo ./system_update --dry-run -f
```

**System Monitoring:**
```bash
# View system status (no sudo required)
./system_update --info
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
sudo ./system_update --dry-run -a

# 2. Review the output

# 3. If acceptable, run for real
sudo ./system_update -a
```

## Production Deployment

### Recommended Setup

#### 1. Initial Installation
```bash
# Compile with all warnings enabled
gcc -Wall -Wextra -o system_update system_update.c utility_functions.c

# Test in dry-run mode first
sudo ./system_update --dry-run -a

# Run actual updates
sudo ./system_update -a
```

#### 2. Automated Updates (Cron)

For **automated weekly updates** without reboots:

```bash
sudo crontab -e
```

Add:
```cron
# Run updates every Sunday at 3 AM
0 3 * * 0 /usr/local/bin/system_update -a >> /var/log/automated_updates.log 2>&1
```

For **automated security updates only**:
```cron
# Run security updates daily at 2 AM
0 2 * * * /usr/local/bin/system_update -c >> /var/log/security_updates.log 2>&1
```

**Important:** Only use `-f` (force with reboot) in cron if you have redundancy/load balancing!

#### 3. Systemd Timer (Alternative to Cron)

Create `/etc/systemd/system/system-update.service`:
```ini
[Unit]
Description=Ubuntu System Update
After=network-online.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/system_update -a
StandardOutput=journal
StandardError=journal
```

Create `/etc/systemd/system/system-update.timer`:
```ini
[Unit]
Description=Weekly System Update Timer

[Timer]
OnCalendar=Sun 03:00
Persistent=true

[Install]
WantedBy=timers.target
```

Enable:
```bash
sudo systemctl enable system-update.timer
sudo systemctl start system-update.timer
```

### Logging

All operations are logged to `/var/log/system_update.log` with timestamps.

**View recent activity:**
```bash
sudo tail -f /var/log/system_update.log
```

**Log format:**
```
[Mon Oct 28 04:44:15 2025] === System Update Tool Started ===
[Mon Oct 28 04:44:15 2025] CMD: sudo apt update
[Mon Oct 28 04:45:32 2025] === ALL UPDATE completed ===
```

**Log rotation** (recommended):

Create `/etc/logrotate.d/system-update`:
```
/var/log/system_update.log {
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
   sudo ./system_update --dry-run -a
   ```

2. **Use the `-a` (all) mode** to control reboot timing:
   ```bash
   sudo ./system_update -a
   ```

3. **Schedule reboots during low-traffic periods:**
   ```bash
   # After updates complete, schedule reboot for off-hours
   sudo shutdown -r 02:00  # Reboot at 2 AM
   ```

### For Database Servers

1. **Use `-c` (critical) for minimal risk:**
   ```bash
   sudo ./system_update -c
   ```

2. **Schedule full updates during maintenance windows:**
   ```bash
   # During scheduled maintenance
   sudo ./system_update -a
   ```

3. **Never use `-f` (force with reboot)** without backups and redundancy.

### For Accounting Applications

Since this server hosts accounting applications, follow these guidelines:

1. **Backup first:**
   ```bash
   # Backup databases and application data
   sudo ./backup_script.sh

   # Then update
   sudo ./system_update -a
   ```

2. **Test in dry-run mode:**
   ```bash
   sudo ./system_update --dry-run -a
   ```

3. **Schedule during off-hours** (weekends, late night):
   ```bash
   # Use cron for automated weekend updates
   0 2 * * 6 /usr/local/bin/system_update -a
   ```

4. **Monitor logs after updates:**
   ```bash
   sudo tail -f /var/log/system_update.log
   ```

## Troubleshooting

### Permission Denied
```
ERROR: This program must be run with sudo privileges.
```
**Solution:** Run with `sudo`:
```bash
sudo ./system_update -a
```

### Cannot Open Log File
If you see warnings about log files:
```
WARNING: Cannot open log file
```
**Solution:** The tool falls back to `/tmp/system_update.log`. To fix permanently:
```bash
sudo touch /var/log/system_update.log
sudo chmod 640 /var/log/system_update.log
```

### APT Lock Issues
If updates fail due to another process using apt:
```bash
# Check what's using apt
sudo lsof /var/lib/dpkg/lock-frontend

# Wait for other process to finish, then retry
sudo ./system_update -a
```

### Cancel Scheduled Reboot
If you used `-f` and need to cancel the reboot:
```bash
sudo shutdown -c
```

## Monitoring & Alerts

### Check for Available Updates
```bash
./system_update -i
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
0 3 * * 0 /usr/local/bin/system_update -a
```

## Security Considerations

### What's Been Fixed in v2.0

1. **Memory Management:** All malloc() calls now have corresponding free() calls
2. **Buffer Overflows:** Replaced strcpy() with bounds-checked alternatives
3. **Input Validation:** All user input validated before processing
4. **Error Handling:** Comprehensive error checking on all system operations
5. **Code Quality:** Reduced from 405 lines to more maintainable, secure code
6. **Logging:** All operations logged for audit trails

### Safe Defaults

- Reboot delay: 5 minutes (not immediate)
- Dry-run mode available for preview
- Sudo checking prevents accidental non-root execution
- All system commands logged
- Error conditions reported and logged

## Comparison with Original

| Feature | Original (2019) | Production v2.0 |
|---------|-----------------|-----------------|
| Memory leaks | 6 leaks | ✅ 0 leaks |
| Buffer overflows | Vulnerable | ✅ Protected |
| Input validation | None | ✅ Full validation |
| Error handling | None | ✅ Comprehensive |
| Logging | None | ✅ Full audit trail |
| Dry-run mode | No | ✅ Yes |
| Help system | Incomplete | ✅ Complete |
| Information page | Under construction | ✅ Implemented |
| Reboot safety | Immediate | ✅ 5-min delay |
| Code efficiency | 200+ line function | ✅ 1 line |

## Support

### Logs Location
- Primary: `/var/log/system_update.log`
- Fallback: `/tmp/system_update.log`

### Bug Reports
Check logs and file issues with:
- Log excerpts
- Command used
- Expected vs. actual behavior

## Changelog

### Version 2.0 (2025-10-28)
- Complete security hardening
- Fixed all memory leaks
- Added buffer overflow protection
- Implemented dry-run mode
- Added comprehensive logging
- Implemented system information page
- Added safe reboot with 5-minute delay
- Full input validation
- Error handling throughout
- Proper header file structure
- Command-line option parsing
- Production-ready features

### Version 1.0 (2019-07-04)
- Initial release
- Basic update functionality
- Interactive menu

## License

Open source - use and modify as needed. Attribution appreciated.

---

**Maintained by:** Vincent T. Mossman
**Production Hardening:** 2025
**For:** Web servers hosting accounting applications and personal projects
