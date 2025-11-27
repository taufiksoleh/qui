# Kube-TUI - Kubernetes Terminal UI

[![CI](https://github.com/taufiksoleh/qui/workflows/CI/badge.svg)](https://github.com/taufiksoleh/qui/actions/workflows/ci.yml)
[![Test Installation](https://github.com/taufiksoleh/qui/workflows/Test%20Installation%20Script/badge.svg)](https://github.com/taufiksoleh/qui/actions/workflows/test-install.yml)
[![Release](https://github.com/taufiksoleh/qui/workflows/Release%20Build/badge.svg)](https://github.com/taufiksoleh/qui/actions/workflows/release.yml)

A powerful terminal user interface (TUI) for managing Kubernetes clusters, written in Rust. Kube-TUI provides an intuitive, interactive way to manage your Kubernetes resources without the complexity of kubectl commands.

## Features

- **Multi-Cluster Support**: View and switch between Kubernetes contexts/clusters
- **Cluster Overview**: Display all available contexts with connection indicator
- **Pod Management**: View, monitor, and delete pods with real-time status updates
- **Deployment Management**: List deployments, scale replicas, and delete deployments
- **Service Viewing**: Browse Kubernetes services with detailed information
- **Log Viewer**: View pod logs directly in the terminal (last 100 lines)
- **Namespace Switching**: Quickly switch between namespaces
- **Interactive Navigation**: Vim-style keybindings (j/k) and arrow key support
- **Resource Operations**: Delete pods and deployments, scale deployments
- **Fast & Lightweight**: Built with Rust for maximum performance

## Prerequisites

- Access to a Kubernetes cluster
- kubectl configured with valid credentials (uses your default kubeconfig)
- (For building from source) Rust 1.70 or later

## Installation

### Quick Install (Recommended)

Use the installation script to automatically detect your system and install the latest version:

```bash
curl -fsSL https://raw.githubusercontent.com/taufiksoleh/qui/main/install.sh | bash
```

Or download and run the script manually:

```bash
wget https://raw.githubusercontent.com/taufiksoleh/qui/main/install.sh
chmod +x install.sh
./install.sh
```

The script will:
- Detect your OS and architecture automatically
- Download the appropriate binary
- Install to `/usr/local/bin` (requires sudo)
- Verify the installation

To install to a custom directory:

```bash
INSTALL_DIR=$HOME/.local/bin ./install.sh
```

### Manual Installation

Pre-built binaries are available for Linux and macOS on the [releases page](https://github.com/taufiksoleh/qui/releases).

**Linux (x86_64):**
```bash
curl -L https://github.com/taufiksoleh/qui/releases/latest/download/kube-tui-linux-x86_64.tar.gz | tar xz
sudo mv kube-tui /usr/local/bin/
```

**macOS (Intel):**
```bash
curl -L https://github.com/taufiksoleh/qui/releases/latest/download/kube-tui-macos-x86_64.tar.gz | tar xz
sudo mv kube-tui /usr/local/bin/
```

**macOS (Apple Silicon):**
```bash
curl -L https://github.com/taufiksoleh/qui/releases/latest/download/kube-tui-macos-aarch64.tar.gz | tar xz
sudo mv kube-tui /usr/local/bin/
```

See [RELEASE.md](RELEASE.md) for more installation options and platform-specific instructions.

### Build from source

```bash
git clone https://github.com/yourusername/kube-tui.git
cd kube-tui
cargo build --release
```

The binary will be available at `target/release/kube-tui`

### Run directly

```bash
cargo run --release
```

## Usage

Launch the application:

```bash
./target/release/kube-tui
# or
cargo run --release
```

For detailed usage instructions, see [USAGE.md](USAGE.md).

### Quick Start

#### Switching Between Clusters/Contexts
1. Press `4` to view all available clusters
2. Use `↑`/`↓` to select a cluster
3. Press `Enter` to switch to that cluster
4. Current cluster is marked with ▶ and highlighted in green

#### Switching Between Namespaces
1. Press `n` to open namespace selector
2. Type to filter or use `↑`/`↓` to select
3. Press `Enter` to switch

### Keyboard Shortcuts

#### Global Navigation
- `q` - Quit the application
- `1` - Switch to Pods view
- `2` - Switch to Deployments view
- `3` - Switch to Services view
- `4` - Switch to Clusters/Contexts view
- `n` - Change namespace (opens input prompt)
- `r` - Refresh current view
- `↑` or `k` - Move selection up
- `↓` or `j` - Move selection down

#### Clusters View
- `Enter` - Switch to selected context/cluster

#### Pods View
- `l` - View logs for selected pod
- `d` - Delete selected pod

#### Deployments View
- `s` - Scale deployment (opens replica count prompt)
- `d` - Delete selected deployment

#### Logs View
- `Esc` - Return to previous view

#### Input Prompts
- `Enter` - Confirm input
- `Esc` - Cancel input
- `Backspace` - Delete character
- `↑/↓` - Navigate suggestions (namespace selector)

## Features Overview

### Clusters View
Displays all available Kubernetes contexts from your kubeconfig with:
- Context name (with ▶ indicator for current context)
- Cluster name
- API server URL
- Default namespace
- Visual highlighting of the active context in green
- Press `Enter` to switch contexts seamlessly

### Pods View
Displays all pods in the current namespace with:
- Pod name
- Ready status (ready containers / total containers)
- Current phase (Running, Pending, Failed, etc.)
- Restart count
- Age

### Deployments View
Shows deployments with:
- Deployment name
- Ready replicas
- Up-to-date replicas
- Available replicas
- Age

### Services View
Lists services with:
- Service name
- Type (ClusterIP, NodePort, LoadBalancer)
- Cluster IP address
- Exposed ports
- Age

### Logs View
- Displays the last 100 lines of logs from a selected pod
- Scrollable view for reviewing application output
- Quick access with `l` key from pods view

## Configuration

Kube-TUI uses your Kubernetes configuration:
- Config file: `~/.kube/config` or path from `$KUBECONFIG` environment variable
- Context: Uses the current context (can be switched from within the app using `4`)
- Authentication: Inherits from kubectl configuration

You can switch contexts either:
1. **From within the app**: Press `4`, select a context, press `Enter`
2. **From command line**: `kubectl config use-context <context-name>` (then restart the app)

## Architecture

The application is built with:
- **ratatui**: Terminal UI framework
- **crossterm**: Cross-platform terminal manipulation
- **kube-rs**: Kubernetes client library
- **tokio**: Async runtime

Project structure:
```
src/
├── main.rs          # Application entry point and terminal setup
├── app.rs           # Application state and event handling
├── ui.rs            # UI rendering logic
├── kube_client.rs   # Kubernetes API client wrapper
└── events.rs        # Event handling and input processing
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [ratatui](https://github.com/ratatui-org/ratatui)
- Kubernetes client: [kube-rs](https://github.com/kube-rs/kube)

## Troubleshooting

### Connection Issues
If you can't connect to your cluster:
1. Verify kubectl works: `kubectl get pods`
2. Check your kubeconfig: `kubectl config view`
3. Ensure you have the necessary RBAC permissions

### Build Issues
If you encounter build errors:
1. Update Rust: `rustup update`
2. Clean build artifacts: `cargo clean`
3. Rebuild: `cargo build --release`

## Future Enhancements

Potential features for future releases:
- ConfigMaps and Secrets management
- Real-time resource metrics (CPU/Memory)
- Port forwarding
- Exec into containers
- YAML editing and apply
- Resource describe view
- Custom themes
- CRD (Custom Resource Definition) support