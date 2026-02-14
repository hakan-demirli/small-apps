use crate::config::{AnchorConfig, Color, Config, LayerType};
use crate::parser::{parse_events, read_events_from_file};
use calloop::timer::{TimeoutAction, Timer};
use calloop::EventLoop;
use calloop_wayland_source::WaylandSource;
use chrono::{Datelike, Local, NaiveDate};
use log::{debug, error, info};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    shell::wlr_layer::{
        Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
        LayerSurfaceConfigure,
    },
    shell::WaylandSurface,
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use std::time::{Duration, Instant};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_region, wl_shm, wl_surface},
    Connection, Dispatch, Proxy, QueueHandle,
};

struct AppData {
    registry_state: RegistryState,
    output_state: OutputState,
    layer_shell: LayerShell,
    shm: Shm,
    compositor: CompositorState,

    layer_surface: Option<LayerSurface>,
    pool: Option<SlotPool>,
    width: u32,
    height: u32,
    configured: bool,
    loop_signal: calloop::LoopSignal,

    font: rusttype::Font<'static>,

    config: Config,

    last_check: Instant,
    cached_deadlines: Vec<(NaiveDate, String)>,
}

impl AppData {
    fn refresh_deadlines(&mut self) {
        let mut deadlines: Vec<(NaiveDate, String)> = Vec::new();

        if self.config.layer.target_dates_from_cli {
            for d_str in &self.config.layer.target_dates {
                if let Ok(dt) = NaiveDate::parse_from_str(d_str, "%Y-%m-%d") {
                    deadlines.push((dt, "CLI Target".to_string()));
                }
            }
        } else {
            let expanded_paths: Vec<String> = self
                .config
                .files
                .iter()
                .map(|p| shellexpand::tilde(p).to_string())
                .collect();

            let has_files = !expanded_paths.is_empty()
                && expanded_paths
                    .iter()
                    .any(|p| std::path::Path::new(p).exists());

            if has_files {
                let lines = read_events_from_file(&expanded_paths);
                let parsed = parse_events(&lines);

                let target_symbols: Vec<char> = self.config.symbols.chars().collect();

                for (date, events) in parsed {
                    if let Some((_, name, _)) = events
                        .iter()
                        .find(|(status, _, _)| target_symbols.contains(status))
                    {
                        deadlines.push((date, name.clone()));
                    }
                }
            }

            if deadlines.is_empty() {
                for d_str in &self.config.layer.target_dates {
                    if let Ok(dt) = NaiveDate::parse_from_str(d_str, "%Y-%m-%d") {
                        deadlines.push((dt, "Default Target".to_string()));
                    }
                }
            }
        }

        deadlines.sort_by_key(|(d, _)| *d);
        deadlines.dedup_by_key(|(d, _)| *d);

        self.cached_deadlines = deadlines;
        debug!(
            "Refreshed deadlines. Count: {} (from_cli: {})",
            self.cached_deadlines.len(),
            self.config.layer.target_dates_from_cli
        );
    }

