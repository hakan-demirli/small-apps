use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use directories::ProjectDirs;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(rename = "files", alias = "file_paths")]
    pub files: Vec<String>,

    pub symbols: String,

    #[serde(default)]
    pub flow: FlowConfig,

    #[serde(default, rename = "deadlines_view", alias = "deadlines")]
    pub deadlines_view: DeadlinesViewConfig,

    #[serde(default)]
    pub layer: LayerToolConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct FlowConfig {}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct DeadlinesViewConfig {
    pub gradient_start: String,
    pub gradient_end: String,
}

impl Default for DeadlinesViewConfig {
    fn default() -> Self {
        Self {
            gradient_start: "#BD93F9".to_string(),
            gradient_end: "#7FD2E4".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct LayerToolConfig {
    pub font_paths: Vec<String>,
    pub font_family: Option<String>,
    pub width: u32,
    pub height: u32,
    pub text_padding_y: f64,
    pub text_padding_x: f64,
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    pub font_size: f32,
    pub anchor: AnchorConfig,
    pub layer: LayerType,
    pub exclusive_zone: i32,

    #[serde(rename = "target_dates", alias = "deadlines", default)]
    pub target_dates: Vec<String>,

    #[serde(skip)]
    pub target_dates_from_cli: bool,

    pub start_date: String,

    #[serde(default)]
    pub colors: Colors,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct Colors {
    pub background_darker: Color,
    pub background: Color,
    pub selection: Color,
    pub foreground: Color,
    pub comment: Color,
    pub cyan: Color,
    pub green: Color,
    pub orange: Color,
    pub pink: Color,
    pub purple: Color,
    pub red: Color,
    pub yellow: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            background_darker: Color {
                r: 30,
                g: 31,
                b: 41,
                a: 255,
            },
            background: Color {
                r: 40,
                g: 42,
                b: 54,
                a: 255,
            },
            selection: Color {
                r: 68,
                g: 71,
                b: 90,
                a: 255,
            },
            foreground: Color {
                r: 248,
                g: 248,
                b: 242,
                a: 255,
            },
            comment: Color {
                r: 98,
                g: 114,
                b: 164,
                a: 255,
            },
            cyan: Color {
                r: 139,
                g: 233,
                b: 253,
                a: 255,
            },
            green: Color {
                r: 80,
                g: 250,
                b: 123,
                a: 255,
            },
            orange: Color {
                r: 255,
                g: 184,
                b: 108,
                a: 255,
            },
            pink: Color {
                r: 255,
                g: 121,
                b: 198,
                a: 255,
            },
            purple: Color {
                r: 189,
                g: 147,
                b: 249,
                a: 255,
            },
            red: Color {
                r: 255,
                g: 85,
                b: 85,
                a: 255,
            },
            yellow: Color {
                r: 241,
                g: 250,
                b: 140,
                a: 255,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.a == 255 {
            serializer.serialize_str(&format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b))
        } else {
            serializer.serialize_str(&format!(
                "#{:02x}{:02x}{:02x}{:02x}",
                self.r, self.g, self.b, self.a
            ))
        }
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ColorVisitor;

        impl<'de> Visitor<'de> for ColorVisitor {
            type Value = Color;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a hex color string (e.g. #RRGGBB or #RRGGBBAA)")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let s = value.trim_start_matches('#');
                if s.len() == 6 {
                    let r = u8::from_str_radix(&s[0..2], 16).map_err(E::custom)?;
                    let g = u8::from_str_radix(&s[2..4], 16).map_err(E::custom)?;
                    let b = u8::from_str_radix(&s[4..6], 16).map_err(E::custom)?;
                    Ok(Color { r, g, b, a: 255 })
                } else if s.len() == 8 {
                    let r = u8::from_str_radix(&s[0..2], 16).map_err(E::custom)?;
                    let g = u8::from_str_radix(&s[2..4], 16).map_err(E::custom)?;
                    let b = u8::from_str_radix(&s[4..6], 16).map_err(E::custom)?;
                    let a = u8::from_str_radix(&s[6..8], 16).map_err(E::custom)?;
                    Ok(Color { r, g, b, a })
                } else {
                    Err(E::custom("invalid hex color length"))
                }
            }
        }

        deserializer.deserialize_str(ColorVisitor)
    }
}

impl Default for LayerToolConfig {
    fn default() -> Self {
        let now = Local::now().date_naive();
        let format_str = "%Y-%m-%d";

        Self {
            font_paths: vec![],
            font_family: None,
            width: 100,
            height: 100,
            text_padding_x: 5.0,
            text_padding_y: 5.0,
            x: 0,
            y: 0,
            font_size: 24.0,
            anchor: AnchorConfig::TopLeft,
            layer: LayerType::Overlay,
            exclusive_zone: -1,
            target_dates: vec![],
            target_dates_from_cli: false,
            start_date: now.format(format_str).to_string(),
            colors: Colors::default(),
        }
    }
}

use clap::ValueEnum;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum AnchorConfig {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum LayerType {
    Background,
    Bottom,
    Top,
    Overlay,
}

use clap::Subcommand;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(after_help = r#"EXAMPLES:
    # Show flow view (default) with custom files
    riveroftime flow --file ~/notes.md ~/tasks.md

    # Show deadlines filtered by urgent symbol
    riveroftime deadlines --symbols "!" --file ~/todo.md

    # Show calendar with events highlighted
    riveroftime calendar --show-events --file ~/events.md

    # Launch Wayland layer overlay with specific target dates
    riveroftime layer --target-dates 2026-06-01 2026-12-31

    # Use a custom config file
    riveroftime -c ~/.config/custom.toml flow

SYMBOLS:
    Events use checkbox syntax with status characters:
      [ ] (space) - Pending       [x] - Completed      [>] - Delegated
      [!] - Urgent                [-] - Cancelled      [/] - In progress
      [?] - Needs clarification   [o] - Blocked        [<] - Deadline marker

CONFIG:
    Default config location: ~/.config/riveroftime/config.toml
    Use --ignore-config to use built-in defaults instead."#)]
pub struct Args {
    #[arg(
        short,
        long,
        global = true,
        help = "Path to config file [default: ~/.config/riveroftime/config.toml]"
    )]
    pub config: Option<PathBuf>,

    #[arg(
        long,
        global = true,
        help = "Use built-in defaults, ignore config file"
    )]
    pub ignore_config: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    Flow {
        #[arg(long, num_args = 1.., help = "Markdown files to read events from [default: from config]")]
        file: Option<Vec<String>>,

        #[arg(
            long,
            help = "Status characters to filter (e.g. \"<\" for deadlines, \"!\" for urgent)"
        )]
        symbols: Option<String>,
    },

    Deadlines {
        #[arg(long, num_args = 1.., help = "Markdown files to read events from [default: from config]")]
        file: Option<Vec<String>>,

        #[arg(
            long,
            help = "Status characters to filter (e.g. \"<\" for deadlines) [default: <]"
        )]
        symbols: Option<String>,

        #[arg(long, help = "Hex color for nearest deadline [default: #BD93F9]")]
        gradient_start: Option<String>,

        #[arg(long, help = "Hex color for furthest deadline [default: #7FD2E4]")]
        gradient_end: Option<String>,
    },

    Calendar {
        #[arg(long, num_args = 1.., help = "Markdown files to read events from (use with --show-events)")]
        file: Option<Vec<String>>,

        #[arg(long, help = "Load and highlight dates with events")]
        show_events: bool,
    },

    Layer {
        #[arg(long, num_args = 1.., help = "Markdown files to parse deadlines from (ignored if --target-dates given)")]
        file: Option<Vec<String>>,

        #[arg(
            long,
            help = "Status characters to filter when parsing files [default: from config]"
        )]
        symbols: Option<String>,

        #[arg(long, num_args = 1.., value_name = "DATE", help = "Explicit dates to count down to (overrides --file)")]
        target_dates: Option<Vec<String>>,

        #[arg(
            long,
            value_name = "DATE",
            help = "Start date for countdown calculation [default: today]"
        )]
        start_date: Option<String>,

        #[arg(short = 'W', long, help = "Width of the layer [default: from config]")]
        width: Option<u32>,

        #[arg(short = 'H', long, help = "Height of the layer [default: from config]")]
        height: Option<u32>,

        #[arg(
            short,
            long,
            allow_hyphen_values = true,
            help = "X offset of the layer [default: from config]"
        )]
        x: Option<i32>,

        #[arg(
            short,
            long,
            allow_hyphen_values = true,
            help = "Y offset of the layer [default: from config]"
        )]
        y: Option<i32>,

        #[arg(long, help = "Anchor position of the layer [default: from config]")]
        anchor: Option<AnchorConfig>,
    },
}

