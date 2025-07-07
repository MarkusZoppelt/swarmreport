use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
};
/// Terminal User Interface for SwarmReport Sentinel
///
/// Displays real-time system information from connected clients in a grid layout.
/// Clients are color-coded based on how recently they've reported in.
use std::time::Duration;

use super::types::{App, ReportEntry, SharedState, parse_cpu_usage};

/// Determines border color based on how recently a client reported
fn get_status_color(seconds_since_update: u64) -> Color {
    match seconds_since_update {
        0..=4 => Color::Green,   // Recent
        5..=30 => Color::Yellow, // Normal
        _ => Color::Red,         // Stale
    }
}

/// Renders the main UI in a lazygit/lazydocker style with multiple information panels
pub fn ui(f: &mut ratatui::Frame, app: &App) {
    let ordered_reports = app.get_ordered_reports();
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Main layout: split into content area and status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    // Content area: split into left and right sections
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_chunks[0]);

    // Left side: split into top (clients list) and bottom (overview stats)
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(content_chunks[0]);

    // Right side: split into top (selected client details) and bottom (services)
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(content_chunks[1]);

    // Render clients list (left top)
    render_clients_list(
        f,
        left_chunks[0],
        &ordered_reports,
        current_time,
        app.selected_client_index,
    );

    // Render overview stats (left bottom)
    render_overview_stats(f, left_chunks[1], &ordered_reports, current_time);

    // Render selected client details (right top)
    let selected_client = app.get_selected_client();
    render_client_details(f, right_chunks[0], selected_client, current_time);

    // Render services overview (right bottom)
    render_services_overview(f, right_chunks[1], selected_client);

    // Render status bar (bottom)
    render_status_bar(f, main_chunks[1], &ordered_reports, current_time);
}

/// Renders the status bar with key bindings and system info
fn render_status_bar(
    f: &mut ratatui::Frame,
    area: Rect,
    reports: &[&ReportEntry],
    _current_time: u64,
) {
    let now = chrono::Utc::now();
    let time_str = now.format("%Y-%m-%d %H:%M:%S UTC").to_string();

    let status_content = Line::from(vec![
        Span::styled(
            "SwarmReport Sentinel",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" | ", Style::default().fg(Color::Gray)),
        Span::styled("q/ESC: Quit", Style::default().fg(Color::Yellow)),
        Span::styled(" | ", Style::default().fg(Color::Gray)),
        Span::styled("↑↓/jk: Navigate", Style::default().fg(Color::Yellow)),
        Span::styled(" | ", Style::default().fg(Color::Gray)),
        Span::styled("r/F5: Refresh", Style::default().fg(Color::Yellow)),
        Span::styled(" | ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("Clients: {}", reports.len()),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(" | ", Style::default().fg(Color::Gray)),
        Span::styled(time_str, Style::default().fg(Color::Gray)),
    ]);

    let status_bar = Paragraph::new(status_content).style(Style::default().bg(Color::DarkGray));

    f.render_widget(status_bar, area);
}

/// Renders the clients list with status indicators and selection highlighting
fn render_clients_list(
    f: &mut ratatui::Frame,
    area: Rect,
    reports: &[&ReportEntry],
    current_time: u64,
    selected_index: usize,
) {
    if reports.is_empty() {
        let no_clients = Paragraph::new("No clients connected")
            .block(Block::default().borders(Borders::ALL).title("Clients (0)"))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(no_clients, area);
        return;
    }

    let items: Vec<ListItem> = reports
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let seconds_since_update = current_time.saturating_sub(entry.last_updated);
            let status_color = get_status_color(seconds_since_update);
            let cpu_usage = parse_cpu_usage(&entry.report.cpu_usage);

            let status_icon = match seconds_since_update {
                0..=4 => "●",
                5..=30 => "◐",
                _ => "○",
            };

            let last_updated = std::time::UNIX_EPOCH + Duration::from_secs(entry.last_updated);
            let datetime = chrono::DateTime::<chrono::Utc>::from(last_updated);
            let time_str = datetime.format("%H:%M:%S").to_string();

            let content = Line::from(vec![
                Span::styled(
                    format!("{status_icon} "),
                    Style::default().fg(status_color),
                ),
                Span::styled(
                    format!("{:<15}", entry.report.hostname),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {:>15}", entry.report.ip_address),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!(" CPU:{cpu_usage:>5.1}%"),
                    Style::default().fg(if cpu_usage > 80.0 {
                        Color::Red
                    } else if cpu_usage > 60.0 {
                        Color::Yellow
                    } else {
                        Color::Green
                    }),
                ),
                Span::styled(format!(" {time_str}"), Style::default().fg(Color::Gray)),
            ]);

            let mut item = ListItem::new(content);

            // Highlight selected item
            if i == selected_index {
                item = item.style(Style::default().bg(Color::DarkGray));
            }

            item
        })
        .collect();

    let mut list_state = ListState::default();
    if selected_index < reports.len() {
        list_state.select(Some(selected_index));
    }

    let clients_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    "Clients ({}) - Use ↑↓/jk to navigate",
                    reports.len()
                ))
                .title_style(
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_stateful_widget(clients_list, area, &mut list_state);
}