    fn draw(&mut self, _qh: &QueueHandle<Self>) {
        if !self.configured || self.layer_surface.is_none() {
            return;
        }

        if self.last_check.elapsed() >= Duration::from_secs(5) {
            self.refresh_deadlines();
            self.last_check = Instant::now();
        }

        let now = Local::now().date_naive();
        let deadlines = &self.cached_deadlines;
        let start_date_str = &self.config.layer.start_date;

        let mut next_deadline = None;
        let mut prev_deadline = None;

        for (d, name) in deadlines {
            if *d > now {
                next_deadline = Some((*d, name.clone()));
                break;
            }
            prev_deadline = Some(*d);
        }

        if prev_deadline.is_none() {
            if let Ok(dt) = NaiveDate::parse_from_str(start_date_str, "%Y-%m-%d") {
                prev_deadline = Some(dt);
            } else {
                prev_deadline = NaiveDate::from_ymd_opt(now.year(), 1, 1);
            }
        }

        let mut panic_text = String::new();
        let (text, percent_burned) = if let Some((next, name)) = next_deadline {
            let prev = prev_deadline.unwrap();
            debug!(
                "Targeting deadline: '{}' ({}) starting from: {}",
                name, next, prev
            );

            let now_full = Local::now();
            let prev_full = prev
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap();
            let next_full = next
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap();

            let total_millis = (next_full - prev_full).num_milliseconds() as f64;
            let burned_millis = (now_full - prev_full).num_milliseconds() as f64;

            let percent_burned = if total_millis <= 0.0 {
                100.0
            } else {
                (burned_millis / total_millis) * 100.0
            };

            let percent_remaining = (100.0 - percent_burned).clamp(0.0, 100.0);

            let millis_remaining = (next_full - now_full).num_milliseconds().max(0);
            let days_remaining = millis_remaining as f64 / (1000.0 * 60.0 * 60.0 * 24.0);
            panic_text = format!("{:010.6}", days_remaining);

            debug!(
                "Update: burned={:.4}%, days_remaining={:.6}, millis_rem={}",
                percent_burned, days_remaining, millis_remaining
            );

            (format!("{:.4}%", percent_remaining), percent_burned)
        } else {
            ("ALL DONE".to_string(), 100.0)
        };

        let font = &self.font;
        let font_size = self.config.layer.font_size;
        let scale = rusttype::Scale::uniform(font_size);
        let v_metrics = font.v_metrics(scale);

        let glyphs: Vec<_> = font
            .layout(&text, scale, rusttype::point(0.0, 0.0))
            .collect();
        let text_width = glyphs
            .last()
            .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
            .unwrap_or(0.0);

        let padding_x = self.config.layer.text_padding_x as f32;
        let padding_y = self.config.layer.text_padding_y as f32;

        let box_left = 0.0;
        let box_top = 0.0;

        let x_start = box_left + padding_x;
        let y_start = box_top + padding_y + v_metrics.ascent;

        let panic_scale = rusttype::Scale::uniform(font_size * 0.6);
        let panic_v_metrics = font.v_metrics(panic_scale);

        let panic_width = if !panic_text.is_empty() {
            let panic_glyphs: Vec<_> = font
                .layout(&panic_text, panic_scale, rusttype::point(0.0, 0.0))
                .collect();
            panic_glyphs
                .last()
                .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let max_text_width = text_width.max(panic_width);

        let box_right = box_left + max_text_width + (padding_x * 2.0);

        let box_bottom = if !panic_text.is_empty() {
            let gap = 5.0;
            let panic_y = y_start - v_metrics.descent + panic_v_metrics.ascent + gap;
            panic_y - panic_v_metrics.descent + padding_y
        } else {
            y_start - v_metrics.descent + padding_y
        };

        let box_w = box_right - box_left;
        let box_h = box_bottom - box_top;

        let constrained_width = false;

        let target_width = if constrained_width {
            self.width
        } else {
            box_w.ceil() as u32
        };

        let target_height = box_h.ceil() as u32;

        if target_width != self.width || target_height != self.height {
            debug!("Resizing layer to {}x{}", target_width, target_height);
            let layer = self.layer_surface.as_ref().unwrap();
            layer.set_size(target_width, target_height);
            layer.commit();
            return;
        }

        let width = self.width;
        let height = self.height;
        let stride = width * 4;

        if self.pool.is_none() {
            let pool = SlotPool::new(width as usize * height as usize * 4, &self.shm)
                .expect("Failed to create pool");
            self.pool = Some(pool);
        }

        let pool = self.pool.as_mut().unwrap();

        if pool.len() < (width * height * 4) as usize {
            pool.resize((width * height * 4) as usize)
                .expect("Failed to resize pool");
        }

        let (buffer, canvas) = pool
            .create_buffer(
                width as i32,
                height as i32,
                stride as i32,
                wl_shm::Format::Argb8888,
            )
            .expect("create buffer");

        for byte in canvas.iter_mut() {
            *byte = 0;
        }

        let bg_color = self.config.layer.colors.background;
        let radius = 10.0;
        let anchor = self.config.layer.anchor;
        let x_off = self.config.layer.x;
        let y_off = self.config.layer.y;

        let anchored_top = matches!(anchor, AnchorConfig::TopLeft | AnchorConfig::TopRight);
        let anchored_bottom =
            matches!(anchor, AnchorConfig::BottomLeft | AnchorConfig::BottomRight);
        let anchored_left = matches!(anchor, AnchorConfig::TopLeft | AnchorConfig::BottomLeft);
        let anchored_right = matches!(anchor, AnchorConfig::TopRight | AnchorConfig::BottomRight);

        let round_top_left = !(anchored_top && y_off <= 0 || anchored_left && x_off <= 0);
        let round_top_right = !(anchored_top && y_off <= 0 || anchored_right && x_off <= 0);
        let round_bottom_left = !(anchored_bottom && y_off <= 0 || anchored_left && x_off <= 0);
        let round_bottom_right = !(anchored_bottom && y_off <= 0 || anchored_right && x_off <= 0);

        let min_x = (box_left - 1.0).max(0.0) as i32;
        let max_x = (box_right + 1.0).min(width as f32) as i32;
        let min_y = (box_top - 1.0).max(0.0) as i32;
        let max_y = (box_bottom + 1.0).min(height as f32) as i32;

        for y in min_y..max_y {
            for x in min_x..max_x {
                let fx = x as f32 + 0.5;
                let fy = y as f32 + 0.5;

                let cx = box_left + box_w * 0.5;
                let cy = box_top + box_h * 0.5;

                let dx = fx - cx;
                let dy = fy - cy;

                let is_right = dx > 0.0;
                let is_bottom = dy > 0.0;

                let should_round = match (is_right, is_bottom) {
                    (false, false) => round_top_left,
                    (true, false) => round_top_right,
                    (false, true) => round_bottom_left,
                    (true, true) => round_bottom_right,
                };

                let dist = if should_round {
                    let half_w = box_w * 0.5 - radius;
                    let half_h = box_h * 0.5 - radius;
                    let adx = dx.abs() - half_w;
                    let ady = dy.abs() - half_h;
                    (adx.max(0.0).powi(2) + ady.max(0.0).powi(2)).sqrt()
                        + adx.min(0.0).max(ady.min(0.0))
                        - radius
                } else {
                    let half_w = box_w * 0.5;
                    let half_h = box_h * 0.5;
                    let adx = dx.abs() - half_w;
                    let ady = dy.abs() - half_h;
                    adx.max(ady)
                };

                let alpha = 1.0 - dist.clamp(0.0, 1.0);

                if alpha > 0.0 {
                    let pixel_idx = (y as usize * width as usize + x as usize) * 4;

                    let out_a = (bg_color.a as f32 / 255.0) * alpha;
                    let out_r = bg_color.r as f32 * out_a;
                    let out_g = bg_color.g as f32 * out_a;
                    let out_b = bg_color.b as f32 * out_a;

                    let existing_a = canvas[pixel_idx + 3] as f32 / 255.0;
                    let existing_b = canvas[pixel_idx] as f32;
                    let existing_g = canvas[pixel_idx + 1] as f32;
                    let existing_r = canvas[pixel_idx + 2] as f32;

                    let inv_a = 1.0 - out_a;

                    canvas[pixel_idx] = (out_b + existing_b * inv_a) as u8;
                    canvas[pixel_idx + 1] = (out_g + existing_g * inv_a) as u8;
                    canvas[pixel_idx + 2] = (out_r + existing_r * inv_a) as u8;
                    canvas[pixel_idx + 3] = ((out_a + existing_a * inv_a) * 255.0) as u8;
                }
            }
        }

        let color = if percent_burned < 50.0 {
            self.config.layer.colors.green
        } else if percent_burned < 75.0 {
            self.config.layer.colors.yellow
        } else if percent_burned < 90.0 {
            self.config.layer.colors.orange
        } else {
            self.config.layer.colors.red
        };

        let mut draw_text = |text: &str, scale: rusttype::Scale, x: f32, y: f32, col: Color| {
            for glyph in font.layout(text, scale, rusttype::point(x, y)) {
                if let Some(bounding_box) = glyph.pixel_bounding_box() {
                    glyph.draw(|gx, gy, v| {
                        let px = gx as i32 + bounding_box.min.x;
                        let py = gy as i32 + bounding_box.min.y;
                        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                            let pixel_idx = (py as usize * width as usize + px as usize) * 4;

                            let v_gamma = v.powf(0.4545);
                            let v_clamped = v_gamma.clamp(0.0, 1.0);

                            if v_clamped > 0.05 {
                                let alpha_f = (col.a as f32 / 255.0) * v_clamped;

                                let existing_a = canvas[pixel_idx + 3] as f32 / 255.0;
                                let existing_b = canvas[pixel_idx] as f32;
                                let existing_g = canvas[pixel_idx + 1] as f32;
                                let existing_r = canvas[pixel_idx + 2] as f32;

                                let r_new = col.r as f32 * alpha_f;
                                let g_new = col.g as f32 * alpha_f;
                                let b_new = col.b as f32 * alpha_f;

                                let inv_a = 1.0 - alpha_f;

                                canvas[pixel_idx] = (b_new + existing_b * inv_a) as u8;
                                canvas[pixel_idx + 1] = (g_new + existing_g * inv_a) as u8;
                                canvas[pixel_idx + 2] = (r_new + existing_r * inv_a) as u8;

                                let out_a = alpha_f + existing_a * inv_a;
                                canvas[pixel_idx + 3] = (out_a * 255.0) as u8;
                            }
                        }
                    });
                }
            }
        };

        draw_text(&text, scale, x_start, y_start, color);

        if !panic_text.is_empty() {
            let panic_x_start = x_start;
            let panic_y_start = y_start - v_metrics.descent + panic_v_metrics.ascent + 5.0;

            draw_text(
                &panic_text,
                panic_scale,
                panic_x_start,
                panic_y_start,
                color,
            );
        }

        let surface = self.layer_surface.as_ref().unwrap().wl_surface();
        surface.attach(Some(buffer.wl_buffer()), 0, 0);
        surface.damage(0, 0, width as i32, height as i32);

        surface.commit();
    }
}

