use crate::parser::ParsedEvents;
use chrono::{Datelike, Local};
use crossterm::style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor};
use crossterm::ExecutableCommand;
use std::io::stdout;

pub fn run(events: Option<ParsedEvents>) {
    let now = Local::now().date_naive();

    let mut months = Vec::new();
    let mut y = now.year();
    let mut m = now.month();

    for _ in 0..3 {
        months.push((y, m));
        m += 1;
        if m > 12 {
            m = 1;
            y += 1;
        }
    }

    let mut stdout = stdout();

    for (i, (y, m)) in months.iter().enumerate() {
        let month_name = chrono::Month::try_from(*m as u8).unwrap().name();
        let title = format!("{} {}", month_name, y);

        if *y == now.year() && *m == now.month() {
            let _ = stdout.execute(SetForegroundColor(Color::Green));
            let _ = stdout.execute(SetAttribute(Attribute::Bold));
        } else {
            let _ = stdout.execute(SetForegroundColor(Color::DarkGrey));
            let _ = stdout.execute(SetAttribute(Attribute::Bold));
        }

        let title_str = format!("{:^20}", title);
        if i < months.len() - 1 {
            print!("{}  ", title_str);
        } else {
            print!("{}", title_str.trim_end());
        }
        let _ = stdout.execute(ResetColor);
    }
    println!();

    for i in 0..3 {
        let _ = stdout.execute(SetForegroundColor(Color::Blue));
        if i < 2 {
            print!("Mo Tu We Th Fr Sa Su  ");
        } else {
            print!("Mo Tu We Th Fr Sa Su");
        }
        let _ = stdout.execute(ResetColor);
    }
    println!();

    let grids: Vec<Vec<Vec<String>>> = months
        .iter()
        .map(|&(y, m)| generate_month_grid(y, m, now, events.as_ref()))
        .collect();

    let mut max_needed_rows = 0;
    for grid in &grids {
        for (row_idx, row) in grid.iter().enumerate() {
            if row.iter().any(|day| day.trim() != "") && row_idx + 1 > max_needed_rows {
                max_needed_rows = row_idx + 1;
            }
        }
    }

    for i in 0..max_needed_rows {
        for (j, grid) in grids.iter().enumerate() {
            let row_str = if i < grid.len() {
                grid[i].join(" ")
            } else {
                " ".repeat(20)
            };

            if j < grids.len() - 1 {
                print!("{}  ", row_str);
            } else {
                print!("{}", row_str.trim_end());
            }
        }
        if i < max_needed_rows - 1 {
            println!();
        }
    }
}

