# Step-by-Step Guide: Publish to PPA and Install

This guide will walk you through publishing ubuntu-maintenance to your PPA and installing it on your machine.

**Estimated time:** 30-45 minutes (first time)

---

## Part 1: Setup (One-Time Only) - 20 minutes

### Step 1: Install Required Tools (2 minutes)

```bash
sudo apt update
sudo apt install -y debhelper devscripts dh-make build-essential gnupg dput
```

**What this does:** Installs all the Debian packaging and signing tools you need.

---

### Step 2: Create Launchpad Account (3 minutes)

1. Go to https://launchpad.net
2. Click **"Log in / Register"**
3. Sign in with Ubuntu One (or create account)
4. Complete your profile with:
   - Your real name
   - Email address
   - Username (write this down!)

**Example:** If you chose username "vinnym", your PPA will be `ppa:vinnym/ubuntu-maintenance`

---

### Step 3: Generate GPG Key (5 minutes)

```bash
# Generate the key
gpg --full-generate-key
```

**Answer the prompts:**
- Key type: **1** (RSA and RSA)
- Key size: **4096**
- Expiration: **0** (doesn't expire) or your preference
- Real name: **Your Real Name** (must match what you'll use in debian files)
- Email: **your_email@example.com** (must match what you'll use)
- Comment: **Ubuntu Maintenance Tool Signing Key**
- Passphrase: Enter a secure passphrase (you'll need this!)

```bash
# List your keys to get the Key ID
gpg --list-secret-keys --keyid-format LONG
```

**Example output:**
```
sec   rsa4096/1234567890ABCDEF 2025-10-28 [SC]
      ABCD1234567890ABCDEF1234567890ABCDEF1234
uid                 [ultimate] Your Name <your_email@example.com>
```

The Key ID is the part after `rsa4096/` → **1234567890ABCDEF**

**Write down your Key ID!** You'll need it.

---

### Step 4: Upload GPG Key to Ubuntu Keyserver (3 minutes)

```bash
# Replace YOUR_KEY_ID with your actual key ID from above
gpg --keyserver keyserver.ubuntu.com --send-keys YOUR_KEY_ID
```

**Example:**
```bash
gpg --keyserver keyserver.ubuntu.com --send-keys 1234567890ABCDEF
```

---

### Step 5: Add GPG Key to Launchpad (5 minutes)

```bash
# Export your public key
gpg --armor --export your_email@example.com > my-gpg-key.asc

# Display it
cat my-gpg-key.asc
```

**Copy the entire output** (including BEGIN and END lines)

1. Go to https://launchpad.net/~YOUR_USERNAME/+editpgpkeys
2. Paste your key in the text box
3. Click **"Import Key"**
4. **Check your email** - Launchpad will send an encrypted confirmation
5. Save the email to a file called `confirmation.txt`
6. Decrypt it:
   ```bash
   gpg -d confirmation.txt
   ```
7. **Click the confirmation link** shown in the decrypted message

**You should see:** "OpenPGP key confirmed"

---

### Step 6: Create Your PPA (2 minutes)

1. Go to https://launchpad.net/~YOUR_USERNAME/+activate-ppa
2. Click **"Create a new PPA"**
3. Fill in:
   - **URL:** `ubuntu-maintenance`
   - **Display name:** `Ubuntu Server Maintenance Tool`
   - **Description:**
     ```
     Production-ready automated Ubuntu server maintenance tool with schedule management.

     Features:
     - Multiple update modes (force, all, critical)
     - Automated schedule management (NEW in v2.1!)
     - Dry-run mode for safety
     - Comprehensive logging
     - Safe reboots with 5-minute delay

     Perfect for web servers, database servers, and production environments.
     ```
4. Click **"Activate"**

**Your PPA address is now:** `ppa:YOUR_USERNAME/ubuntu-maintenance`

---

## Part 2: Prepare Your Package (10 minutes)

### Step 7: Update Maintainer Information (5 minutes)

You need to update three files with **your actual information**.

#### File 1: debian/changelog

```bash
cd /home/user/ubuntuMaintenance
nano debian/changelog
```

Change the **last line** of the first entry (line 15):
```
 -- Your Real Name <your_email@example.com>  Mon, 28 Oct 2025 06:00:00 +0000
```

**Important:**
- Name must match your GPG key name
- Email must match your GPG key email
- Keep the exact date/time format

**Save:** Ctrl+O, Enter, Ctrl+X

---

#### File 2: debian/control

```bash
nano debian/control
```

Change line 4:
```
Maintainer: Your Real Name <your_email@example.com>
```

**Save:** Ctrl+O, Enter, Ctrl+X

---

#### File 3: build-ppa.sh

```bash
nano build-ppa.sh
```

Change lines 18-19:
```bash
MAINTAINER_NAME="Your Real Name"
MAINTAINER_EMAIL="your_email@example.com"
```

**Save:** Ctrl+O, Enter, Ctrl+X

---

### Step 8: Configure dput (2 minutes)

```bash
nano ~/.dput.cf
```

**Paste this** (replace YOUR_LAUNCHPAD_USERNAME):
```
[ppa]
fqdn = ppa.launchpad.net
method = ftp
incoming = ~YOUR_LAUNCHPAD_USERNAME/ubuntu-maintenance/ubuntu/
login = anonymous
allow_unsigned_uploads = 0
```

**Example:** If your Launchpad username is "vinnym":
```
incoming = ~vinnym/ubuntu-maintenance/ubuntu/
```

**Save:** Ctrl+O, Enter, Ctrl+X

---

### Step 9: Set Environment Variables (1 minute)

```bash
# Add these to your current session
export DEBEMAIL="your_email@example.com"
export DEBFULLNAME="Your Real Name"
export GPG_TTY=$(tty)

# Optional: Add to ~/.bashrc to make permanent
echo 'export DEBEMAIL="your_email@example.com"' >> ~/.bashrc
echo 'export DEBFULLNAME="Your Real Name"' >> ~/.bashrc
echo 'export GPG_TTY=$(tty)' >> ~/.bashrc
```

---

## Part 3: Build and Upload (10 minutes)

### Step 10: Build the Source Package (5 minutes)

```bash
cd /home/user/ubuntuMaintenance

# Clean previous builds
make clean
rm -f ../*.deb ../*.dsc ../*.tar.* ../*.build ../*.changes ../*.buildinfo 2>/dev/null

# Build the source package
debuild -S -sa -d
```

**You'll be asked for your GPG passphrase** - enter it.

**Expected output at the end:**
```
dpkg-buildpackage: info: binary and diff upload (original source not included)
Successfully signed dsc, buildinfo, changes files
```

**Check what was created:**
```bash
ls -lh ../ubuntu-maintenance_2.1.0-1*
```

You should see:
- `ubuntu-maintenance_2.1.0-1.dsc`
- `ubuntu-maintenance_2.1.0-1.tar.xz`
- `ubuntu-maintenance_2.1.0-1_source.build`
- `ubuntu-maintenance_2.1.0-1_source.changes`
- `ubuntu-maintenance_2.1.0-1_source.buildinfo`

---

### Step 11: Upload to PPA (3 minutes)

```bash
# Upload
dput ppa ../ubuntu-maintenance_2.1.0-1_source.changes
```

**Expected output:**
```
Uploading to ppa (via ftp to ppa.launchpad.net):
  Uploading ubuntu-maintenance_2.1.0-1.dsc: done.
  Uploading ubuntu-maintenance_2.1.0-1.tar.xz: done.
  Uploading ubuntu-maintenance_2.1.0-1_source.buildinfo: done.
  Uploading ubuntu-maintenance_2.1.0-1_source.changes: done.
Successfully uploaded packages.
```

---

### Step 12: Monitor the Build (10-30 minutes)

**Check your email** - Launchpad will send you:
1. **Acceptance email** (within 2 minutes): "Package accepted"
2. **Build status emails** (5-30 minutes): "Build succeeded" or "Build failed"

**Check build status online:**
```
https://launchpad.net/~YOUR_USERNAME/+archive/ubuntu/ubuntu-maintenance/+packages
```

**Build process:**
- Launchpad will build for: amd64, i386, arm64, armhf
- Each architecture takes 5-15 minutes
- Status shows: 🔵 Building → ✅ Published

**If build fails:**
- Click on the architecture that failed
- View the build log
- Look for errors
- Fix and re-upload with incremented version (2.1.0-2)

---

## Part 4: Install on Your Machine (5 minutes)

### Step 13: Wait for Publication

After all builds succeed, wait **5-10 minutes** for publication to complete.

**You'll know it's ready when:**
- All architecture builds show ✅ Published
- No "waiting to build" messages

---

### Step 14: Add the PPA to Your System

```bash
# Add your PPA
sudo add-apt-repository ppa:YOUR_USERNAME/ubuntu-maintenance

# Update package lists
sudo apt update
```

**Example:**
```bash
sudo add-apt-repository ppa:vinnym/ubuntu-maintenance
sudo apt update
```

---

### Step 15: Install the Package

```bash
# Install
sudo apt install ubuntu-maintenance

# Verify installation
which system_update
system_update --help
```

**Expected output:**
```
/usr/bin/system_update

=== UBUNTU SERVER MAINTENANCE TOOL ===

Usage: sudo system_update [options]
...
```

---

### Step 16: Test It! 🎉

```bash
# Try the new schedule management feature
sudo system_update
```

**Select option 6** to test schedule management!

```bash
# View man page
man system_update

# Check version
apt show ubuntu-maintenance
```

---

## Troubleshooting

### Problem: "gpg: signing failed: Inappropriate ioctl for device"

**Solution:**
```bash
export GPG_TTY=$(tty)
# Try again
debuild -S -sa -d
```

---

### Problem: "Package rejected - already exists"

**Solution:**
```bash
# Increment version number
dch -i
# Add a note about changes, save
# Rebuild
debuild -S -sa -d
dput ppa ../ubuntu-maintenance_2.1.0-2_source.changes
```

---

### Problem: "Failed to fetch" when installing

**Solution:**
- Wait 10 more minutes - publication might still be processing
- Check PPA page shows "Published" for all architectures
- Try: `sudo apt update` again

---

### Problem: Build failed on Launchpad

**Solution:**
1. Click the failed build link
2. Click "buildlog" to see errors
3. Common issues:
   - Missing dependencies: Add to `debian/control` Build-Depends
   - Compilation errors: Test locally first with `debuild -b`
4. Fix code, increment version, re-upload

---

## Quick Reference Card

```bash
# Build and upload (after initial setup)
cd /home/user/ubuntuMaintenance
export GPG_TTY=$(tty)
make clean
debuild -S -sa -d
dput ppa ../ubuntu-maintenance_2.1.0-1_source.changes

# Install on any Ubuntu machine
sudo add-apt-repository ppa:YOUR_USERNAME/ubuntu-maintenance
sudo apt update
sudo apt install ubuntu-maintenance

# Use the tool
sudo system_update
```

---

## For Other Ubuntu Versions

To build for Ubuntu 20.04 (Focal) and 24.04 (Noble):

```bash
# Use the helper script
./build-ppa.sh --all
```

This builds for all supported versions automatically.

---

## Getting Help

- **PPA page:** https://launchpad.net/~YOUR_USERNAME/+archive/ubuntu/ubuntu-maintenance
- **Build logs:** Click architecture → buildlog
- **Email:** Check for Launchpad notifications
- **Launchpad help:** https://help.launchpad.net/Packaging/PPA

---

## Success Checklist

- [ ] Installed build tools
- [ ] Created Launchpad account
- [ ] Generated GPG key
- [ ] Uploaded GPG key to Ubuntu keyserver
- [ ] Confirmed GPG key on Launchpad
- [ ] Created PPA on Launchpad
- [ ] Updated maintainer info in 3 files
- [ ] Configured ~/.dput.cf
- [ ] Built source package successfully
- [ ] Uploaded to PPA
- [ ] Received acceptance email
- [ ] All builds completed successfully
- [ ] Added PPA to your system
- [ ] Installed ubuntu-maintenance
- [ ] Tested the application

**Congratulations! You're now a PPA maintainer! 🎉**

---

## Next Steps

1. **Share your PPA:**
   ```
   sudo add-apt-repository ppa:YOUR_USERNAME/ubuntu-maintenance
   sudo apt install ubuntu-maintenance
   ```

2. **Update the package later:**
   - Make changes to code
   - Run: `dch -i` to add changelog entry
   - Build and upload again

3. **Automatic updates:**
   - Users get updates automatically with `apt upgrade`
   - You just need to upload new versions

---

**Your PPA URL:** `ppa:YOUR_USERNAME/ubuntu-maintenance`

**Share with others:** Anyone can now install your package with:
```bash
sudo add-apt-repository ppa:YOUR_USERNAME/ubuntu-maintenance
sudo apt install ubuntu-maintenance
```