fn load_font_data(paths: &[String], family: Option<&str>) -> Vec<u8> {
    for path in paths {
        if let Ok(data) = std::fs::read(path) {
            return data;
        }
    }

    let family = family.unwrap_or("sans");
    debug!(
        "Standard paths failed, trying fc-match for family '{}'...",
        family
    );

    if let Ok(output) = std::process::Command::new("fc-match")
        .arg("--format=%{file}")
        .arg(family)
        .output()
    {
        if output.status.success() {
            let path_s = String::from_utf8_lossy(&output.stdout);
            let path = path_s.trim();
            debug!("fc-match found: {}", path);
            if let Ok(data) = std::fs::read(path) {
                return data;
            }
        }
    }

    error!("Warning: Could not find fonts.");
    error!("Please install standard fonts or ensure 'fc-match' is available.");
    std::process::exit(1);
}

impl CompositorHandler for AppData {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }
    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
    }
    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: wayland_client::protocol::wl_output::Transform,
    ) {
    }
    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for AppData {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for AppData {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.loop_signal.stop();
    }
    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        self.width = configure.new_size.0;
        self.height = configure.new_size.1;
        self.configured = true;

        if self.width == 0 {
            self.width = self.config.layer.width;
        }
        if self.height == 0 {
            self.height = self.config.layer.height;
        }

        self.draw(qh);
    }
}