fn generate_month_grid(
    year: i32,
    month: u32,
    today: chrono::NaiveDate,
    events: Option<&ParsedEvents>,
) -> Vec<Vec<String>> {
    let first_day = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();

    let start_weekday = first_day.weekday().num_days_from_monday();

    let days_in_month = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    }
    .signed_duration_since(first_day)
    .num_days();

    let mut weeks = Vec::new();
    let mut current_week = Vec::new();

    for _ in 0..start_weekday {
        current_week.push("  ".to_string());
    }

    for day in 1..=days_in_month {
        let current_date = chrono::NaiveDate::from_ymd_opt(year, month, day as u32).unwrap();
        let s_day = format!("{:>2}", day);

        let reset = "\x1b[0m";
        let gray = "\x1b[90m";
        let weekend_color = "\x1b[38;5;246m";
        let reverse = "\x1b[7m";
        let underline = "\x1b[4m";
        let event_color = "\x1b[38;5;214m";

        let has_event = if let Some(evts) = events {
            evts.contains_key(&current_date)
        } else {
            false
        };

        let styled_day = if current_date == today {
            if has_event {
                format!("{}{}{}{}{}", reverse, event_color, s_day, reset, reset)
            } else {
                format!("{}{}{}", reverse, s_day, reset)
            }
        } else if has_event {
            format!("{}{}{}{}", underline, event_color, s_day, reset)
        } else if current_date < today {
            format!("{}{}{}", gray, s_day, reset)
        } else {
            let is_weekend = current_week.len() >= 5;
            if is_weekend {
                format!("{}{}{}", weekend_color, s_day, reset)
            } else {
                s_day
            }
        };

        current_week.push(styled_day);

        if current_week.len() == 7 {
            weeks.push(current_week);
            current_week = Vec::new();
        }
    }

    if !current_week.is_empty() {
        while current_week.len() < 7 {
            current_week.push("  ".to_string());
        }
        weeks.push(current_week);
    }

    while weeks.len() < 6 {
        weeks.push(vec!["  ".to_string(); 7]);
    }

    weeks
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_generate_month_grid() {
        let today = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        let grid = generate_month_grid(2025, 1, today, None);

        assert_eq!(grid.len(), 6);
        for row in &grid {
            assert_eq!(row.len(), 7);
        }

        let week0 = &grid[0];
        assert_eq!(week0[0], "  ");
        assert_eq!(week0[1], "  ");
        assert!(week0[2].contains("1"));

        let week4 = &grid[4];
        assert!(week4[0].contains("27"));
        assert!(week4[4].contains("31"));
        assert_eq!(week4[5], "  ");
    }

    #[test]
    fn test_generate_month_grid_february_non_leap_year() {
        let today = NaiveDate::from_ymd_opt(2025, 2, 10).unwrap();
        let grid = generate_month_grid(2025, 2, today, None);

        assert_eq!(grid.len(), 6);
        let week0 = &grid[0];
        assert_eq!(week0[0], "  ");
        assert_eq!(week0[4], "  ");
        assert!(week0[5].contains("1"));

        let mut found_28 = false;
        for week in &grid {
            for day in week {
                if day.contains("28") {
                    found_28 = true;
                }
                assert!(!day.contains("29") || day.trim().is_empty());
            }
        }
        assert!(found_28);
    }

    #[test]
    fn test_generate_month_grid_february_leap_year() {
        let today = NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();
        let grid = generate_month_grid(2024, 2, today, None);

        let mut found_29 = false;
        for week in &grid {
            for day in week {
                if day.contains("29") {
                    found_29 = true;
                }
            }
        }
        assert!(found_29);
    }

    #[test]
    fn test_generate_month_grid_december_year_boundary() {
        let today = NaiveDate::from_ymd_opt(2025, 12, 25).unwrap();
        let grid = generate_month_grid(2025, 12, today, None);

        assert_eq!(grid.len(), 6);
        let mut found_31 = false;
        for week in &grid {
            for day in week {
                if day.contains("31") {
                    found_31 = true;
                }
            }
        }
        assert!(found_31);
    }

    #[test]
    fn test_generate_month_grid_month_starting_monday() {
        let today = NaiveDate::from_ymd_opt(2025, 9, 15).unwrap();
        let grid = generate_month_grid(2025, 9, today, None);

        let week0 = &grid[0];
        assert!(week0[0].contains("1"));
    }

    #[test]
    fn test_generate_month_grid_month_starting_sunday() {
        let today = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let grid = generate_month_grid(2025, 6, today, None);

        let week0 = &grid[0];
        assert_eq!(week0[0], "  ");
        assert_eq!(week0[5], "  ");
        assert!(week0[6].contains("1"));
    }

    #[test]
    fn test_generate_month_grid_today_has_reverse_style() {
        let today = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let grid = generate_month_grid(2025, 1, today, None);

        let mut found_today_styled = false;
        for week in &grid {
            for day in week {
                if day.contains("15") && day.contains("\x1b[7m") {
                    found_today_styled = true;
                }
            }
        }
        assert!(
            found_today_styled,
            "Today should have reverse video styling"
        );
    }

    #[test]
    fn test_generate_month_grid_past_days_are_gray() {
        let today = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let grid = generate_month_grid(2025, 1, today, None);

        let mut found_gray_past = false;
        for week in &grid {
            for day in week {
                if day.contains("10") && day.contains("\x1b[90m") {
                    found_gray_past = true;
                }
            }
        }
        assert!(found_gray_past, "Past days should have gray styling");
    }

    #[test]
    fn test_generate_month_grid_with_events() {
        use std::collections::BTreeMap;

        let today = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let event_date = NaiveDate::from_ymd_opt(2025, 1, 20).unwrap();

        let mut events: ParsedEvents = BTreeMap::new();
        events.insert(event_date, vec![(' ', "Test Event".to_string(), 1)]);

        let grid = generate_month_grid(2025, 1, today, Some(&events));

        let mut found_event_styled = false;
        for week in &grid {
            for day in week {
                if day.contains("20") && day.contains("\x1b[4m") && day.contains("\x1b[38;5;214m") {
                    found_event_styled = true;
                }
            }
        }
        assert!(
            found_event_styled,
            "Event days should have underline and event color"
        );
    }

    #[test]
    fn test_generate_month_grid_today_with_event() {
        use std::collections::BTreeMap;

        let today = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        let mut events: ParsedEvents = BTreeMap::new();
        events.insert(today, vec![(' ', "Today Event".to_string(), 1)]);

        let grid = generate_month_grid(2025, 1, today, Some(&events));

        let mut found_today_event_styled = false;
        for week in &grid {
            for day in week {
                if day.contains("15") && day.contains("\x1b[7m") && day.contains("\x1b[38;5;214m") {
                    found_today_event_styled = true;
                }
            }
        }
        assert!(
            found_today_event_styled,
            "Today with event should have reverse + event color"
        );
    }

    #[test]
    fn test_generate_month_grid_future_weekends_styled() {
        let today = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let grid = generate_month_grid(2025, 1, today, None);

        let mut found_weekend_styled = false;
        for week in &grid {
            if week.len() == 7 {
                let saturday = &week[5];
                if saturday.contains("18") && saturday.contains("\x1b[38;5;246m") {
                    found_weekend_styled = true;
                }
            }
        }
        assert!(
            found_weekend_styled,
            "Future weekends should have weekend color"
        );
    }
}
