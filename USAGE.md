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
| `n` | Change Namespace | Open namespace selection prompt |
| `r` | Refresh | Reload current view data |
| `↑` or `k` | Move Up | Move selection cursor up |
| `↓` or `j` | Move Down | Move selection cursor down |

## View-Specific Commands

### Pods View (Press `1`)

| Key | Action | Description |
|-----|--------|-------------|
| `l` | View Logs | Display logs for selected pod (last 100 lines) |
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

1. Press `n` to open the namespace prompt
2. Start typing the namespace name
3. Use `↑`/`↓` arrows to autocomplete from available namespaces
4. Press `Enter` to confirm, or `Esc` to cancel

**Example:**
```
┌─ Enter Namespace (Esc to cancel) ───────────┐
│ kube-system                                  │
└──────────────────────────────────────────────┘

Type to filter, ↑/↓ to select, Enter to confirm
```

**Quick Tips:**
- The header shows your current context and namespace: `Context: minikube | Namespace: default`
- Namespace switching applies to the current context only
- When switching contexts, you'll automatically be placed in that context's default namespace

## Input Modes

### Normal Mode
- Default mode for navigation and viewing
- All keyboard shortcuts are active

### Namespace Selection Mode
- Activated by pressing `n`
- Type to filter namespaces
- `Enter` to confirm, `Esc` to cancel
- `↑`/`↓` to navigate suggestions

### Scale Mode (Deployments only)
- Activated by pressing `s` in Deployments view
- Enter number of replicas
- `Enter` to confirm, `Esc` to cancel

## Status Messages

The bottom of the screen shows:
- **Green messages**: Successful operations (e.g., "Switched to context: production")
- **Red messages**: Errors or failures
- **Help text**: Available commands for current view

## Requirements

- `kubectl` must be installed and configured
- Valid kubeconfig file at `~/.kube/config` or path specified in `$KUBECONFIG`
- Network access to Kubernetes clusters

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
