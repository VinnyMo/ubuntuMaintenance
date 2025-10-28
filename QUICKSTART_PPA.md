# Quick Start: Publishing to PPA

This is a streamlined guide to get your package on a PPA in under 30 minutes.

## Prerequisites (One-Time Setup)

### 1. Install Build Tools (5 minutes)

```bash
sudo apt update
sudo apt install -y debhelper devscripts dh-make build-essential gnupg dput
```

### 2. Create Launchpad Account (5 minutes)

1. Go to https://launchpad.net and create account
2. Verify your email address

### 3. Generate and Upload GPG Key (10 minutes)

```bash
# Generate GPG key
gpg --full-generate-key

# Select these options:
# - Type: RSA and RSA
# - Size: 4096
# - Expiration: 0 (doesn't expire)
# - Name: Your Real Name
# - Email: your_email@example.com

# Get your key ID
gpg --list-keys
# Look for the line starting with "pub" - the ID is the long hex string

# Upload to Ubuntu keyserver
gpg --keyserver keyserver.ubuntu.com --send-keys YOUR_KEY_ID

# Export for Launchpad
gpg --armor --export your_email@example.com > my-key.asc
cat my-key.asc
```

4. Add key to Launchpad:
   - Go to https://launchpad.net/~YOUR_USERNAME/+editpgpkeys
   - Paste the key (including BEGIN/END lines)
   - Click "Import Key"
   - Check email and follow confirmation link

### 4. Create PPA (2 minutes)

1. Go to https://launchpad.net/~YOUR_USERNAME/+activate-ppa
2. Click "Create a new PPA"
3. Set URL to: `ubuntu-maintenance`
4. Add description and click "Activate"

## Uploading Your Package

### Step 1: Update Your Information

Edit these files with **YOUR** information:

**debian/control** - Line 4:
```
Maintainer: Your Name <your_email@example.com>
```

**debian/changelog** - Last line:
```
 -- Your Name <your_email@example.com>  Mon, 28 Oct 2025 04:00:00 +0000
```

**build-ppa.sh** - Lines 14-15:
```bash
MAINTAINER_NAME="Your Name"
MAINTAINER_EMAIL="your_email@example.com"
```

### Step 2: Configure dput

Create `~/.dput.cf` with your username:

```bash
cat > ~/.dput.cf << 'EOF'
[ppa]
fqdn = ppa.launchpad.net
method = ftp
incoming = ~YOUR_LAUNCHPAD_USERNAME/ubuntu-maintenance/ubuntu/
login = anonymous
allow_unsigned_uploads = 0
EOF
```

**Important:** Replace `YOUR_LAUNCHPAD_USERNAME` with your actual Launchpad username!

### Step 3: Build and Upload

```bash
# Option 1: Use the helper script (recommended)
./build-ppa.sh

# Option 2: Manual build
make clean
debuild -S -sa -d
dput ppa ../ubuntu-maintenance_2.0.0-1_source.changes
```

### Step 4: Monitor Build

1. Check your email for Launchpad notifications
2. Visit: https://launchpad.net/~YOUR_USERNAME/+archive/ubuntu/ubuntu-maintenance/+packages
3. Wait for build to complete (5-30 minutes)
4. Build status will show:
   - 🔵 Building (in progress)
   - ✅ Published (success)
   - ❌ Failed (check build log)

## Testing Your PPA

On any Ubuntu system:

```bash
# Add your PPA
sudo add-apt-repository ppa:YOUR_USERNAME/ubuntu-maintenance

# Install
sudo apt update
sudo apt install ubuntu-maintenance

# Test
system_update --help
sudo system_update --dry-run -a
```

## Building for Multiple Ubuntu Versions

For Ubuntu 20.04, 22.04, and 24.04:

```bash
./build-ppa.sh --all
```

Or individually:
```bash
./build-ppa.sh focal   # 20.04
./build-ppa.sh jammy   # 22.04
./build-ppa.sh noble   # 24.04
```

## Common Issues

### "No GPG key found"
```bash
# Make sure email matches
gpg --list-keys
# Email in GPG key must match debian/changelog and debian/control
```

### "Upload rejected - already exists"
```bash
# Increment version
dch -i
# Edit changelog, save, then rebuild
debuild -S -sa -d
```

### "Build failed on Launchpad"
```bash
# Test local build first
debuild -b
# Fix any errors, then upload again
```

### "gpg: signing failed"
```bash
export GPG_TTY=$(tty)
# Try again
debuild -S -sa -d
```

## Quick Reference Card

```bash
# Full workflow
./build-ppa.sh                    # Build and upload

# Build only (don't upload)
./build-ppa.sh --no-upload

# Build for all Ubuntu versions
./build-ppa.sh --all

# Manual workflow
make clean
debuild -S -sa -d
dput ppa ../ubuntu-maintenance_2.0.0-1_source.changes

# Check PPA status
https://launchpad.net/~YOUR_USERNAME/+archive/ubuntu/ubuntu-maintenance

# User installation
sudo add-apt-repository ppa:YOUR_USERNAME/ubuntu-maintenance
sudo apt install ubuntu-maintenance
```

## After Publishing

Share with users:

```bash
# Installation instructions
sudo add-apt-repository ppa:YOUR_USERNAME/ubuntu-maintenance
sudo apt update
sudo apt install ubuntu-maintenance

# Usage
sudo system_update --help
sudo system_update --dry-run -a  # Preview updates
sudo system_update -a             # Apply updates
```

## Updating the Package

For bug fixes:
```bash
dch -i                           # Opens changelog editor
# Add your changes, save
./build-ppa.sh                   # Build and upload
```

For new version:
```bash
dch -v 2.1.0-1                   # New version
# Add features to changelog, save
./build-ppa.sh                   # Build and upload
```

## Getting Help

- **Full guide**: Read PPA_GUIDE.md
- **Launchpad help**: https://help.launchpad.net/Packaging/PPA
- **Package issues**: Check build logs on Launchpad
- **Tool issues**: Check GitHub

---

**Time to PPA:** ~30 minutes (first time), ~2 minutes (updates)

**Status check:** https://launchpad.net/~YOUR_USERNAME/+archive/ubuntu/ubuntu-maintenance

**Your PPA URL:** ppa:YOUR_USERNAME/ubuntu-maintenance
