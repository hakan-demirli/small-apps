use crate::parser::{parse_events, read_events_from_file};
use crate::shared::{get_status_symbols, hex_to_rgb, interpolate_color};
use anyhow::Result;
use chrono::Local;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Constraint,
    style::Style,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

pub fn run(
    file_paths: Option<Vec<String>>,
    symbols: Option<String>,
    start_hex: String,
    end_hex: String,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let paths = file_paths.unwrap_or_default();

    let target_symbols: Vec<char> = if let Some(s) = symbols {
        s.chars().collect()
    } else {
        vec!['<']
    };

    let start_rgb = hex_to_rgb(&start_hex);
    let end_rgb = hex_to_rgb(&end_hex);

    let tick_rate = Duration::from_secs(5);
    let mut last_tick = Instant::now();

    loop {
        let lines = read_events_from_file(&paths);
        let parsed_events = parse_events(&lines);

        let today = Local::now().date_naive();
        let mut all_events = Vec::new();

        for (date, events) in parsed_events {
            let days_remaining = (date - today).num_days();
            for (status, name, line_num) in events {
                if target_symbols.contains(&status) {
                    all_events.push((days_remaining, line_num, status, name));
                }
            }
        }

        all_events.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        let total_items = all_events.len();

        terminal.draw(|f| {
            let size = f.area();

            if all_events.is_empty() {
                let p = Paragraph::new("No upcoming deadlines found.")
                    .block(Block::default().borders(Borders::NONE));
                f.render_widget(p, size);
            } else {
                let mut rows = Vec::new();
                let status_symbols = get_status_symbols();

                for (i, (days, _, status, name)) in all_events.iter().enumerate() {
                    let fraction = if total_items > 1 {
                        i as f64 / (total_items - 1) as f64
                    } else {
                        0.0
                    };

                    let color = interpolate_color(start_rgb, end_rgb, fraction);
                    let style = Style::default().fg(color);

                    let symbol_char = status_symbols.get(status).unwrap_or(&'○');

                    rows.push(Row::new(vec![
                        Cell::from(format!("{}", days)).style(style),
                        Cell::from(format!("{}", symbol_char)).style(style),
                        Cell::from(name.clone()).style(style),
                    ]));
                }

                let widths = [
                    Constraint::Length(5),
                    Constraint::Length(3),
                    Constraint::Percentage(100),
                ];

                let table = Table::new(rows, widths).block(Block::default().borders(Borders::NONE));

                f.render_widget(table, size);
            }
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q')
                    || key.code == KeyCode::Esc
                    || key.code == KeyCode::Char('c')
                        && key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