impl Default for Config {
    fn default() -> Self {
        Self {
            files: vec!["~/notes.md".to_string()],
            symbols: "<".to_string(),
            flow: FlowConfig::default(),
            deadlines_view: DeadlinesViewConfig::default(),
            layer: LayerToolConfig::default(),
        }
    }
}

pub fn load_config(args: &Args) -> Result<Config> {
    if args.ignore_config {
        return Ok(Config::default());
    }

    let config_path = if let Some(ref path) = args.config {
        path.clone()
    } else {
        get_default_config_path()
    };

    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory at {:?}", parent))?;
        }
        let default_config = Config::default();
        let toml_string = toml::to_string_pretty(&default_config)
            .context("Failed to serialize default config")?;

        std::fs::write(&config_path, toml_string)
            .with_context(|| format!("Failed to write default config to {:?}", config_path))?;

        println!("Created default config at {:?}", config_path);
        return Ok(default_config);
    }

    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file at {:?}", config_path))?;

    toml::from_str(&content).with_context(|| "Failed to parse config file")
}

fn get_default_config_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "riveroftime") {
        let config_dir = proj_dirs.config_dir();
        config_dir.join("config.toml")
    } else {
        PathBuf::from("config.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_deserialize_6_digit_hex() {
        let toml_str = "color = \"#FF0000\"";
        #[derive(Deserialize)]
        struct Wrapper {
            color: Color,
        }
        let w: Wrapper = toml::from_str(toml_str).unwrap();
        assert_eq!(w.color.r, 255);
        assert_eq!(w.color.g, 0);
        assert_eq!(w.color.b, 0);
        assert_eq!(w.color.a, 255);
    }

    #[test]
    fn test_color_deserialize_8_digit_hex_with_alpha() {
        let toml_str = "color = \"#FF000080\"";
        #[derive(Deserialize)]
        struct Wrapper {
            color: Color,
        }
        let w: Wrapper = toml::from_str(toml_str).unwrap();
        assert_eq!(w.color.r, 255);
        assert_eq!(w.color.g, 0);
        assert_eq!(w.color.b, 0);
        assert_eq!(w.color.a, 128);
    }

    #[test]
    fn test_color_deserialize_lowercase() {
        let toml_str = "color = \"#abcdef\"";
        #[derive(Deserialize)]
        struct Wrapper {
            color: Color,
        }
        let w: Wrapper = toml::from_str(toml_str).unwrap();
        assert_eq!(w.color.r, 171);
        assert_eq!(w.color.g, 205);
        assert_eq!(w.color.b, 239);
        assert_eq!(w.color.a, 255);
    }

    #[test]
    fn test_color_deserialize_without_hash() {
        let toml_str = "color = \"AABBCC\"";
        #[derive(Deserialize)]
        struct Wrapper {
            color: Color,
        }
        let w: Wrapper = toml::from_str(toml_str).unwrap();
        assert_eq!(w.color.r, 170);
        assert_eq!(w.color.g, 187);
        assert_eq!(w.color.b, 204);
    }

    #[test]
    fn test_color_deserialize_invalid_length_fails() {
        let toml_str = "color = \"#FFF\"";
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Wrapper {
            color: Color,
        }
        let result: Result<Wrapper, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_color_serialize_without_alpha() {
        let color = Color {
            r: 255,
            g: 128,
            b: 64,
            a: 255,
        };
        #[derive(Serialize)]
        #[allow(dead_code)]
        struct Wrapper {
            color: Color,
        }
        let toml_out = toml::to_string(&Wrapper { color }).unwrap();
        assert!(toml_out.contains("#ff8040"));
    }

    #[test]
    fn test_color_serialize_with_alpha() {
        let color = Color {
            r: 255,
            g: 128,
            b: 64,
            a: 128,
        };
        #[derive(Serialize)]
        #[allow(dead_code)]
        struct Wrapper {
            color: Color,
        }
        let toml_out = toml::to_string(&Wrapper { color }).unwrap();
        assert!(toml_out.contains("#ff804080"));
    }

    #[test]
    fn test_color_roundtrip() {
        let original = Color {
            r: 189,
            g: 147,
            b: 249,
            a: 255,
        };
        #[derive(Serialize, Deserialize)]
        struct Wrapper {
            color: Color,
        }
        let toml_out = toml::to_string(&Wrapper { color: original }).unwrap();
        let w: Wrapper = toml::from_str(&toml_out).unwrap();
        assert_eq!(original.r, w.color.r);
        assert_eq!(original.g, w.color.g);
        assert_eq!(original.b, w.color.b);
        assert_eq!(original.a, w.color.a);
    }

    #[test]
    fn test_config_default_has_files() {
        let config = Config::default();
        assert!(!config.files.is_empty());
        assert!(config.files[0].contains("notes"));
    }

    #[test]
    fn test_config_default_symbols() {
        let config = Config::default();
        assert_eq!(config.symbols, "<");
    }

    #[test]
    fn test_deadlines_view_config_default_gradients() {
        let config = DeadlinesViewConfig::default();
        assert_eq!(config.gradient_start, "#BD93F9");
        assert_eq!(config.gradient_end, "#7FD2E4");
    }

    #[test]
    fn test_layer_tool_config_default_dimensions() {
        let config = LayerToolConfig::default();
        assert_eq!(config.width, 100);
        assert_eq!(config.height, 100);
        assert_eq!(config.font_size, 24.0);
    }

    #[test]
    fn test_layer_tool_config_default_has_target_dates() {
        let config = LayerToolConfig::default();
        assert_eq!(config.target_dates.len(), 0);
    }

    #[test]
    fn test_colors_default_dracula_theme() {
        let colors = Colors::default();
        assert_eq!(colors.purple.r, 189);
        assert_eq!(colors.purple.g, 147);
        assert_eq!(colors.purple.b, 249);
        assert_eq!(colors.background.r, 40);
        assert_eq!(colors.background.g, 42);
        assert_eq!(colors.background.b, 54);
    }

    #[test]
    fn test_anchor_config_serialize() {
        #[derive(Serialize)]
        struct Wrapper {
            anchor: AnchorConfig,
        }
        let toml_out = toml::to_string(&Wrapper {
            anchor: AnchorConfig::TopLeft,
        })
        .unwrap();
        assert!(toml_out.contains("top_left"));
    }

    #[test]
    fn test_anchor_config_deserialize() {
        let toml_str = "anchor = \"bottom_right\"";
        #[derive(Deserialize)]
        struct Wrapper {
            anchor: AnchorConfig,
        }
        let w: Wrapper = toml::from_str(toml_str).unwrap();
        assert!(matches!(w.anchor, AnchorConfig::BottomRight));
    }

    #[test]
    fn test_layer_type_serialize() {
        #[derive(Serialize)]
        struct Wrapper {
            layer: LayerType,
        }
        let toml_out = toml::to_string(&Wrapper {
            layer: LayerType::Overlay,
        })
        .unwrap();
        assert!(toml_out.contains("overlay"));
    }

    #[test]
    fn test_layer_type_deserialize() {
        let toml_str = "layer = \"background\"";
        #[derive(Deserialize)]
        struct Wrapper {
            layer: LayerType,
        }
        let w: Wrapper = toml::from_str(toml_str).unwrap();
        assert!(matches!(w.layer, LayerType::Background));
    }

    #[test]
    fn test_config_from_toml_minimal() {
        let toml_str = r#"
            files = ["~/notes.md"]
            symbols = " "
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.files, vec!["~/notes.md"]);
        assert_eq!(config.symbols, " ");
    }

    #[test]
    fn test_config_from_toml_with_deadlines_view() {
        let toml_str = "files = [\"~/test.md\"]\nsymbols = \"!\"\n\n[deadlines_view]\ngradient_start = \"#FF0000\"\ngradient_end = \"#00FF00\"";
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.deadlines_view.gradient_start, "#FF0000");
        assert_eq!(config.deadlines_view.gradient_end, "#00FF00");
    }

    #[test]
    fn test_config_alias_file_paths() {
        let toml_str = "file_paths = [\"~/alias_test.md\"]\nsymbols = \"x\"";
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.files, vec!["~/alias_test.md"]);
    }

    #[test]
    fn test_selective_color_override() {
        let toml_str = r##"
            files = ["test.md"]
            symbols = "<"
            [layer.colors]
            green = "#ffb86c"
        "##;
        let config: Config = toml::from_str(toml_str).unwrap();

        assert_eq!(config.layer.colors.green.r, 255);
        assert_eq!(config.layer.colors.green.g, 184);
        assert_eq!(config.layer.colors.green.b, 108);

        let default_colors = Colors::default();
        assert_eq!(config.layer.colors.purple.r, default_colors.purple.r);
        assert_eq!(config.layer.colors.purple.g, default_colors.purple.g);
        assert_eq!(config.layer.colors.purple.b, default_colors.purple.b);
    }
}
