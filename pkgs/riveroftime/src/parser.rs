use chrono::NaiveDate;
use log::{debug, info, trace, warn};
use regex::Regex;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub type EventList = Vec<(char, String, usize)>;
pub type ParsedEvents = BTreeMap<NaiveDate, EventList>;

pub fn read_events_from_file<P: AsRef<Path>>(file_paths: &[P]) -> Vec<String> {
    let mut all_lines = Vec::new();

    for path in file_paths {
        let path = path.as_ref();
        if !path.exists() {
            warn!("Events file not found at {:?}", path);
            continue;
        }

        match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();
                info!("Successfully read {} lines from {:?}", lines.len(), path);
                all_lines.extend(lines);
            }
            Err(e) => {
                warn!("Error reading events file {:?}: {}", path, e);
            }
        }
    }
    all_lines
}

pub fn parse_events(event_lines: &[String]) -> ParsedEvents {
    info!("Starting to parse {} lines", event_lines.len());
    let mut parsed: ParsedEvents = BTreeMap::new();

    let bracket_pattern = Regex::new(r"\[(\d{1,2})[/\.-](\d{1,2})(?:[/\.-](\d{2,4}))?\]").unwrap();
    let prefix_pattern = Regex::new(r"^(\d{1,2})[/\.-](\d{1,2})(?:[/\.-](\d{2,4}))?:").unwrap();
    let status_pattern = Regex::new(r"^\*?\s*\[(.)\]\s*").unwrap();

    let mut context_stack: HashMap<usize, String> = HashMap::new();

    for (i, line) in event_lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        trace!("Processing line: '{}'", line);
        let indent_level = line.len() - line.trim_start().len();

        let stale_indents: Vec<usize> = context_stack
            .keys()
            .filter(|&&indent| indent >= indent_level)
            .cloned()
            .collect();
        for indent in stale_indents {
            context_stack.remove(&indent);
            trace!("Popped context at indent {}", indent);
        }

        let cleaned_line = line.trim();

        let is_header = cleaned_line.ends_with(':')
            && !bracket_pattern.is_match(line)
            && !prefix_pattern.is_match(line);

        if is_header {
            let tag = cleaned_line
                .strip_suffix(':')
                .unwrap_or(cleaned_line)
                .trim_start_matches(['*', ' '])
                .trim()
                .to_string();
            context_stack.insert(indent_level, tag.clone());
            trace!("Pushed new context at indent {}: '{}'", indent_level, tag);
            continue;
        }

        let match_result = bracket_pattern
            .find(line)
            .or_else(|| prefix_pattern.find(line));

        if let Some(_mat) = match_result {
            let parent_tag = if !context_stack.is_empty() {
                let parent_indent = context_stack.keys().filter(|&&k| k < indent_level).max();
                parent_indent.and_then(|k| context_stack.get(k)).cloned()
            } else {
                None
            };

            let mut status_char = ' ';
            let mut event_name_str;

            let caps = if let Some(c) = bracket_pattern.captures(line) {
                let match_str = c.get(0).unwrap().as_str();
                let temp_name = line.replace(match_str, "");
                let temp_name = temp_name.trim();
                let temp_name = if temp_name.starts_with(':') {
                    temp_name.trim_start_matches(':').trim()
                } else {
                    temp_name
                };

                if let Some(status_match) = status_pattern.captures(temp_name) {
                    status_char = status_match
                        .get(1)
                        .map(|m| m.as_str().chars().next().unwrap_or(' '))
                        .unwrap_or(' ');
                    let end = status_match.get(0).unwrap().end();
                    event_name_str = temp_name[end..].trim().to_string();
                } else {
                    event_name_str = temp_name.to_string();
                }
                c
            } else {
                let c = prefix_pattern.captures(line).unwrap();
                let end = c.get(0).unwrap().end();
                let rest = line[end..].trim();

                if let Some(status_match) = status_pattern.captures(rest) {
                    status_char = status_match
                        .get(1)
                        .map(|m| m.as_str().chars().next().unwrap_or(' '))
                        .unwrap_or(' ');
                    let status_end = status_match.get(0).unwrap().end();
                    event_name_str = rest[status_end..].trim().to_string();
                } else {
                    event_name_str = rest.to_string();
                }
                c
            };

            event_name_str = event_name_str
                .trim_matches(':')
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join(" ");

            if let Some(pt) = parent_tag {
                event_name_str = format!("{}: {}", pt, event_name_str);
                trace!("Applied tag '{}' to event.", pt);
            }

            let day_str = caps.get(1).map_or("", |m| m.as_str());
            let month_str = caps.get(2).map_or("", |m| m.as_str());
            let year_opt = caps.get(3).map(|m| m.as_str());

            let now = chrono::Local::now();
            let year_str = match year_opt {
                Some(y) => {
                    if y.len() == 2 {
                        format!("20{}", y)
                    } else {
                        y.to_string()
                    }
                }
                None => now.format("%Y").to_string(),
            };

            let date_str = format!("{}/{}/{}", day_str, month_str, year_str);
            if let Ok(event_date) = NaiveDate::parse_from_str(&date_str, "%d/%m/%Y") {
                if event_name_str.is_empty() {
                    event_name_str = "Untitled Event".to_string();
                }

                debug!(
                    "Parsed date '{}' with event '{}'",
                    event_date, event_name_str
                );
                parsed
                    .entry(event_date)
                    .or_default()
                    .push((status_char, event_name_str, i));
            } else {
                warn!("Failed to parse date: {}", date_str);
            }
        } else {
            trace!("Line skipped: '{}'", line);
        }
    }

    parsed
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Local};

    #[test]
    fn test_parse_events() {
        let test_cases = vec![
            "[01/11/2025]".to_string(),
            "[01-11-2025]".to_string(),
            "[01.11.2025]".to_string(),
            "[1.11.2025]".to_string(),
            "[1.11.26]".to_string(),
            "[8/11/26]".to_string(),
            "[8.11/29]".to_string(),
            "[8/11.29]".to_string(),
            "[8.11.38]".to_string(),
            "* [8/11.29] do smth1".to_string(),
            "* do smth2 [8/11.29]".to_string(),
            "* do smth3[8/11.29]".to_string(),
            "* do smth4[01-11-2025]".to_string(),
            "    * do smth5[01-11-2025]".to_string(),
            "        * do smth6[01-11-2025]".to_string(),
            "        * [8/11.29] do smth7".to_string(),
            "    * [8/11.29] do smth8".to_string(),
            "[8/11.29]: do smth9".to_string(),
            "do smth10 [8/11.29]".to_string(),
            "do smth11:[8/11.29]".to_string(),
            "do smth12: [8/11.29]".to_string(),
            "do smth13: [8/11.29]".to_string(),
            "* [ ] task1 [15/11/2025]".to_string(),
            "* [x] task2 [15/11/2025]".to_string(),
            "* [X] task3 [15/11/2025]".to_string(),
            "* [-] task4 [15/11/2025]".to_string(),
            "* [a] task5 [15/11/2025]".to_string(),
            "[15/11/2025] * [ ] task6".to_string(),
            "[15/11/2025]: * [x] task7".to_string(),
            "* [!] urgent1 [16/11/2025]".to_string(),
            "* [>] delegated1 [16/11/2025]".to_string(),
            "* [/] inprogress1 [16/11/2025]".to_string(),
            "* [?] clarify1 [16/11/2025]".to_string(),
        ];

        let actual_result = parse_events(&test_cases);

        let verify = |y, m, d, expected: Vec<(char, &str)>| {
            let date = NaiveDate::from_ymd_opt(y, m, d).unwrap();
            let events = actual_result.get(&date).unwrap();

            let mut actual_simplified: Vec<(char, String)> =
                events.iter().map(|(c, s, _)| (*c, s.clone())).collect();
            actual_simplified.sort();

            let mut expected_sorted: Vec<(char, String)> =
                expected.iter().map(|(c, s)| (*c, s.to_string())).collect();
            expected_sorted.sort();

            assert_eq!(
                actual_simplified, expected_sorted,
                "Mismatch for date {}",
                date
            );
        };

        verify(
            2025,
            11,
            1,
            vec![
                (' ', "Untitled Event"),
                (' ', "Untitled Event"),
                (' ', "Untitled Event"),
                (' ', "Untitled Event"),
                (' ', "* do smth4"),
                (' ', "* do smth5"),
                (' ', "* do smth6"),
            ],
        );

        verify(
            2025,
            11,
            15,
            vec![
                (' ', "task1"),
                ('x', "task2"),
                ('X', "task3"),
                ('-', "task4"),
                ('a', "task5"),
                (' ', "task6"),
                ('x', "task7"),
            ],
        );

        verify(
            2025,
            11,
            16,
            vec![
                ('!', "urgent1"),
                ('>', "delegated1"),
                ('/', "inprogress1"),
                ('?', "clarify1"),
            ],
        );

        verify(2026, 11, 1, vec![(' ', "Untitled Event")]);
        verify(2026, 11, 8, vec![(' ', "Untitled Event")]);

        verify(
            2029,
            11,
            8,
            vec![
                (' ', "Untitled Event"),
                (' ', "Untitled Event"),
                (' ', "* do smth1"),
                (' ', "* do smth2"),
                (' ', "* do smth3"),
                (' ', "* do smth7"),
                (' ', "* do smth8"),
                (' ', "do smth9"),
                (' ', "do smth10"),
                (' ', "do smth11"),
                (' ', "do smth12"),
                (' ', "do smth13"),
            ],
        );

        verify(2038, 11, 8, vec![(' ', "Untitled Event")]);
    }

    #[test]
    fn test_parse_events_optional_year() {
        let current_year = Local::now().year();
        let test_cases = vec![
            "* MICRO:".to_string(),
            "  * [!] [02/02] bong".to_string(),
            "[05/05] Cinco de Mayo".to_string(),
            "10/10: Ten Ten".to_string(),
        ];

        let actual_result = parse_events(&test_cases);

        let verify = |m, d, expected: (char, &str)| {
            let date = NaiveDate::from_ymd_opt(current_year, m, d).unwrap();
            let events = actual_result
                .get(&date)
                .unwrap_or_else(|| panic!("No events for {}", date));
            let (status, name, _) = &events[0];
            assert_eq!(*status, expected.0);
            assert_eq!(name, expected.1);
        };

        verify(2, 2, ('!', "MICRO: bong"));
        verify(5, 5, (' ', "Cinco de Mayo"));
        verify(10, 10, (' ', "Ten Ten"));
    }

    #[test]
    fn test_read_events_from_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file1 = NamedTempFile::new().unwrap();
        writeln!(file1, "event1").unwrap();

        let mut file2 = NamedTempFile::new().unwrap();
        writeln!(file2, "event2").unwrap();

        let paths = vec![
            file1.path().to_str().unwrap().to_string(),
            file2.path().to_str().unwrap().to_string(),
        ];

        let lines = read_events_from_file(&paths);
        assert_eq!(lines, vec!["event1", "event2"]);
    }
}
