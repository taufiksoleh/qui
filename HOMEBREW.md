# Homebrew Tap for QUI

This repository serves as a Homebrew tap for installing QUI (Kubernetes Terminal UI).

## What is a Homebrew Tap?

A Homebrew tap is a third-party repository that provides formulae for Homebrew, the package manager for macOS and Linux. By adding this tap, users can install QUI using the familiar `brew install` command.

## Installation

### For Users

Install QUI using Homebrew:

```bash
# Tap the repository
brew tap taufiksoleh/qui

# Install QUI
brew install qui
```

Or install directly without tapping:

```bash
brew install taufiksoleh/qui/qui
```

### Upgrade

To upgrade to the latest version:

```bash
brew update
brew upgrade qui
```

### Uninstall

```bash
brew uninstall qui
brew untap taufiksoleh/qui  # Optional: remove the tap
```

## How It Works

### Automatic Formula Updates

The Homebrew formula is automatically updated when a new release is created:

1. **Version Tag**: When you push a version tag (e.g., `v0.2.0`), the release workflow is triggered
2. **Build**: Binaries are built for macOS (Intel and Apple Silicon) and Linux
3. **Checksums**: SHA256 checksums are generated for each binary
4. **Release**: GitHub Release is created with all binaries and checksums
5. **Formula Update**: The `Formula/qui.rb` file is automatically updated with:
   - New version number
   - Updated download URLs
   - New SHA256 checksums
6. **Commit**: Changes are committed and pushed back to the repository

### Formula Structure

The formula (`Formula/qui.rb`) defines:
- **Description**: What QUI does
- **Homepage**: Link to the project
- **Version**: Current version number
- **License**: Software license (MIT)
- **URLs**: Download links for different platforms and architectures
- **SHA256**: Checksums for verifying downloads
- **Installation**: How to install the binary
- **Tests**: Basic tests to verify the installation

### Supported Platforms

The Homebrew formula supports:
- **macOS Intel** (x86_64)
- **macOS Apple Silicon** (aarch64/arm64)
- **Linux Intel** (x86_64)

Homebrew automatically selects the correct binary based on the user's platform and architecture.

## For Maintainers

### Creating a New Release

To create a new release that updates the Homebrew formula:

1. Update the version in `Cargo.toml`:
   ```toml
   version = "0.2.0"
   ```

2. Commit your changes:
   ```bash
   git add Cargo.toml
   git commit -m "Bump version to 0.2.0"
   git push
   ```

3. Create and push a version tag:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. The GitHub Actions workflow will:
   - Build binaries for all platforms
   - Generate SHA256 checksums
   - Create a GitHub Release
   - Automatically update `Formula/qui.rb`
   - Commit and push the updated formula

### Manual Formula Updates

If you need to manually update the formula:

1. Edit `Formula/qui.rb`
2. Update the version number:
   ```ruby
   version "0.2.0"
   ```
3. Update the download URLs to point to the new release
4. Update the SHA256 checksums (get them from the release artifacts)
5. Commit and push the changes

### Getting SHA256 Checksums

If you need to manually calculate checksums:

```bash
# Download the binary
curl -L -o qui-macos-x86_64.tar.gz \
  https://github.com/taufiksoleh/qui/releases/download/v0.2.0/qui-macos-x86_64.tar.gz

# Calculate SHA256
shasum -a 256 qui-macos-x86_64.tar.gz
```

### Testing the Formula Locally

Before pushing changes, test the formula locally:

```bash
# Audit the formula
brew audit --strict Formula/qui.rb

# Test installation
brew install --build-from-source Formula/qui.rb

# Run tests
brew test qui

# Uninstall
brew uninstall qui
```

### Formula Linting

Ensure the formula follows Homebrew standards:

```bash
brew audit --new-formula Formula/qui.rb
brew style Formula/qui.rb
```

## Troubleshooting

### Formula Not Found

If users get "formula not found" errors:

```bash
# Update Homebrew
brew update

# Verify tap is added
brew tap

# Re-tap if needed
brew untap taufiksoleh/qui
brew tap taufiksoleh/qui
```

### Checksum Mismatch

If checksums don't match:
1. Verify the release artifacts on GitHub
2. Recalculate checksums using `shasum -a 256`
3. Update `Formula/qui.rb` with correct checksums
4. Commit and push

### Installation Fails

For debugging installation issues:

```bash
# Install with verbose output
brew install --verbose qui

# Check the formula
brew info qui

# View installation logs
brew --cache
```

## Resources

- [Homebrew Documentation](https://docs.brew.sh/)
- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Creating Taps](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
- [Acceptable Formulae](https://docs.brew.sh/Acceptable-Formulae)

## Support

For issues related to:
- **Homebrew installation**: Open an issue in this repository
- **QUI functionality**: Open an issue in the main [qui repository](https://github.com/taufiksoleh/qui)
- **Homebrew itself**: See [Homebrew's troubleshooting guide](https://docs.brew.sh/Troubleshooting)
