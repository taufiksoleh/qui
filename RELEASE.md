# Release Process

This document describes the release process for QUI and the automated build pipeline.

## Automated Build Pipeline

The project uses GitHub Actions to automatically build release binaries for multiple platforms.

### Supported Platforms

The release pipeline builds binaries for:
- **Linux x86_64** (Intel/AMD 64-bit)
- **Linux aarch64** (ARM 64-bit, e.g., Raspberry Pi 4, AWS Graviton)
- **macOS x86_64** (Intel Macs)
- **macOS aarch64** (Apple Silicon M1/M2/M3)

### Continuous Integration (CI)

The CI pipeline runs on every push and pull request to `main`/`master` branches:
- **Code Check**: Validates code compiles correctly
- **Tests**: Runs test suite on Linux and macOS
- **Formatting**: Checks code formatting with rustfmt
- **Linting**: Runs clippy for code quality
- **Build**: Creates release builds on Linux and macOS

### Release Workflow

The release workflow is triggered in two ways:

#### 1. Automated Release (Recommended)

When you push a version tag:

```bash
# Update version in Cargo.toml
# Commit your changes
git add Cargo.toml
git commit -m "Bump version to 0.2.0"

# Create and push a version tag
git tag v0.2.0
git push origin v0.2.0
```

This will:
1. Build binaries for all supported platforms
2. Create a GitHub Release with auto-generated release notes
3. Upload all binaries as release assets

#### 2. Manual Trigger

You can also manually trigger the release workflow from the GitHub Actions tab without creating a tag. This will build the binaries but won't create a GitHub release.

## Release Artifacts

Each release includes the following artifacts:

- `qui-linux-x86_64.tar.gz` - Linux x86_64 binary
- `qui-linux-x86_64.tar.gz.sha256` - Linux x86_64 SHA256 checksum
- `qui-macos-x86_64.tar.gz` - macOS Intel binary
- `qui-macos-x86_64.tar.gz.sha256` - macOS Intel SHA256 checksum
- `qui-macos-aarch64.tar.gz` - macOS Apple Silicon binary
- `qui-macos-aarch64.tar.gz.sha256` - macOS Apple Silicon SHA256 checksum

## Installing Released Binaries

### Quick Install (Recommended)

Use the installation script to automatically detect your system and install:

```bash
curl -fsSL https://raw.githubusercontent.com/taufiksoleh/qui/main/install.sh | bash
```

The script automatically:
- Detects your OS (Linux/macOS) and architecture (x86_64/aarch64)
- Downloads the correct binary from the latest release
- Installs to `/usr/local/bin` (or custom directory with `INSTALL_DIR`)
- Verifies the installation

Custom installation directory:

```bash
INSTALL_DIR=$HOME/.local/bin ./install.sh
```

### Homebrew (macOS and Linux)

Install using Homebrew:

```bash
brew tap taufiksoleh/qui
brew install qui
```

The Homebrew formula is automatically updated with each release. See [HOMEBREW.md](HOMEBREW.md) for more details.

### Manual Installation by Platform

#### Linux (x86_64)

```bash
curl -L https://github.com/taufiksoleh/qui/releases/latest/download/qui-linux-x86_64.tar.gz | tar xz
sudo mv qui /usr/local/bin/
```

#### Linux (ARM64)

```bash
curl -L https://github.com/taufiksoleh/qui/releases/latest/download/qui-linux-aarch64.tar.gz | tar xz
sudo mv qui /usr/local/bin/
```

#### macOS (Intel)

```bash
curl -L https://github.com/taufiksoleh/qui/releases/latest/download/qui-macos-x86_64.tar.gz | tar xz
sudo mv qui /usr/local/bin/
```

#### macOS (Apple Silicon)

```bash
curl -L https://github.com/taufiksoleh/qui/releases/latest/download/qui-macos-aarch64.tar.gz | tar xz
sudo mv qui /usr/local/bin/
```

## Version Numbering

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version for incompatible API changes
- **MINOR** version for new functionality in a backward compatible manner
- **PATCH** version for backward compatible bug fixes

Examples:
- `v1.0.0` - First stable release
- `v1.1.0` - Added new features
- `v1.1.1` - Bug fixes
- `v2.0.0` - Breaking changes

## Release Checklist

Before creating a new release:

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md` (if you maintain one)
- [ ] Update `README.md` if there are new features
- [ ] Run tests locally: `cargo test`
- [ ] Build locally: `cargo build --release`
- [ ] Commit all changes
- [ ] Create and push git tag: `git tag v<version> && git push origin v<version>`
- [ ] Verify GitHub Actions workflow completes successfully
- [ ] Test downloaded binaries on target platforms
- [ ] Update release notes on GitHub if needed

## Troubleshooting Builds

### Build Fails on Cross-Compilation

If the Linux ARM64 build fails, it may be due to missing cross-compilation tools. The workflow installs these automatically, but local builds require:

```bash
sudo apt-get install gcc-aarch64-linux-gnu
```

### macOS Universal Binary

To create a universal macOS binary (Intel + Apple Silicon):

```bash
# Build both architectures
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Combine into universal binary
lipo -create \
  target/x86_64-apple-darwin/release/qui \
  target/aarch64-apple-darwin/release/qui \
  -output qui-universal
```

### Testing Binaries

After download, verify the binary:

```bash
# Check binary type
file qui

# Check it runs
./qui --version  # (if version flag is implemented)
```

## Manual Release Process (Fallback)

If automated releases fail, you can create releases manually:

1. Build locally for your platform:
   ```bash
   cargo build --release
   ```

2. Create a tarball:
   ```bash
   cd target/release
   tar -czf qui-<platform>.tar.gz qui
   ```

3. Create a GitHub Release manually and upload the tarball

## Implemented Features

The release process includes:
- ✅ SHA256 checksums for all binaries
- ✅ Homebrew tap with automatic formula updates
- ✅ Multi-platform builds (Linux x86_64, macOS Intel, macOS Apple Silicon)
- ✅ Automated release creation with GitHub Actions
- ✅ Automatic installation script

## Future Enhancements

Planned improvements to the release process:
- Windows builds (requires WSL2 or containerized builds)
- Docker images published to GitHub Container Registry
- Binary signing for macOS (requires Apple Developer account)
- APT/YUM repositories for Linux distributions
- Automated changelog generation
- Linux ARM64 builds
