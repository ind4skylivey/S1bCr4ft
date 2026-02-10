# S1bCr4ft AUR Packaging

This directory contains the PKGBUILD files for publishing S1bCr4ft to the Arch User Repository (AUR).

## Packages

### s1bcr4ft
Stable release package tracking official releases.

### s1bcr4ft-git
Development package tracking the latest git commits.

## Building Locally

```bash
# Clone this directory
cd packaging/aur

# Build stable version
makepkg -si

# Or build git version
cp PKGBUILD-git PKGBUILD
makepkg -si
```

## Publishing to AUR

### First Time Setup

```bash
# Clone AUR repository
git clone ssh://aur@aur.archlinux.org/s1bcr4ft.git aur-s1bcr4ft
cd aur-s1bcr4ft

# Copy PKGBUILD
cp ../PKGBUILD .

# Generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Commit and push
git add PKGBUILD .SRCINFO
git commit -m "Initial commit"
git push
```

### Updating

```bash
# Update version in PKGBUILD
# Regenerate .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Commit and push
git add PKGBUILD .SRCINFO
git commit -m "Update to version X.Y.Z"
git push
```

## Testing

Before publishing, always test the package:

```bash
# Test build
makepkg -sf

# Test installation
makepkg -si

# Test functionality
s1bcr4ft --version
s1bcr4ft --help
```

## Dependencies

- **Required**: pacman
- **Optional**: paru or yay (for AUR support)
- **Build**: rust, cargo

## Links

- AUR Package: https://aur.archlinux.org/packages/s1bcr4ft
- AUR Git Package: https://aur.archlinux.org/packages/s1bcr4ft-git
- GitHub: https://github.com/S1b-Team/S1bCr4ft