/// Renders overview statistics
fn render_overview_stats(
    f: &mut ratatui::Frame,
    area: Rect,
    reports: &[&ReportEntry],
    current_time: u64,
) {
    let _total_clients = reports.len();
    let (online, warning, offline) = reports.iter().fold((0, 0, 0), |(on, warn, off), entry| {
        let seconds_since_update = current_time.saturating_sub(entry.last_updated);
        match seconds_since_update {
            0..=4 => (on + 1, warn, off),
            5..=30 => (on, warn + 1, off),
            _ => (on, warn, off + 1),
        }
    });

    let avg_cpu = if !reports.is_empty() {
        reports
            .iter()
            .map(|entry| parse_cpu_usage(&entry.report.cpu_usage))
            .sum::<f64>()
            / reports.len() as f64
    } else {
        0.0
    };

    let total_services = reports
        .iter()
        .map(|entry| entry.report.services.len())
        .sum::<usize>();
    let running_services = reports
        .iter()
        .map(|entry| {
            entry
                .report
                .services
                .iter()
                .filter(|s| s.status == "running")
                .count()
        })
        .sum::<usize>();

    let stats_content = vec![
        Line::from(vec![
            Span::styled(
                "Status: ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("● {online} "), Style::default().fg(Color::Green)),
            Span::styled(
                format!("◐ {warning} "),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(format!("○ {offline}"), Style::default().fg(Color::Red)),
        ]),
        Line::from(vec![
            Span::styled(
                "Avg CPU: ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{avg_cpu:.1}%"),
                Style::default().fg(if avg_cpu > 80.0 {
                    Color::Red
                } else if avg_cpu > 60.0 {
                    Color::Yellow
                } else {
                    Color::Green
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Services: ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{running_services}/{total_services} running"),
                Style::default().fg(Color::Cyan),
            ),
        ]),
    ];

    let stats_block = Paragraph::new(stats_content).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Overview")
            .title_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
    );

    f.render_widget(stats_block, area);
}

/// Renders detailed information for the selected client
fn render_client_details(
    f: &mut ratatui::Frame,
    area: Rect,
    selected_client: Option<&ReportEntry>,
    current_time: u64,
) {
    let Some(entry) = selected_client else {
        let no_details = Paragraph::new("No client selected")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Client Details"),
            )
            .style(Style::default().fg(Color::Gray));
        f.render_widget(no_details, area);
        return;
    };
    let seconds_since_update = current_time.saturating_sub(entry.last_updated);
    let status_color = get_status_color(seconds_since_update);
    let cpu_usage = parse_cpu_usage(&entry.report.cpu_usage);

    let last_updated = std::time::UNIX_EPOCH + Duration::from_secs(entry.last_updated);
    let datetime = chrono::DateTime::<chrono::Utc>::from(last_updated);
    let time_str = datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string();

    // Create a mini layout for the details
    let detail_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(area);

    // Basic info section
    let basic_info = vec![
        Line::from(vec![
            Span::styled(
                "Hostname: ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&entry.report.hostname, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled(
                "IP Address: ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&entry.report.ip_address, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled(
                "Node ID: ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&entry.report.node_id, Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled(
                "Last Update: ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(time_str, Style::default().fg(status_color)),
        ]),
        Line::from(vec![
            Span::styled(
                "Memory: ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                &entry.report.memory_usage,
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Disk: ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&entry.report.disk_usage, Style::default().fg(Color::Green)),
        ]),
    ];

    let basic_block = Paragraph::new(basic_info).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Client Details - {}", entry.report.hostname))
            .title_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
    );

    f.render_widget(basic_block, detail_chunks[0]);

    // CPU usage gauge
    let cpu_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("CPU Usage"))
        .gauge_style(Style::default().fg(if cpu_usage > 80.0 {
            Color::Red
        } else if cpu_usage > 60.0 {
            Color::Yellow
        } else {
            Color::Green
        }))
        .percent((cpu_usage as u16).min(100))
        .label(format!("{cpu_usage:.1}%"));

    f.render_widget(cpu_gauge, detail_chunks[1]);
}

