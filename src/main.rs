mod app;
mod events;
mod kube_client;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use app::{App, PendingAction};
use events::EventHandler;
use kube_client::KubeClient;
use ui::ui;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let app = App::new().await?;
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    mut app: App,
) -> Result<()> {
    let mut event_handler = EventHandler::new();
    let mut last_log_refresh = Instant::now();
    let log_refresh_interval = Duration::from_secs(2); // Refresh logs every 2 seconds

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        // Handle pending actions that require TUI suspension
        if let Some(action) = app.take_pending_action() {
            match action {
                PendingAction::ExecIntoPod { namespace, pod_name } => {
                    // Suspend TUI
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;

                    // Execute into pod
                    let result = KubeClient::exec_into_pod(&namespace, &pod_name);

                    // Resume TUI
                    enable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        EnterAlternateScreen,
                        EnableMouseCapture
                    )?;

                    // Clear the terminal to force a full redraw
                    terminal.clear()?;

                    // Set result message
                    match result {
                        Ok(_) => app.set_exec_result(&pod_name, true, None),
                        Err(e) => app.set_exec_result(&pod_name, false, Some(e.to_string())),
                    }
                }
            }
        }

        // Refresh logs if in follow mode and enough time has passed
        if last_log_refresh.elapsed() >= log_refresh_interval {
            app.refresh_logs().await?;
            last_log_refresh = Instant::now();
        }

        if let Some(event) = event_handler.next()? {
            if !app.handle_event(event).await? {
                return Ok(());
            }
        }
    }
}
