pub fn find_occurrences(source_lines: &[String], search_block_str: &str) -> (Vec<usize>, usize) {
    let src_strict: Vec<String> = source_lines
        .iter()
        .map(|s| s.trim_end().to_string())
        .collect();

    let search_lines_strict: Vec<String> = search_block_str
        .lines()
        .map(|s| s.trim_end().to_string())
        .collect();

    let matches = find_sublist(&src_strict, &search_lines_strict);
    if !matches.is_empty() {
        return (matches, search_lines_strict.len());
    }

    let search_block_trimmed = search_block_str.trim_matches(|c| c == '\n' || c == '\r');
    if search_block_trimmed != search_block_str {
        let search_lines_trimmed: Vec<String> = search_block_trimmed
            .lines()
            .map(|s| s.trim_end().to_string())
            .collect();

        if !search_lines_trimmed.is_empty() {
            let matches = find_sublist(&src_strict, &search_lines_trimmed);
            if !matches.is_empty() {
                return (matches, search_lines_trimmed.len());
            }
        }
    }

    let src_loose: Vec<String> = source_lines.iter().map(|s| s.trim().to_string()).collect();

    let search_lines_trimmed_iter = search_block_str
        .trim_matches(|c| c == '\n' || c == '\r')
        .lines();

    let search_lines_loose: Vec<String> = search_lines_trimmed_iter
        .map(|s| s.trim().to_string())
        .collect();

    if search_lines_loose.is_empty() {
        return (vec![], 0);
    }

    let matches = find_sublist(&src_loose, &search_lines_loose);

    let len = search_lines_loose.len();

    (matches, len)
}

fn find_sublist<T: PartialEq>(full_list: &[T], sub_list: &[T]) -> Vec<usize> {
    let mut matches = Vec::new();
    let n = full_list.len();
    let m = sub_list.len();

    if m == 0 {
        return matches;
    }

    for i in 0..=n.saturating_sub(m) {
        if &full_list[i..i + m] == sub_list {
            matches.push(i);
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_occurrences_strategies() {
        let src = vec!["a\n".to_string(), "  b\n".to_string(), "c\n".to_string()];

        let (idxs, len) = find_occurrences(&src, "  b");
        assert_eq!(idxs, vec![1]);
        assert_eq!(len, 1);

        let (idxs, len) = find_occurrences(&src, "\n  b\n");
        assert_eq!(idxs, vec![1]);
        assert_eq!(len, 1);

        let src_indented = vec![
            "    x\n".to_string(),
            "    y\n".to_string(),
            "    z\n".to_string(),
        ];
        let block_flat = "x\ny\nz";
        let (idxs, len) = find_occurrences(&src_indented, block_flat);
        assert_eq!(idxs, vec![0]);
        assert_eq!(len, 3);
    }
}
