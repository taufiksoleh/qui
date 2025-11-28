use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::app::{App, InputMode, View};

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_tabs(f, app, chunks[1]);
    render_main_content(f, app, chunks[2]);
    render_footer(f, app, chunks[3]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let mut title = vec![Span::styled(
        "Kubernetes TUI",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )];

    if !app.current_context.is_empty() {
        title.push(Span::raw(" │ "));
        title.push(Span::styled(
            format!("Context: {}", app.current_context),
            Style::default().fg(Color::Green),
        ));
    }

    title.push(Span::raw(" │ "));
    title.push(Span::styled(
        format!("Namespace: {}", app.current_namespace),
        Style::default().fg(Color::Yellow),
    ));

    let header = Paragraph::new(Line::from(title)).block(Block::default().borders(Borders::ALL));

    f.render_widget(header, area);
}

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let tabs = vec![
        ("1", "Pods", View::Pods),
        ("2", "Deployments", View::Deployments),
        ("3", "Services", View::Services),
        ("4", "Clusters", View::Clusters),
        ("5", "Namespaces", View::Namespaces),
        ("?", "Help", View::Help),
    ];

    let mut tab_spans = Vec::new();

    for (i, (key, label, view)) in tabs.iter().enumerate() {
        if i > 0 {
            tab_spans.push(Span::raw(" "));
        }

        let is_active = *view == app.current_view;

        let style = if is_active {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::DIM)
        };

        let tab_text = format!(" {} {} ", key, label);
        tab_spans.push(Span::styled(tab_text, style));
    }

    tab_spans.push(Span::raw("  "));
    tab_spans.push(Span::styled(
        "← → Navigate",
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    ));

    let tabs_paragraph = Paragraph::new(Line::from(tab_spans))
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(tabs_paragraph, area);
}

fn render_main_content(f: &mut Frame, app: &App, area: Rect) {
    match app.current_view {
        View::Pods => render_pods_view(f, app, area),
        View::Deployments => render_deployments_view(f, app, area),
        View::Services => render_services_view(f, app, area),
        View::Logs => render_logs_view(f, app, area),
        View::Clusters => render_clusters_view(f, app, area),
        View::Namespaces => render_namespaces_view(f, app, area),
        View::Help => render_help_view(f, app, area),
        View::Terminal => render_terminal_view(f, app, area),
    }
}