/// Renders services for the selected client
fn render_services_overview(
    f: &mut ratatui::Frame,
    area: Rect,
    selected_client: Option<&ReportEntry>,
) {
    let Some(entry) = selected_client else {
        let no_services = Paragraph::new("No client selected")
            .block(Block::default().borders(Borders::ALL).title("Services"))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(no_services, area);
        return;
    };

    if entry.report.services.is_empty() {
        let no_services = Paragraph::new("No services running on this client")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Services - {}", entry.report.hostname))
                    .title_style(
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .style(Style::default().fg(Color::Gray));
        f.render_widget(no_services, area);
        return;
    }

    let items: Vec<ListItem> = entry
        .report
        .services
        .iter()
        .map(|service| {
            let status_color = match service.status.as_str() {
                "running" => Color::Green,
                "stopped" => Color::Red,
                _ => Color::Yellow,
            };

            let status_icon = match service.status.as_str() {
                "running" => "✓",
                "stopped" => "✗",
                _ => "?",
            };

            let update_indicator = if service.needs_update {
                " (update available)"
            } else {
                ""
            };

            let content = Line::from(vec![
                Span::styled(
                    format!("{status_icon} "),
                    Style::default().fg(status_color),
                ),
                Span::styled(
                    format!("{:<25}", service.name),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:<10}", service.status),
                    Style::default().fg(status_color),
                ),
                Span::styled(update_indicator, Style::default().fg(Color::Yellow)),
            ]);

            ListItem::new(content)
        })
        .collect();

    let services_list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(
                "Services - {} ({} total)",
                entry.report.hostname,
                entry.report.services.len()
            ))
            .title_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
    );

    f.render_widget(services_list, area);
}

pub async fn run_tui_display_only(
    state: SharedState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut last_refresh = std::time::Instant::now();
    let refresh_interval = Duration::from_millis(500); // Refresh every 500ms for more responsive UI

    loop {
        // Force refresh every interval for real-time updates
        let now = std::time::Instant::now();
        let should_refresh = now.duration_since(last_refresh) >= refresh_interval;

        if should_refresh {
            let app = state.lock().unwrap();
            terminal.draw(|f| ui(f, &app))?;
            last_refresh = now;
        }

        // Check for input with shorter polling interval
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('r') => {
                        // Force immediate refresh
                        let app = state.lock().unwrap();
                        terminal.draw(|f| ui(f, &app))?;
                        last_refresh = std::time::Instant::now();
                    }
                    KeyCode::F(5) => {
                        // F5 for refresh (common pattern)
                        let app = state.lock().unwrap();
                        terminal.draw(|f| ui(f, &app))?;
                        last_refresh = std::time::Instant::now();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        // Navigate up in client list
                        {
                            let mut app = state.lock().unwrap();
                            app.select_previous_client();
                        }
                        // Force immediate redraw to show selection change
                        let app = state.lock().unwrap();
                        terminal.draw(|f| ui(f, &app))?;
                        last_refresh = std::time::Instant::now();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        // Navigate down in client list
                        {
                            let mut app = state.lock().unwrap();
                            app.select_next_client();
                        }
                        // Force immediate redraw to show selection change
                        let app = state.lock().unwrap();
                        terminal.draw(|f| ui(f, &app))?;
                        last_refresh = std::time::Instant::now();
                    }
                    _ => {}
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
