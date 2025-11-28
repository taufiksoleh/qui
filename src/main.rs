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
use std::io;
use std::time::{Duration, Instant};

use app::App;
use events::EventHandler;
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

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> Result<()> {
    let mut event_handler = EventHandler::new();
    let mut last_log_refresh = Instant::now();
    let log_refresh_interval = Duration::from_secs(2); // Refresh logs every 2 seconds
    let mut last_terminal_refresh = Instant::now();
    let terminal_refresh_interval = Duration::from_millis(50); // Refresh terminal every 50ms for smooth updates

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        // Refresh logs if in follow mode and enough time has passed
        if last_log_refresh.elapsed() >= log_refresh_interval {
            app.refresh_logs().await?;
            last_log_refresh = Instant::now();
        }

        // Refresh terminal more frequently for smooth interactive commands
        if matches!(app.current_view, app::View::Terminal) {
            if last_terminal_refresh.elapsed() >= terminal_refresh_interval {
                app.refresh_terminal();
                last_terminal_refresh = Instant::now();
            }
        }

        if let Some(event) = event_handler.next()? {
            if !app.handle_event(event).await? {
                return Ok(());
            }
        }
    }
}