impl ShmHandler for AppData {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for AppData {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    fn runtime_add_global(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: u32,
        _: &str,
        _: u32,
    ) {
    }
    fn runtime_remove_global(&mut self, _: &Connection, _: &QueueHandle<Self>, _: u32, _: &str) {}
}

impl Dispatch<wl_region::WlRegion, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &wl_region::WlRegion,
        _event: <wl_region::WlRegion as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

delegate_compositor!(AppData);
delegate_output!(AppData);
delegate_shm!(AppData);
delegate_layer!(AppData);
delegate_registry!(AppData);

pub fn run(config: Config) {
    env_logger::init();

    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
    let (globals, event_queue) = registry_queue_init::<AppData>(&conn).unwrap();
    let qh = event_queue.handle();
    let mut event_loop = EventLoop::<AppData>::try_new().unwrap();
    let loop_signal = event_loop.get_signal();

    let font_data = load_font_data(
        &config.layer.font_paths,
        config.layer.font_family.as_deref(),
    );
    let font = rusttype::Font::try_from_vec(font_data).expect("Error constructing Font");

    let layer_type = match config.layer.layer {
        LayerType::Background => Layer::Background,
        LayerType::Bottom => Layer::Bottom,
        LayerType::Top => Layer::Top,
        LayerType::Overlay => Layer::Overlay,
    };

    let anchor = match config.layer.anchor {
        AnchorConfig::TopLeft => Anchor::TOP | Anchor::LEFT,
        AnchorConfig::TopRight => Anchor::TOP | Anchor::RIGHT,
        AnchorConfig::BottomLeft => Anchor::BOTTOM | Anchor::LEFT,
        AnchorConfig::BottomRight => Anchor::BOTTOM | Anchor::RIGHT,
    };

    let mut app_data = AppData {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        layer_shell: LayerShell::bind(&globals, &qh).expect("layer shell not available"),
        shm: Shm::bind(&globals, &qh).expect("wl_shm is not available"),
        compositor: CompositorState::bind(&globals, &qh).expect("wl_compositor is not available"),
        layer_surface: None,
        pool: None,
        width: 0,
        height: config.layer.height,
        configured: false,
        loop_signal,
        font,
        config: config.clone(),
        last_check: Instant::now(),
        cached_deadlines: Vec::new(),
    };

    app_data.refresh_deadlines();

    let surface = app_data.compositor.create_surface(&qh);

    let layer = app_data.layer_shell.create_layer_surface(
        &qh,
        surface,
        layer_type,
        Some("floating_text"),
        None,
    );

    layer.set_anchor(anchor);

    let (margin_top, margin_right, margin_bottom, margin_left) = {
        let x = config.layer.x;
        let y = config.layer.y;

        let mut t = 0;
        let mut r = 0;
        let mut b = 0;
        let mut l = 0;

        if anchor.contains(Anchor::TOP) {
            t = y;
        } else if anchor.contains(Anchor::BOTTOM) {
            b = y;
        } else {
            t = y;
        }

        if anchor.contains(Anchor::LEFT) {
            l = x;
        } else if anchor.contains(Anchor::RIGHT) {
            r = x;
        } else {
            l = x;
        }

        (t, r, b, l)
    };

    layer.set_margin(margin_top, margin_right, margin_bottom, margin_left);

    let use_width = config.layer.width;

    layer.set_size(use_width, config.layer.height);
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.set_exclusive_zone(config.layer.exclusive_zone);

    let region = app_data.compositor.wl_compositor().create_region(&qh, ());
    layer.wl_surface().set_input_region(Some(&region));
    region.destroy();

    layer.commit();

    app_data.layer_surface = Some(layer);

    let timer = Timer::immediate();

    event_loop
        .handle()
        .insert_source(timer, move |_, _, app_data| {
            debug!("Timer fired");
            app_data.draw(&qh);
            TimeoutAction::ToDuration(Duration::from_millis(200))
        })
        .unwrap();

    event_loop
        .handle()
        .insert_source(
            WaylandSource::new(conn.clone(), event_queue),
            |_, queue, app_data| queue.dispatch_pending(app_data),
        )
        .unwrap();

    info!("Starting floating deadline counter (text only)...");

    loop {
        if event_loop.dispatch(None, &mut app_data).is_err() {
            break;
        }
    }
}
