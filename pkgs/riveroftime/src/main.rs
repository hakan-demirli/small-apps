mod config;
mod layer;
mod parser;
mod shared;
mod tui;

use clap::Parser;
use config::{load_config, Args, Command};

fn main() {
    let args = Args::parse();
    let config = match load_config(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {:#}", e);
            std::process::exit(1);
        }
    };

    match args.command {
        Some(Command::Deadlines {
            file,
            symbols,
            gradient_start,
            gradient_end,
        }) => {
            let paths = file.or_else(|| Some(config.files.clone()));
            let symbols = symbols.or_else(|| Some(config.symbols.clone()));
            let start_hex = gradient_start.unwrap_or(config.deadlines_view.gradient_start.clone());
            let end_hex = gradient_end.unwrap_or(config.deadlines_view.gradient_end.clone());

            if let Err(e) = tui::deadlines::run(paths, symbols, start_hex, end_hex) {
                eprintln!("Error running deadlines view: {}", e);
            }
        }

        Some(Command::Flow { file, symbols: _ }) => {
            let paths = file.or_else(|| Some(config.files.clone()));
            if let Err(e) = tui::flow::run(paths) {
                eprintln!("Error running flow view: {}", e);
            }
        }

        None => {
            let paths = Some(config.files.clone());
            if let Err(e) = tui::flow::run(paths) {
                eprintln!("Error running flow view: {}", e);
            }
        }

        Some(Command::Calendar { file, show_events }) => {
            let events = if show_events {
                let paths = file.or_else(|| Some(config.files.clone()));
                if let Some(p) = paths {
                    let lines = parser::read_events_from_file(&p);
                    Some(parser::parse_events(&lines))
                } else {
                    None
                }
            } else {
                None
            };
            tui::calendar::run(events);
        }

        Some(Command::Layer {
            file,
            symbols,
            target_dates,
            start_date,
            width,
            height,
            x,
            y,
            anchor,
        }) => {
            let mut final_config = config.clone();

            if let Some(files) = file {
                final_config.files = files;
            }

            if let Some(symbols) = symbols {
                final_config.symbols = symbols;
            }

            if let Some(dates) = target_dates {
                final_config.layer.target_dates = dates;
                final_config.layer.target_dates_from_cli = true;
            }

            if let Some(start) = start_date {
                final_config.layer.start_date = start;
            }

            if let Some(w) = width {
                final_config.layer.width = w;
            }

            if let Some(h) = height {
                final_config.layer.height = h;
            }

            if let Some(x_val) = x {
                final_config.layer.x = x_val;
            }

            if let Some(y_val) = y {
                final_config.layer.y = y_val;
            }

            if let Some(a) = anchor {
                final_config.layer.anchor = a;
            }

            layer::run(final_config);
        }
    }
}
