use crate::parser::{parse_events, read_events_from_file};
use crate::shared::{
    get_base_colors, get_faded_color, get_status_colors, get_status_symbols, DAYS_BEFORE_TODAY,
    FADE_TARGET_RGB,
};
use anyhow::Result;
use chrono::{Datelike, Duration, Local};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Terminal,
};
use std::{
    io,
    time::{self, Instant},
};

pub fn run(file_paths: Option<Vec<String>>) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let paths = file_paths.unwrap_or_default();

    let base_colors = get_base_colors();
    let status_symbols = get_status_symbols();
    let status_colors_map = get_status_colors();

    let header_rgb = base_colors.get("header").unwrap();
    let header_color = Color::Rgb(header_rgb.0, header_rgb.1, header_rgb.2);

    let past_event_rgb = FADE_TARGET_RGB;
    let past_event_color = Color::Rgb(past_event_rgb.0, past_event_rgb.1, past_event_rgb.2);

    let unhandled_past_rgb = base_colors.get("unhandled_past").unwrap();
    let unhandled_past_color = Color::Rgb(
        unhandled_past_rgb.0,
        unhandled_past_rgb.1,
        unhandled_past_rgb.2,
    );

    let today_rgb = base_colors.get("today").unwrap();
    let today_color = Color::Rgb(today_rgb.0, today_rgb.1, today_rgb.2);

    let countdown_base_rgb = base_colors.get("countdown").unwrap();
    let day_base_rgb = base_colors.get("day").unwrap();

    let tick_rate = time::Duration::from_secs(5);
    let mut last_tick = Instant::now();

    loop {
        let expanded_paths: Vec<String> = paths
            .iter()
            .map(|p| shellexpand::tilde(p).to_string())
            .collect();

        let lines = read_events_from_file(&expanded_paths);
        let events_dict = parse_events(&lines);

        terminal.draw(|f| {
            let size = f.area();
            let height = size.height as i64;
            let available_lines = height.saturating_sub(1);

            let mut renderables: Vec<Line> = Vec::new();
            let mut lines_printed = 0;

            let today = Local::now().date_naive();
            let mut current_date = today - Duration::days(DAYS_BEFORE_TODAY);
            let mut last_printed_month = None;

            while lines_printed < available_lines {
                let month = current_date.month();

                if last_printed_month != Some(month) {
                    if lines_printed + 1 > available_lines {
                        break;
                    }
                    let month_name = chrono::Month::try_from(month as u8).unwrap().name();
                    let header_text = month_name.to_uppercase().to_string();
                    renderables.push(Line::from(Span::styled(
                        header_text,
                        Style::default().fg(header_color),
                    )));
                    lines_printed += 1;
                    last_printed_month = Some(month);
                }

                let events_for_day = events_dict.get(&current_date);
                let day_num = current_date.day();
                let distance = (current_date - today).num_days();

                let mut has_unhandled_past = false;
                if distance < 0 {
                    if let Some(evts) = events_for_day {
                        if evts.iter().any(|(s, _, _)| !['x', 'X', '>'].contains(s)) {
                            has_unhandled_past = true;
                        }
                    }
                }

                let (day_style, countdown_style) = if distance < 0 {
                    let d_col = if has_unhandled_past {
                        unhandled_past_color
                    } else {
                        past_event_color
                    };
                    (
                        Style::default().fg(d_col),
                        Style::default().fg(past_event_color),
                    )
                } else if distance == 0 {
                    (
                        Style::default()
                            .fg(today_color)
                            .add_modifier(Modifier::BOLD),
                        Style::default().fg(Color::Rgb(
                            countdown_base_rgb.0,
                            countdown_base_rgb.1,
                            countdown_base_rgb.2,
                        )),
                    )
                } else {
                    let d_col = get_faded_color(*day_base_rgb, distance);
                    let c_col = get_faded_color(*countdown_base_rgb, distance);
                    (Style::default().fg(d_col), Style::default().fg(c_col))
                };

                let day_gutter = Span::styled(format!("{:>2}", day_num), day_style);

                let mut spans = vec![Span::raw("  "), day_gutter, Span::raw(" ")];

                if let Some(evts) = events_for_day {
                    if !evts.is_empty() {
                        let countdown_text = if distance < 0 {
                            " -".to_string()
                        } else {
                            format!("{:>2}", distance)
                        };
                        spans.push(Span::styled(countdown_text, countdown_style));
                        spans.push(Span::raw(" "));

                        let (status_char, event_name, _) = &evts[0];
                        let symbol = status_symbols.get(status_char).unwrap_or(&'○');
                        let base_status_col = status_colors_map
                            .get(status_char)
                            .unwrap_or(&(127, 210, 228));

                        let status_col = if distance < 0 {
                            if !['x', 'X', '>'].contains(status_char) {
                                unhandled_past_color
                            } else {
                                past_event_color
                            }
                        } else if distance == 0 {
                            Color::Rgb(base_status_col.0, base_status_col.1, base_status_col.2)
                        } else {
                            get_faded_color(*base_status_col, distance)
                        };

                        spans.push(Span::styled(
                            format!("{} {}", symbol, event_name),
                            Style::default().fg(status_col),
                        ));

                        renderables.push(Line::from(spans));
                        lines_printed += 1;

                        for (status_char, event_name, _) in evts.iter().skip(1) {
                            if lines_printed >= available_lines {
                                break;
                            }

                            let symbol = status_symbols.get(status_char).unwrap_or(&'○');
                            let base_status_col = status_colors_map
                                .get(status_char)
                                .unwrap_or(&(127, 210, 228));

                            let status_col = if distance < 0 {
                                if !['x', 'X', '>'].contains(status_char) {
                                    unhandled_past_color
                                } else {
                                    past_event_color
                                }
                            } else if distance == 0 {
                                Color::Rgb(base_status_col.0, base_status_col.1, base_status_col.2)
                            } else {
                                get_faded_color(*base_status_col, distance)
                            };

                            let indent = " ".repeat(9);
                            renderables.push(Line::from(vec![
                                Span::raw(indent),
                                Span::styled(
                                    format!("{} {}", symbol, event_name),
                                    Style::default().fg(status_col),
                                ),
                            ]));
                            lines_printed += 1;
                        }
                    } else {
                        renderables.push(Line::from(spans));
                        lines_printed += 1;
                    }
                } else {
                    renderables.push(Line::from(spans));
                    lines_printed += 1;
                }

                current_date += Duration::days(1);
            }

            let paragraph = Paragraph::new(renderables);
            f.render_widget(paragraph, size);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| time::Duration::from_secs(0));

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
