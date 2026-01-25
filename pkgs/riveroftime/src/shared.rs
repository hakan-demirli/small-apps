use ratatui::style::Color;
use std::collections::HashMap;

pub const MAX_FADE_DAYS: f64 = 30.0;
pub const FADE_TARGET_RGB: (u8, u8, u8) = (85, 85, 85);
pub const DAYS_BEFORE_TODAY: i64 = 5;

pub fn get_base_colors() -> HashMap<&'static str, (u8, u8, u8)> {
    let mut m = HashMap::new();
    m.insert("day", (128, 128, 128));
    m.insert("event", (127, 210, 228));
    m.insert("countdown", (189, 147, 249));
    m.insert("header", (85, 85, 85));
    m.insert("today", (255, 255, 255));
    m.insert("unhandled_past", (255, 80, 80));
    m
}

pub fn get_status_symbols() -> HashMap<char, char> {
    let mut m = HashMap::new();
    m.insert(' ', '○');
    m.insert('x', '✓');
    m.insert('X', '✓');
    m.insert('>', '\u{203a}'); // ›
    m.insert('!', '!');
    m.insert('-', '-');
    m.insert('/', '…');
    m.insert('?', '?');
    m.insert('o', '⊘');
    m.insert('I', '\u{2139}'); // ℹ
    m.insert('L', '⚲');
    m.insert('*', '*');
    m.insert('<', '\u{2039}'); // ‹
    m
}

pub fn get_status_colors() -> HashMap<char, (u8, u8, u8)> {
    let mut m = HashMap::new();
    m.insert(' ', (127, 210, 228));
    m.insert('x', (85, 85, 85));
    m.insert('X', (85, 85, 85));
    m.insert('>', (150, 120, 180));
    m.insert('!', (255, 140, 80));
    m.insert('-', (85, 85, 85));
    m.insert('/', (180, 200, 100));
    m.insert('?', (220, 180, 100));
    m.insert('o', (220, 87, 125));
    m.insert('I', (100, 180, 220));
    m.insert('L', (100, 220, 120));
    m.insert('*', (150, 150, 150));
    m.insert('<', (120, 150, 220));
    m
}

pub fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return (255, 255, 255);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    (r, g, b)
}

pub fn get_faded_color(base_rgb: (u8, u8, u8), distance_from_today: i64) -> Color {
    if distance_from_today <= 0 {
        return Color::Rgb(base_rgb.0, base_rgb.1, base_rgb.2);
    }

    let fade_factor = (distance_from_today as f64).abs() / MAX_FADE_DAYS;
    let fade_factor = fade_factor.min(1.0);

    let r = base_rgb.0 as f64 + (FADE_TARGET_RGB.0 as f64 - base_rgb.0 as f64) * fade_factor;
    let g = base_rgb.1 as f64 + (FADE_TARGET_RGB.1 as f64 - base_rgb.1 as f64) * fade_factor;
    let b = base_rgb.2 as f64 + (FADE_TARGET_RGB.2 as f64 - base_rgb.2 as f64) * fade_factor;

    Color::Rgb(r as u8, g as u8, b as u8)
}

