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
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_main_content(f, app, chunks[1]);
    render_footer(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let mut title = vec![
        Span::styled(
            "Kubernetes TUI",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ];

    if !app.current_context.is_empty() {
        title.push(Span::raw(" | "));
        title.push(Span::styled(
            format!("Context: {}", app.current_context),
            Style::default().fg(Color::Green),
        ));
    }

    title.push(Span::raw(" | "));
    title.push(Span::styled(
        format!("Namespace: {}", app.current_namespace),
        Style::default().fg(Color::Yellow),
    ));

    let header = Paragraph::new(Line::from(title)).block(Block::default().borders(Borders::ALL));

    f.render_widget(header, area);
}

fn render_main_content(f: &mut Frame, app: &App, area: Rect) {
    match app.current_view {
        View::Pods => render_pods_view(f, app, area),
        View::Deployments => render_deployments_view(f, app, area),
        View::Services => render_services_view(f, app, area),
        View::Logs => render_logs_view(f, app, area),
        View::Clusters => render_clusters_view(f, app, area),
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
    let logs = Paragraph::new(app.logs.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Pod Logs (Last 100 lines)")
                .style(Style::default()),
        )
        .wrap(Wrap { trim: false });

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
        InputMode::Namespace => {
            let input = Paragraph::new(app.input_buffer.clone())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Enter Namespace (Esc to cancel)"),
                )
                .style(Style::default().fg(Color::Yellow));

            f.render_widget(input, chunks[1]);
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
