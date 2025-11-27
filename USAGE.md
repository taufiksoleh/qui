# Kube-TUI Usage Guide

A Terminal User Interface for managing Kubernetes resources with easy navigation and cluster/namespace switching.

## Starting the Application

```bash
kube-tui
```

The application will automatically connect to your current Kubernetes context and display pods in the default namespace.

## Navigation Commands

### Global Commands (Available in all views)

| Key | Action | Description |
|-----|--------|-------------|
| `q` | Quit | Exit the application |
| `1` | Pods View | Switch to Pods view |
| `2` | Deployments View | Switch to Deployments view |
| `3` | Services View | Switch to Services view |
| `4` | Clusters View | Switch to Clusters/Contexts view |
| `5` or `n` | Namespaces View | Switch to Namespaces view |
| `?` or `h` | Help | Show help screen with all commands |
| `r` | Refresh | Reload current view data |
| `↑` or `k` | Move Up | Move selection cursor up |
| `↓` or `j` | Move Down | Move selection cursor down |
| `Esc` | Back/Close | Return to previous view or close dialogs |

## View-Specific Commands

### Pods View (Press `1`)

| Key | Action | Description |
|-----|--------|-------------|
| `l` | View Logs | Display logs for selected pod (last 100 lines) |
| `e` | Exec into Pod | Open interactive shell in the selected pod |
| `d` | Delete | Delete the selected pod |

### Deployments View (Press `2`)

| Key | Action | Description |
|-----|--------|-------------|
| `s` | Scale | Scale the selected deployment (enter replica count) |
| `d` | Delete | Delete the selected deployment |

### Services View (Press `3`)

Shows list of services with TYPE, CLUSTER-IP, PORTS, and AGE information.

### Clusters View (Press `4`)

| Key | Action | Description |
|-----|--------|-------------|
| `Enter` | Switch Context | Switch to the selected Kubernetes context/cluster |

The Clusters view displays:
- **CONTEXT**: Context name (▶ indicator shows current context)
- **CLUSTER**: Associated cluster name
- **SERVER**: Kubernetes API server URL
- **NAMESPACE**: Default namespace for the context

**Note**: The currently connected context is highlighted in green with a ▶ arrow indicator.

### Namespaces View (Press `5` or `n`)

| Key | Action | Description |
|-----|--------|-------------|
| `Enter` | Switch Namespace | Switch to the selected namespace |

The Namespaces view displays all available namespaces in the current cluster. The current namespace is marked with ▶ and highlighted in yellow.

### Help View (Press `?` or `h`)

Shows a comprehensive quick reference guide with all available commands organized by category. Press `Esc` to close.

### Logs View

| Key | Action | Description |
|-----|--------|-------------|
| `Esc` | Back | Return to previous view |

## How to Switch Between Contexts (Clusters)

1. Press `4` to open the Clusters view
2. Use `↑`/`↓` or `k`/`j` to navigate to the desired context
3. Press `Enter` to switch to that context
4. The application will automatically reconnect and refresh all data

**Example:**
```
┌─ Clusters / Contexts ──────────────────────────────────────┐
│ CONTEXT              CLUSTER           SERVER              │
│ ▶ minikube           minikube          https://...         │ ← Current
│   production         prod-cluster      https://...         │
│   staging            stage-cluster     https://...         │
└────────────────────────────────────────────────────────────┘

Navigate with ↑/↓, press Enter to switch
```

## How to Switch Between Namespaces

1. Press `5` or `n` to open the Namespaces view
2. Use `↑`/`↓` or `k`/`j` to navigate to the desired namespace
3. Press `Enter` to switch to that namespace
4. You'll automatically return to the Pods view with the new namespace active

**Example:**
```
┌─ Namespaces ───────────────────────────────────┐
│ NAMESPACE                                      │
│ ▶ default                                      │ ← Current
│   kube-system                                  │
│   kube-public                                  │
│   my-app                                       │
└────────────────────────────────────────────────┘

Navigate with ↑/↓, press Enter to switch
```

**Quick Tips:**
- The header shows your current context and namespace: `Context: minikube | Namespace: default`
- Current namespace is marked with ▶ and highlighted in yellow
- Namespace switching applies to the current context only
- When switching contexts, you'll automatically be placed in that context's default namespace
- Press `?` or `h` anytime to see the help screen

## Input Modes

### Normal Mode
- Default mode for navigation and viewing
- All keyboard shortcuts are active
- Navigate between views, select items, perform actions

### Scale Mode (Deployments only)
- Activated by pressing `s` in Deployments view
- Enter number of replicas for the selected deployment
- `Enter` to confirm, `Esc` to cancel
- Only numeric input is accepted

## Status Messages

The bottom of the screen shows:
- **Green messages**: Successful operations (e.g., "Switched to context: production")
- **Red messages**: Errors or failures
- **Help text**: Available commands for current view

## Requirements

- `kubectl` must be installed and configured
- Valid kubeconfig file at `~/.kube/config` or path specified in `$KUBECONFIG`
- Network access to Kubernetes clusters

## Advanced Features

### Exec into Pod (Terminal Access)

Press `e` in the Pods view to open an interactive shell inside the selected pod.

**How it works:**
1. Navigate to the Pods view (press `1`)
2. Select a pod using `↑`/`↓` arrows
3. Press `e` to exec into the pod
4. An interactive shell will open (tries `/bin/sh`, falls back to `/bin/bash`)
5. Type `exit` or press `Ctrl+D` to return to the TUI

**Example use cases:**
- Debug application issues
- Inspect file systems
- Run diagnostic commands
- Check environment variables
- Test network connectivity

**Note:** The pod must have `/bin/sh` or `/bin/bash` available.

### Built-in Help System

Press `?` or `h` at any time to open the comprehensive help screen that shows:
- All navigation commands
- View-specific operations
- Keyboard shortcuts
- Tips and tricks

The help screen is context-aware and always available, making it easy to discover features.

## Troubleshooting

### Cannot switch contexts
- Ensure `kubectl config use-context <name>` works from terminal
- Verify kubectl is in your PATH
- Check kubeconfig file permissions

### Namespaces not showing
- Verify you have permissions to list namespaces
- Check your RBAC permissions in the current context

### Cannot see pods/deployments
- Ensure you're in the correct namespace
- Verify namespace exists: `kubectl get namespaces`
- Check RBAC permissions for resource access