pub fn interpolate_color(start_rgb: (u8, u8, u8), end_rgb: (u8, u8, u8), fraction: f64) -> Color {
    let fraction = fraction.clamp(0.0, 1.0);

    let r = start_rgb.0 as f64 + (end_rgb.0 as f64 - start_rgb.0 as f64) * fraction;
    let g = start_rgb.1 as f64 + (end_rgb.1 as f64 - start_rgb.1 as f64) * fraction;
    let b = start_rgb.2 as f64 + (end_rgb.2 as f64 - start_rgb.2 as f64) * fraction;

    Color::Rgb(r as u8, g as u8, b as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_rgb_valid_with_hash() {
        assert_eq!(hex_to_rgb("#FF0000"), (255, 0, 0));
        assert_eq!(hex_to_rgb("#00FF00"), (0, 255, 0));
        assert_eq!(hex_to_rgb("#0000FF"), (0, 0, 255));
    }

    #[test]
    fn test_hex_to_rgb_valid_without_hash() {
        assert_eq!(hex_to_rgb("FF0000"), (255, 0, 0));
        assert_eq!(hex_to_rgb("ABCDEF"), (171, 205, 239));
    }

    #[test]
    fn test_hex_to_rgb_lowercase() {
        assert_eq!(hex_to_rgb("#ff8800"), (255, 136, 0));
        assert_eq!(hex_to_rgb("abcdef"), (171, 205, 239));
    }

    #[test]
    fn test_hex_to_rgb_mixed_case() {
        assert_eq!(hex_to_rgb("#FfAa00"), (255, 170, 0));
    }

    #[test]
    fn test_hex_to_rgb_black_and_white() {
        assert_eq!(hex_to_rgb("#000000"), (0, 0, 0));
        assert_eq!(hex_to_rgb("#FFFFFF"), (255, 255, 255));
    }

    #[test]
    fn test_hex_to_rgb_invalid_length_returns_white() {
        assert_eq!(hex_to_rgb("#FFF"), (255, 255, 255));
        assert_eq!(hex_to_rgb("#FFFFFFFF"), (255, 255, 255));
        assert_eq!(hex_to_rgb(""), (255, 255, 255));
    }

    #[test]
    fn test_hex_to_rgb_invalid_chars_returns_partial() {
        assert_eq!(hex_to_rgb("#GGGGGG"), (255, 255, 255));
    }

    #[test]
    fn test_interpolate_color_at_zero() {
        let result = interpolate_color((0, 0, 0), (255, 255, 255), 0.0);
        assert_eq!(result, Color::Rgb(0, 0, 0));
    }

    #[test]
    fn test_interpolate_color_at_one() {
        let result = interpolate_color((0, 0, 0), (255, 255, 255), 1.0);
        assert_eq!(result, Color::Rgb(255, 255, 255));
    }

    #[test]
    fn test_interpolate_color_at_half() {
        let result = interpolate_color((0, 0, 0), (255, 255, 255), 0.5);
        assert_eq!(result, Color::Rgb(127, 127, 127));
    }

    #[test]
    fn test_interpolate_color_clamps_above_one() {
        let result = interpolate_color((0, 0, 0), (100, 100, 100), 2.0);
        assert_eq!(result, Color::Rgb(100, 100, 100));
    }

    #[test]
    fn test_interpolate_color_clamps_below_zero() {
        let result = interpolate_color((100, 100, 100), (200, 200, 200), -1.0);
        assert_eq!(result, Color::Rgb(100, 100, 100));
    }

    #[test]
    fn test_interpolate_color_different_channels() {
        let result = interpolate_color((0, 100, 200), (100, 200, 100), 0.5);
        assert_eq!(result, Color::Rgb(50, 150, 150));
    }

    #[test]
    fn test_get_faded_color_today_returns_base() {
        let base = (127, 210, 228);
        let result = get_faded_color(base, 0);
        assert_eq!(result, Color::Rgb(127, 210, 228));
    }

    #[test]
    fn test_get_faded_color_past_returns_base() {
        let base = (127, 210, 228);
        let result = get_faded_color(base, -5);
        assert_eq!(result, Color::Rgb(127, 210, 228));
    }

    #[test]
    fn test_get_faded_color_max_days_returns_target() {
        let base = (255, 255, 255);
        let result = get_faded_color(base, MAX_FADE_DAYS as i64);
        assert_eq!(
            result,
            Color::Rgb(FADE_TARGET_RGB.0, FADE_TARGET_RGB.1, FADE_TARGET_RGB.2)
        );
    }

    #[test]
    fn test_get_faded_color_beyond_max_clamped() {
        let base = (255, 255, 255);
        let result = get_faded_color(base, 100);
        assert_eq!(
            result,
            Color::Rgb(FADE_TARGET_RGB.0, FADE_TARGET_RGB.1, FADE_TARGET_RGB.2)
        );
    }

    #[test]
    fn test_get_faded_color_halfway() {
        let base = (255, 255, 255);
        let result = get_faded_color(base, (MAX_FADE_DAYS / 2.0) as i64);
        if let Color::Rgb(r, g, b) = result {
            assert!(r < 255 && r > FADE_TARGET_RGB.0);
            assert!(g < 255 && g > FADE_TARGET_RGB.1);
            assert!(b < 255 && b > FADE_TARGET_RGB.2);
        } else {
            panic!("Expected Color::Rgb");
        }
    }

    #[test]
    fn test_get_base_colors_contains_expected_keys() {
        let colors = get_base_colors();
        assert!(colors.contains_key("day"));
        assert!(colors.contains_key("event"));
        assert!(colors.contains_key("countdown"));
        assert!(colors.contains_key("header"));
        assert!(colors.contains_key("today"));
        assert!(colors.contains_key("unhandled_past"));
    }

    #[test]
    fn test_get_base_colors_values() {
        let colors = get_base_colors();
        assert_eq!(colors.get("today"), Some(&(255, 255, 255)));
        assert_eq!(colors.get("header"), Some(&(85, 85, 85)));
    }

    #[test]
    fn test_get_status_symbols_contains_expected_mappings() {
        let symbols = get_status_symbols();
        assert_eq!(symbols.get(&' '), Some(&'○'));
        assert_eq!(symbols.get(&'x'), Some(&'✓'));
        assert_eq!(symbols.get(&'X'), Some(&'✓'));
        assert_eq!(symbols.get(&'!'), Some(&'!'));
        assert_eq!(symbols.get(&'/'), Some(&'…'));
    }

    #[test]
    fn test_get_status_symbols_all_entries() {
        let symbols = get_status_symbols();
        assert_eq!(symbols.len(), 13);
    }

    #[test]
    fn test_get_status_colors_contains_expected_mappings() {
        let colors = get_status_colors();
        assert_eq!(colors.get(&' '), Some(&(127, 210, 228)));
        assert_eq!(colors.get(&'x'), Some(&(85, 85, 85)));
        assert_eq!(colors.get(&'!'), Some(&(255, 140, 80)));
    }

    #[test]
    fn test_get_status_colors_all_entries() {
        let colors = get_status_colors();
        assert_eq!(colors.len(), 13);
    }
}
