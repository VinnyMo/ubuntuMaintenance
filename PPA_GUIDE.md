# PPA Distribution Guide for Ubuntu Maintenance Tool

This guide walks you through publishing the Ubuntu Maintenance Tool to a Personal Package Archive (PPA) on Launchpad for easy distribution to Ubuntu users.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Initial Launchpad Setup](#initial-launchpad-setup)
3. [GPG Key Setup](#gpg-key-setup)
4. [Creating Your PPA](#creating-your-ppa)
5. [Building the Source Package](#building-the-source-package)
6. [Uploading to PPA](#uploading-to-ppa)
7. [Testing the PPA](#testing-the-ppa)
8. [Updating the Package](#updating-the-package)
9. [Troubleshooting](#troubleshooting)

## Prerequisites

### Required Packages

Install the necessary tools on your Ubuntu system:

```bash
sudo apt update
sudo apt install -y \
    debhelper \
    devscripts \
    dh-make \
    build-essential \
    gnupg \
    dput
```

### Required Information

- **Launchpad Account**: Create at https://launchpad.net
- **Email Address**: Must match the one in debian/changelog
- **GPG Key**: For signing packages
- **PPA Name**: Choose a name for your PPA (e.g., "ubuntu-maintenance")

## Initial Launchpad Setup

### 1. Create Launchpad Account

1. Go to https://launchpad.net
2. Click "Log in / Register"
3. Create account with Ubuntu One
4. Complete your profile with your real name

### 2. Accept Terms of Use

1. Navigate to https://launchpad.net/+tour
2. Read and accept the terms of service

### 3. Set Up SSH Keys (Optional but Recommended)

```bash
# Generate SSH key if you don't have one
ssh-keygen -t rsa -b 4096 -C "your_email@example.com"

# Display your public key
cat ~/.ssh/id_rsa.pub
```

Add this to your Launchpad profile:
- Go to https://launchpad.net/~YOUR_USERNAME/+editsshkeys
- Paste your public key

## GPG Key Setup

### 1. Generate GPG Key

```bash
# Generate a new GPG key
gpg --full-generate-key
```

Select:
- Key type: (1) RSA and RSA
- Key size: 4096 bits
- Expiration: 0 (does not expire) or set your preference
- Real name: Your Name (must match debian/changelog)
- Email: your_email@example.com (must match debian/changelog)
- Comment: Ubuntu Maintenance Tool Signing Key

### 2. List Your Keys

```bash
gpg --list-keys
```

Output will look like:
```
pub   rsa4096 2025-10-28 [SC]
      1234567890ABCDEF1234567890ABCDEF12345678
uid           [ultimate] Your Name <your_email@example.com>
sub   rsa4096 2025-10-28 [E]
```

The long string (1234567890ABCDEF...) is your KEY_ID.

### 3. Export Public Key

```bash
# Replace KEY_ID with your actual key ID
gpg --keyserver keyserver.ubuntu.com --send-keys KEY_ID
```

### 4. Add GPG Key to Launchpad

```bash
# Export your public key in ASCII format
gpg --armor --export your_email@example.com > my-gpg-key.asc

# Display the key
cat my-gpg-key.asc
```

Add to Launchpad:
1. Go to https://launchpad.net/~YOUR_USERNAME/+editpgpkeys
2. Paste the entire GPG key (including BEGIN and END lines)
3. Click "Import Key"
4. Check your email for confirmation
5. Decrypt the confirmation message:
   ```bash
   gpg -d confirmation_email.txt
   ```
6. Follow the confirmation link

## Creating Your PPA

### 1. Create PPA on Launchpad

1. Go to https://launchpad.net/~YOUR_USERNAME/+activate-ppa
2. Click "Create a new PPA"
3. Fill in:
   - **URL**: ubuntu-maintenance (will be ppa:YOUR_USERNAME/ubuntu-maintenance)
   - **Display name**: Ubuntu Maintenance Tool
   - **Description**:
     ```
     Production-ready automated Ubuntu/Debian server maintenance tool.
     Provides safe, logged, and auditable system updates with multiple
     operation modes suitable for production servers.
     ```

4. Click "Activate"

### 2. Note Your PPA Address

Your PPA URL will be: `ppa:YOUR_USERNAME/ubuntu-maintenance`

## Building the Source Package

### 1. Update Maintainer Information

Edit `debian/changelog` and `debian/control` to use your information:

```bash
# Update your email in these files
nano debian/changelog  # Change maintainer email
nano debian/control    # Change maintainer email
```

**debian/changelog** first line should be:
```
ubuntu-maintenance (2.0.0-1) jammy; urgency=medium
```

And the last line should be:
```
 -- Your Name <your_email@example.com>  Mon, 28 Oct 2025 04:00:00 +0000
```

**debian/control** maintainer line:
```
Maintainer: Your Name <your_email@example.com>
```

### 2. Clean the Source

```bash
# Clean any previous builds
make clean
rm -rf ../ubuntu-maintenance_*
```

### 3. Build Source Package

For **Ubuntu 22.04 (Jammy)**:

```bash
# Build source package
debuild -S -sa -d

# If you get GPG errors, specify your key:
debuild -S -sa -d -kYOUR_KEY_ID
```

For **multiple Ubuntu versions**, you'll need to create separate uploads:

```bash
# For Ubuntu 22.04 (Jammy)
debuild -S -sa -d

# For Ubuntu 20.04 (Focal) - after building for Jammy
dch -v 2.0.0-1~focal1 "Backport to Focal"
dch -r ""
debuild -S -sd

# For Ubuntu 24.04 (Noble) - after building for Jammy
dch -v 2.0.0-1~noble1 "Build for Noble"
dch -r ""
debuild -S -sd
```

### 4. Verify Source Package

After building, you should have in the parent directory:

```bash
ls -la ../ | grep ubuntu-maintenance
```

Output should include:
```
ubuntu-maintenance_2.0.0-1.dsc
ubuntu-maintenance_2.0.0-1.tar.xz
ubuntu-maintenance_2.0.0-1_source.build
ubuntu-maintenance_2.0.0-1_source.changes
```

## Uploading to PPA

### 1. Configure dput

Create or edit `~/.dput.cf`:

```bash
cat > ~/.dput.cf << 'EOF'
[ppa]
fqdn = ppa.launchpad.net
method = ftp
incoming = ~YOUR_USERNAME/ubuntu-maintenance/ubuntu/
login = anonymous
allow_unsigned_uploads = 0
EOF
```

Replace `YOUR_USERNAME` with your actual Launchpad username.

### 2. Upload to PPA

```bash
# Upload the source package
dput ppa ../ubuntu-maintenance_2.0.0-1_source.changes
```

### 3. Check Upload Status

1. You'll receive an email from Launchpad (may take a few minutes)
2. Check build status at: https://launchpad.net/~YOUR_USERNAME/+archive/ubuntu/ubuntu-maintenance/+packages
3. Build process takes 5-30 minutes depending on queue

### 4. Monitor Build

Launchpad will build your package for:
- amd64 (64-bit)
- i386 (32-bit, if enabled)
- arm64 (ARM 64-bit)
- armhf (ARM 32-bit)

Check build logs if there are failures:
- Click on the architecture link
- View build log
- Fix issues and re-upload

## Testing the PPA

### On a Test System

```bash
# Add your PPA
sudo add-apt-repository ppa:YOUR_USERNAME/ubuntu-maintenance

# Update package lists
sudo apt update

# Install the package
sudo apt install ubuntu-maintenance

# Test it
system_update --help
sudo system_update --dry-run -a
```

### Verify Installation

```bash
# Check installed files
dpkg -L ubuntu-maintenance

# Check version
dpkg -l | grep ubuntu-maintenance

# Read man page
man system_update

# Check logs
ls -la /var/log/system_update.log
```

## Updating the Package

### For Bug Fixes (2.0.0-1 → 2.0.0-2)

```bash
# Update the code
nano system_update.c  # Make your fixes

# Update changelog
dch -i

# Add your changes to changelog, save

# Build and upload
debuild -S -sa -d
dput ppa ../ubuntu-maintenance_2.0.0-2_source.changes
```

### For New Version (2.0.0 → 2.1.0)

```bash
# Update the code
nano system_update.c  # Add new features

# Update version
dch -v 2.1.0-1

# Add your changes to changelog, save

# Build and upload
debuild -S -sa -d
dput ppa ../ubuntu-maintenance_2.1.0-1_source.changes
```

## Multi-Distribution Support

To support multiple Ubuntu versions (20.04, 22.04, 24.04):

### Script for Multi-Version Upload

Create `upload-to-ppa.sh`:

```bash
#!/bin/bash
# upload-to-ppa.sh - Build and upload for multiple Ubuntu versions

set -e

PACKAGE="ubuntu-maintenance"
VERSION="2.0.0"
DISTRIBUTIONS="focal jammy noble"

# Build for each distribution
for DIST in $DISTRIBUTIONS; do
    echo "Building for $DIST..."

    # Clean previous build
    make clean
    rm -f ../${PACKAGE}_${VERSION}*

    # Update changelog for this distribution
    DEBEMAIL="your_email@example.com" DEBFULLNAME="Your Name" \
        dch -v ${VERSION}-1~${DIST}1 -D ${DIST} "Build for ${DIST}"

    # Build source package
    debuild -S -sa -d

    # Upload
    dput ppa ../${PACKAGE}_${VERSION}-1~${DIST}1_source.changes

    echo "Uploaded for $DIST"
    echo "---"
done

echo "All distributions uploaded!"
```

Make it executable:
```bash
chmod +x upload-to-ppa.sh
./upload-to-ppa.sh
```

## User Installation Instructions

Once published, users can install with:

```bash
sudo add-apt-repository ppa:YOUR_USERNAME/ubuntu-maintenance
sudo apt update
sudo apt install ubuntu-maintenance
```

## PPA Description Template

For your PPA description on Launchpad, use:

```
Ubuntu Server Maintenance Tool - Production Ready

Automated Ubuntu/Debian server maintenance and update utility.

FEATURES:
• Multiple update modes: force (with reboot), all (no reboot), critical only
• Dry-run mode for safe preview before applying changes
• Comprehensive logging to /var/log/system_update.log
• Safe reboots with 5-minute cancellation window
• Interactive menu and command-line interface
• Production-ready with security hardening

SECURITY:
• All memory leaks fixed
• Buffer overflow protection
• Input validation
• Comprehensive error handling
• Full audit logging

USAGE:
sudo system_update [options]

Options:
  -f, --force       Force update with reboot
  -a, --all         All updates without reboot
  -c, --critical    Critical security updates only
  -i, --info        Display system information
  -d, --dry-run     Preview updates without applying
  -h, --help        Display help message

Perfect for web servers, database servers, and production environments.

Documentation: man system_update
GitHub: https://github.com/VinnyMo/ubuntuMaintenance
```

## Troubleshooting

### GPG Signing Errors

```bash
# Problem: gpg: signing failed: Inappropriate ioctl for device
export GPG_TTY=$(tty)

# Try again
debuild -S -sa -d
```

### Email Mismatch

```
Problem: Maintainer email doesn't match GPG key
Solution: Update debian/changelog and debian/control to match your GPG key email
```

### Build Failures

```bash
# Check build log on Launchpad
# Common issues:
# - Missing dependencies: Add to debian/control Build-Depends
# - Compilation errors: Test locally first with 'debuild -b'
```

### Upload Rejected

```
Problem: File already exists in PPA
Solution: Increment version number in debian/changelog and rebuild
```

### PPA Not Found

```bash
# Problem: E: Unable to locate package ubuntu-maintenance
# Wait 10-30 minutes after upload for build to complete
# Check build status on Launchpad
```

## Best Practices

1. **Always test locally first**: `debuild -b` to build locally
2. **Use dry-run**: Test with `--dry-run` before releasing
3. **Version carefully**: Follow semantic versioning
4. **Document changes**: Write clear changelog entries
5. **Sign everything**: Always sign packages with your GPG key
6. **Test on clean system**: Use VM or container for testing
7. **Support LTS versions**: Build for Ubuntu 20.04, 22.04, 24.04
8. **Monitor feedback**: Watch for bug reports on Launchpad

## Quick Reference

```bash
# Full workflow
make clean
debuild -S -sa -d
dput ppa ../ubuntu-maintenance_2.0.0-1_source.changes

# Check status
https://launchpad.net/~YOUR_USERNAME/+archive/ubuntu/ubuntu-maintenance

# User installation
sudo add-apt-repository ppa:YOUR_USERNAME/ubuntu-maintenance
sudo apt install ubuntu-maintenance
```

## Resources

- Launchpad PPA Guide: https://help.launchpad.net/Packaging/PPA
- Debian New Maintainer's Guide: https://www.debian.org/doc/manuals/maint-guide/
- Ubuntu Packaging Guide: https://packaging.ubuntu.com/html/
- GPG Documentation: https://gnupg.org/documentation/

## Support

For issues with:
- **The tool itself**: GitHub issues
- **PPA/packaging**: Launchpad answers
- **Installation**: Check /var/log/system_update.log

---

**Author:** Vincent T. Mossman
**License:** MIT
**Version:** 2.0.0