fn render_pods_view(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["NAME", "READY", "STATUS", "RESTARTS", "AGE"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));

    let header = Row::new(header_cells)
        .style(Style::default())
        .height(1)
        .bottom_margin(1);

    let rows = app.pods.iter().enumerate().map(|(i, pod)| {
        let cells = vec![
            Cell::from(pod.name.clone()),
            Cell::from(pod.ready.clone()),
            Cell::from(pod.status.clone()),
            Cell::from(pod.restarts.to_string()),
            Cell::from(pod.age.clone()),
        ];

        let style = if i == app.pod_index {
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        Row::new(cells).style(style).height(1)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Pods")
            .style(Style::default()),
    );

    f.render_widget(table, area);
}

fn render_deployments_view(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["NAME", "READY", "UP-TO-DATE", "AVAILABLE", "AGE"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));

    let header = Row::new(header_cells)
        .style(Style::default())
        .height(1)
        .bottom_margin(1);

    let rows = app.deployments.iter().enumerate().map(|(i, dep)| {
        let cells = vec![
            Cell::from(dep.name.clone()),
            Cell::from(dep.ready.clone()),
            Cell::from(dep.up_to_date.to_string()),
            Cell::from(dep.available.to_string()),
            Cell::from(dep.age.clone()),
        ];

        let style = if i == app.deployment_index {
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        Row::new(cells).style(style).height(1)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Deployments")
            .style(Style::default()),
    );

    f.render_widget(table, area);
}

fn render_services_view(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["NAME", "TYPE", "CLUSTER-IP", "PORTS", "AGE"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));

    let header = Row::new(header_cells)
        .style(Style::default())
        .height(1)
        .bottom_margin(1);

    let rows = app.services.iter().enumerate().map(|(i, svc)| {
        let cells = vec![
            Cell::from(svc.name.clone()),
            Cell::from(svc.service_type.clone()),
            Cell::from(svc.cluster_ip.clone()),
            Cell::from(svc.ports.clone()),
            Cell::from(svc.age.clone()),
        ];

        let style = if i == app.service_index {
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        Row::new(cells).style(style).height(1)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Services")
            .style(Style::default()),
    );

    f.render_widget(table, area);
}

fn render_logs_view(f: &mut Frame, app: &App, area: Rect) {
    let total_lines = app.logs.lines().count();
    let follow_indicator = if app.logs_follow { " [FOLLOW]" } else { "" };
    let title = format!(
        "Pod Logs (Last 100 lines) - Line {}/{}{} - Press 'f' to toggle follow",
        app.logs_scroll + 1,
        total_lines.max(1),
        follow_indicator
    );

    let logs = Paragraph::new(app.logs.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default()),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.logs_scroll as u16, 0));

    f.render_widget(logs, area);
}

fn render_clusters_view(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["CONTEXT", "CLUSTER", "SERVER", "NAMESPACE"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));

    let header = Row::new(header_cells)
        .style(Style::default())
        .height(1)
        .bottom_margin(1);

    let rows = app.contexts.iter().enumerate().map(|(i, ctx)| {
        let mut cells = vec![
            Cell::from(ctx.name.clone()),
            Cell::from(ctx.cluster.clone()),
            Cell::from(ctx.server.clone()),
            Cell::from(ctx.namespace.clone()),
        ];

        // Add a visual indicator for the current context
        if ctx.is_current {
            cells[0] = Cell::from(format!("▶ {}", ctx.name));
        }

        let style = if i == app.context_index {
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else if ctx.is_current {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        Row::new(cells).style(style).height(1)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(35),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Clusters / Contexts")
            .style(Style::default()),
    );

    f.render_widget(table, area);
}

fn render_namespaces_view(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["NAMESPACE"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));

    let header = Row::new(header_cells)
        .style(Style::default())
        .height(1)
        .bottom_margin(1);

    let rows = app.namespaces.iter().enumerate().map(|(i, ns)| {
        let mut name = ns.clone();

        // Add indicator for current namespace
        if ns == &app.current_namespace {
            name = format!("▶ {}", ns);
        }

        let cells = vec![Cell::from(name)];

        let style = if i == app.namespace_index {
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else if ns == &app.current_namespace {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        Row::new(cells).style(style).height(1)
    });

    let table = Table::new(rows, [Constraint::Percentage(100)])
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Namespaces")
                .style(Style::default()),
        );

    f.render_widget(table, area);
}

fn render_help_view(f: &mut Frame, _app: &App, area: Rect) {
    let help_text = vec![
        Line::from(vec![Span::styled(
            "Kube-TUI Quick Reference",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ←/→ - Switch Tab       │  Navigate between tabs with arrow keys"),
        Line::from("  1 - Pods View          │  List all pods in current namespace"),
        Line::from("  2 - Deployments View   │  List all deployments"),
        Line::from("  3 - Services View      │  List all services"),
        Line::from("  4 - Clusters View      │  List all contexts/clusters"),
        Line::from("  5/n - Namespaces View  │  List all namespaces"),
        Line::from("  ?/h - Help View        │  This help screen"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Pod Operations:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  l - View Logs          │  Show last 100 lines of pod logs"),
        Line::from("  e - Exec into Pod      │  Open interactive shell in pod"),
        Line::from("  d - Delete Pod         │  Delete selected pod"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Deployment Operations:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  s - Scale              │  Change replica count"),
        Line::from("  d - Delete             │  Delete selected deployment"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Context & Namespace:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Enter - Switch         │  Switch to selected cluster/namespace"),
        Line::from("  Current items marked with ▶ and highlighted"),
        Line::from("  Note: If connection fails on startup, press 4 to switch context"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Logs View:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ↑/k - Scroll Up        │  Scroll logs up one line"),
        Line::from("  ↓/j - Scroll Down      │  Scroll logs down one line"),
        Line::from("  f - Follow Mode        │  Toggle real-time log following"),
        Line::from("  Esc - Back             │  Return to pods view"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "General:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  r - Refresh            │  Reload current view data"),
        Line::from("  ↑/k - Move Up          │  Navigate selection up (or scroll in logs)"),
        Line::from("  ↓/j - Move Down        │  Navigate selection down (or scroll in logs)"),
        Line::from("  Esc - Back/Close       │  Return to previous view"),
        Line::from("  q - Quit               │  Exit application"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Tips:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  • Use ←/→ arrows or number keys (1-5) to switch between tabs"),
        Line::from("  • Header shows current context and namespace"),
        Line::from("  • Active tab is highlighted in the tab bar"),
        Line::from("  • Status messages appear in green (success) or red (error)"),
        Line::from("  • If cluster is unreachable, switch context (4) and press Enter"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press Esc to close this help",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::ITALIC),
        )]),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .style(Style::default()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_terminal_view(f: &mut Frame, app: &App, area: Rect) {
    let title = if let Some(pod_name) = &app.terminal_pod_name {
        format!(
            "Terminal - Pod: {} | Ruby/Rails: 'irb' or 'bin/rails c' | PgUp/PgDn: Scroll | Esc/Ctrl+D: Exit",
            pod_name
        )
    } else {
        "Terminal (Press Esc or Ctrl+D to exit)".to_string()
    };

    let content = if let Some(lines) = app.get_terminal_screen() {
        if lines.is_empty() {
            "Connecting to pod shell...\n\nTip: Common commands for Ruby/Rails:\n  - irb                  (Interactive Ruby)\n  - bin/rails console    (Rails console)\n  - bundle exec rails c  (Rails console via bundler)\n  - bin/console          (Custom console script)\n\nWaiting for response...".to_string()
        } else {
            // Show the last N lines that fit in the viewport
            let visible_height = area.height.saturating_sub(2) as usize; // -2 for borders
            let total_lines = lines.len();

            // Calculate scroll position
            let scroll = app.terminal_scroll.min(total_lines.saturating_sub(visible_height));

            // Get the visible slice
            let start = if scroll == 0 && total_lines > visible_height {
                // Auto-scroll to bottom if not manually scrolled
                total_lines.saturating_sub(visible_height)
            } else {
                scroll
            };

            let end = (start + visible_height).min(total_lines);

            lines[start..end].join("\n")
        }
    } else {
        "Connecting to pod...".to_string()
    };

    let terminal = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default().fg(Color::Green)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(terminal, area);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(2)])
        .split(area);

    // Status/Error message
    if let Some(error) = &app.error_message {
        let error_msg = Paragraph::new(error.clone()).style(Style::default().fg(Color::Red));
        f.render_widget(error_msg, chunks[0]);
    } else if !app.status_message.is_empty() {
        let status_msg =
            Paragraph::new(app.status_message.clone()).style(Style::default().fg(Color::Green));
        f.render_widget(status_msg, chunks[0]);
    }

    // Input mode or help
    match app.input_mode {
        InputMode::Normal => {
            let help_items: Vec<Span> = app
                .get_help_text()
                .iter()
                .flat_map(|(key, desc)| {
                    vec![
                        Span::styled(
                            *key,
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(format!(":{} ", desc)),
                    ]
                })
                .collect();

            let help = Paragraph::new(Line::from(help_items))
                .block(Block::default().borders(Borders::ALL));

            f.render_widget(help, chunks[1]);
        }
        InputMode::Scale => {
            let input = Paragraph::new(app.input_buffer.clone())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Enter number of replicas (Esc to cancel)"),
                )
                .style(Style::default().fg(Color::Yellow));

            f.render_widget(input, chunks[1]);
        }
    }
}
