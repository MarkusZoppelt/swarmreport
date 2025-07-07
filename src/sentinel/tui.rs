use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
/// Terminal User Interface for SwarmReport Sentinel
///
/// Displays real-time system information from connected clients in a grid layout.
/// Clients are color-coded based on how recently they've reported in.
use std::time::Duration;

use super::types::{App, SharedState, parse_cpu_usage};

/// Determines border color based on how recently a client reported
fn get_status_color(seconds_since_update: u64) -> Color {
    match seconds_since_update {
        0..=4 => Color::Green,   // Recent
        5..=30 => Color::Yellow, // Normal
        _ => Color::Red,         // Stale
    }
}

/// Formats services list for display, showing up to 3 services with status icons
fn format_services_display(services: &[crate::swarmreport::Service]) -> String {
    if services.is_empty() {
        return "No services".to_string();
    }

    services
        .iter()
        .take(3)
        .map(|s| {
            let status_icon = match s.status.as_str() {
                "running" => "✓",
                "stopped" => "✗",
                _ => "?",
            };
            format!("{} {}", status_icon, s.name)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Renders the main UI showing all connected clients in a grid
pub fn ui(f: &mut ratatui::Frame, app: &App) {
    let ordered_reports = app.get_ordered_reports();

    if ordered_reports.is_empty() {
        let no_clients = Paragraph::new("No clients connected")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("SwarmReport - No Active Clients"),
            )
            .style(Style::default().fg(Color::Gray));
        f.render_widget(no_clients, f.area());
        return;
    }

    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let client_areas = calculate_grid_layout(f.area(), ordered_reports.len());

    for (i, entry) in ordered_reports.iter().enumerate() {
        if i >= client_areas.len() {
            break;
        }

        let seconds_since_update = current_time.saturating_sub(entry.last_updated);
        let border_color = get_status_color(seconds_since_update);

        let last_updated = std::time::UNIX_EPOCH + Duration::from_secs(entry.last_updated);
        let datetime = chrono::DateTime::<chrono::Utc>::from(last_updated);
        let time_str = datetime.format("%H:%M:%S").to_string();

        let services_text = format_services_display(&entry.report.services);
        let cpu_usage = parse_cpu_usage(&entry.report.cpu_usage);

        let content = format!(
            "{}\n{}\nLast: {}\n\nCPU: {:.1}%\nMEM: {}\nDISK: {}\n\nServices:\n{}",
            entry.report.hostname,
            entry.report.ip_address,
            time_str,
            cpu_usage,
            entry.report.memory_usage,
            entry.report.disk_usage,
            services_text
        );

        let client_block = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(format!("Client {}", i + 1)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(client_block, client_areas[i]);
    }
}

/// Calculates optimal grid layout for displaying clients
/// Returns a vector of rectangles representing each client's display area
fn calculate_grid_layout(area: Rect, num_clients: usize) -> Vec<Rect> {
    if num_clients == 0 {
        return vec![];
    }

    // Calculate grid dimensions (roughly square)
    let cols = ((num_clients as f64).sqrt().ceil() as u16).max(1);
    let rows = ((num_clients as f64 / cols as f64).ceil() as u16).max(1);

    // Create row constraints
    let row_constraints: Vec<Constraint> = (0..rows)
        .map(|_| Constraint::Percentage(100 / rows))
        .collect();

    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(area);

    let mut areas = Vec::new();

    for row in 0..rows {
        // Calculate how many clients in this row (last row might have fewer)
        let clients_in_row = if row == rows - 1 {
            num_clients - (row as usize * cols as usize)
        } else {
            cols as usize
        };

        // Create column constraints for this row
        let col_constraints: Vec<Constraint> = (0..clients_in_row)
            .map(|_| Constraint::Percentage(100 / clients_in_row as u16))
            .collect();

        let col_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .split(row_chunks[row as usize]);

        areas.extend(col_chunks.iter().cloned());
    }

    areas
}

pub async fn run_tui_display_only(
    state: SharedState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        {
            let app = state.lock().unwrap();
            terminal.draw(|f| ui(f, &app))?;
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
