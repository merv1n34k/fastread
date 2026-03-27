use eframe::egui;
use std::sync::Arc;
use web_time::Instant;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    eframe::run_native(
        "FastRead",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([640.0, 520.0])
                .with_min_inner_size([400.0, 300.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            load_fonts(&cc.egui_ctx);
            let app = App::default();
            app.apply_theme(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    load_fonts(&cc.egui_ctx);
                    let app = App::default();
                    app.apply_theme(&cc.egui_ctx);
                    Ok(Box::new(app))
                }),
            )
            .await
            .expect("failed to start eframe");
    });
}

fn load_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let paths = [
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/Library/Fonts/Arial Unicode.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        "C:\\Windows\\Fonts\\arial.ttf",
    ];
    for path in paths {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert(
                "fallback".into(),
                Arc::new(egui::FontData::from_owned(data)),
            );
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts.families.entry(family).or_default().push("fallback".into());
            }
            break;
        }
    }
    ctx.set_fonts(fonts);
}

#[derive(Clone, Copy)]
struct Theme {
    bg: egui::Color32,
    textarea_bg: egui::Color32,
    text_hi: egui::Color32,
    text_dim: egui::Color32,
    text_muted: egui::Color32,
    accent: egui::Color32,
    track: egui::Color32,
}

impl Theme {
    fn dark() -> Self {
        Self {
            bg: egui::Color32::from_rgb(12, 12, 12),
            textarea_bg: egui::Color32::from_rgb(20, 20, 20),
            text_hi: egui::Color32::from_rgb(220, 220, 220),
            text_dim: egui::Color32::from_rgb(90, 90, 90),
            text_muted: egui::Color32::from_rgb(60, 60, 60),
            accent: egui::Color32::from_rgb(220, 50, 50),
            track: egui::Color32::from_rgb(30, 30, 30),
        }
    }

    fn light() -> Self {
        Self {
            bg: egui::Color32::from_rgb(245, 245, 245),
            textarea_bg: egui::Color32::from_rgb(235, 235, 235),
            text_hi: egui::Color32::from_rgb(20, 20, 20),
            text_dim: egui::Color32::from_rgb(120, 120, 120),
            text_muted: egui::Color32::from_rgb(170, 170, 170),
            accent: egui::Color32::from_rgb(200, 40, 40),
            track: egui::Color32::from_rgb(220, 220, 220),
        }
    }
}

struct App {
    text: String,
    words: Vec<String>,
    idx: usize,
    playing: bool,
    wpm: u32,
    last_tick: Instant,
    focused: bool,
    dark: bool,
    theme: Theme,
}

impl Default for App {
    fn default() -> Self {
        Self {
            text: String::new(),
            words: vec![],
            idx: 0,
            playing: false,
            wpm: 300,
            last_tick: Instant::now(),
            focused: false,
            dark: true,
            theme: Theme::dark(),
        }
    }
}

impl App {
    fn apply_theme(&self, ctx: &egui::Context) {
        let t = &self.theme;
        let mut vis = if self.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        vis.panel_fill = t.bg;
        vis.extreme_bg_color = t.bg;
        vis.widgets.noninteractive.bg_fill = t.textarea_bg;
        vis.widgets.inactive.bg_fill = t.textarea_bg;
        vis.widgets.active.bg_fill = t.textarea_bg;
        vis.widgets.hovered.bg_fill = t.textarea_bg;
        // kill all outlines/strokes on widgets
        vis.widgets.active.bg_stroke = egui::Stroke::NONE;
        vis.widgets.hovered.bg_stroke = egui::Stroke::NONE;
        vis.widgets.inactive.bg_stroke = egui::Stroke::NONE;
        vis.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
        vis.widgets.active.fg_stroke = egui::Stroke::new(1.0, t.text_dim);
        vis.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, t.text_dim);
        // selection highlight (text selection, not focus ring)
        vis.selection.bg_fill = t.accent.linear_multiply(0.15);
        vis.selection.stroke = egui::Stroke::NONE;
        ctx.set_visuals(vis);
    }

    fn toggle_theme(&mut self, ctx: &egui::Context) {
        self.dark = !self.dark;
        self.theme = if self.dark { Theme::dark() } else { Theme::light() };
        self.apply_theme(ctx);
    }

    fn reparse(&mut self) {
        self.words = parse_words(&self.text);
        self.idx = 0;
        self.playing = false;
    }
}

const STRIP: &[char] = &[
    '(', ')', '[', ']', '{', '}', '<', '>',
    '*', '_', '~', '`', '#',
    '\u{00AB}', '\u{00BB}',
];

fn parse_words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .flat_map(|w| {
            w.split('\u{2014}') // em dash
                .flat_map(|p| p.split('\u{2013}')) // en dash
                .flat_map(|p| p.split("--"))
                .map(|p| {
                    let s: String = p.chars().filter(|c| !STRIP.contains(c)).collect();
                    let s = s.replace("\u{2026}", "").replace("...", "");
                    collapse_trailing_punct(s)
                })
                .filter(|s| !s.is_empty() && !s.chars().all(|c| c.is_ascii_punctuation()))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn collapse_trailing_punct(s: String) -> String {
    let bytes = s.as_bytes();
    if bytes.len() < 2 {
        return s;
    }
    let last = *bytes.last().unwrap();
    if !last.is_ascii_punctuation() {
        return s;
    }
    let mut i = bytes.len() - 1;
    while i > 0 && bytes[i - 1] == last {
        i -= 1;
    }
    if bytes.len() - i > 1 {
        format!("{}{}", &s[..i], last as char)
    } else {
        s
    }
}

impl App {
    fn focal(word: &str) -> usize {
        let n = word.chars().count();
        if n <= 1 { 0 } else { (n - 1) / 3 }
    }

    fn progress(&self) -> f32 {
        if self.words.len() <= 1 { return 0.0; }
        self.idx as f32 / (self.words.len() - 1) as f32
    }

    fn eta(&self) -> String {
        if self.words.is_empty() { return "--:--".into(); }
        let left = self.words.len().saturating_sub(self.idx);
        let s = (left as f64 * 60.0 / self.wpm as f64) as u64;
        format!("{}:{:02}", s / 60, s % 60)
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let t = self.theme;

        if self.playing && !self.words.is_empty() {
            let dt = std::time::Duration::from_secs_f64(60.0 / self.wpm as f64);
            if self.last_tick.elapsed() >= dt {
                self.last_tick = Instant::now();
                if self.idx + 1 < self.words.len() {
                    self.idx += 1;
                } else {
                    self.playing = false;
                }
            }
            ctx.request_repaint();
        }

        // -- bottom: text input (fixed 20% of window) --
        let panel_h = ctx.screen_rect().height() * 0.2;
        egui::TopBottomPanel::bottom("input")
            .exact_height(panel_h)
            .frame(egui::Frame::default().fill(t.bg).inner_margin(egui::Margin {
                left: 32,
                right: 32,
                top: 10,
                bottom: 10,
            }))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let r = ui.add(
                        egui::TextEdit::multiline(&mut self.text)
                            .desired_rows(4)
                            .desired_width(f32::INFINITY)
                            .frame(false)
                            .hint_text("Paste your text here..."),
                    );
                    if r.has_focus() {
                        self.focused = false;
                        self.playing = false;
                    }
                    if r.changed() {
                        self.reparse();
                    }
                });
            });

        // -- center: reader --
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(t.bg).inner_margin(egui::Margin {
                left: 32,
                right: 32,
                top: 0,
                bottom: 0,
            }))
            .show(ctx, |ui| {
                let (rect, resp) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::click());

                if resp.clicked() {
                    self.focused = true;
                }

                // keyboard
                if self.focused {
                    let mut toggle = false;
                    ctx.input(|i| {
                        if i.key_pressed(egui::Key::Space) && !self.words.is_empty() {
                            self.playing = !self.playing;
                            self.last_tick = Instant::now();
                            if self.playing && self.idx + 1 >= self.words.len() {
                                self.idx = 0;
                            }
                        }
                        if i.key_pressed(egui::Key::ArrowRight)
                            && self.idx + 1 < self.words.len()
                        {
                            self.idx += 1;
                        }
                        if i.key_pressed(egui::Key::ArrowLeft) && self.idx > 0 {
                            self.idx -= 1;
                        }
                        if i.key_pressed(egui::Key::ArrowUp) {
                            self.wpm = (self.wpm + 25).min(1500);
                        }
                        if i.key_pressed(egui::Key::ArrowDown) {
                            self.wpm = self.wpm.saturating_sub(25).max(50);
                        }
                        if i.key_pressed(egui::Key::Escape) {
                            self.focused = false;
                            self.playing = false;
                        }
                        if i.key_pressed(egui::Key::T) {
                            toggle = true;
                        }
                    });
                    if toggle {
                        self.toggle_theme(ctx);
                        return;
                    }
                }

                let p = ui.painter_at(rect);
                let cx = rect.center().x;
                let word_y = rect.top() + rect.height() * 0.45;

                // -- word --
                if let Some(word) = self.words.get(self.idx).cloned() {
                    let fi = App::focal(&word);
                    let font = egui::FontId::monospace(44.0);
                    let cw = p
                        .layout_no_wrap("W".into(), font.clone(), t.text_hi)
                        .rect
                        .width();
                    let x0 = cx - (fi as f32 + 0.5) * cw;

                    // guide ticks
                    p.line_segment(
                        [egui::pos2(cx, word_y - 32.0), egui::pos2(cx, word_y - 25.0)],
                        egui::Stroke::new(1.0, t.text_muted),
                    );
                    p.line_segment(
                        [egui::pos2(cx, word_y + 25.0), egui::pos2(cx, word_y + 32.0)],
                        egui::Stroke::new(1.0, t.text_muted),
                    );

                    for (i, ch) in word.chars().enumerate() {
                        let color = if i == fi { t.accent } else { t.text_hi };
                        p.text(
                            egui::pos2(x0 + (i as f32 + 0.5) * cw, word_y),
                            egui::Align2::CENTER_CENTER,
                            ch.to_string(),
                            font.clone(),
                            color,
                        );
                    }
                } else {
                    p.text(
                        egui::pos2(cx, word_y),
                        egui::Align2::CENTER_CENTER,
                        "Paste text below to start",
                        egui::FontId::proportional(16.0),
                        t.text_muted,
                    );
                }

                // -- progress bar --
                let bar_y = rect.bottom() - 30.0;
                let bar_w = rect.width();
                let bar_h = 2.0;
                let bar_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.left(), bar_y),
                    egui::vec2(bar_w, bar_h),
                );
                p.rect_filled(bar_rect, egui::CornerRadius::same(1), t.track);
                let fw = bar_w * self.progress();
                if fw > 0.0 {
                    p.rect_filled(
                        egui::Rect::from_min_size(bar_rect.min, egui::vec2(fw, bar_h)),
                        egui::CornerRadius::same(1),
                        t.text_dim,
                    );
                }

                // -- status --
                let sy = bar_y - 6.0;
                p.text(
                    egui::pos2(rect.left(), sy),
                    egui::Align2::LEFT_BOTTOM,
                    format!("{} wpm", self.wpm),
                    egui::FontId::proportional(14.0),
                    t.text_dim,
                );
                if !self.words.is_empty() {
                    p.text(
                        egui::pos2(cx, sy),
                        egui::Align2::CENTER_BOTTOM,
                        format!("{} / {}", self.idx + 1, self.words.len()),
                        egui::FontId::proportional(14.0),
                        t.text_muted,
                    );
                }
                p.text(
                    egui::pos2(rect.right(), sy),
                    egui::Align2::RIGHT_BOTTOM,
                    self.eta(),
                    egui::FontId::proportional(14.0),
                    t.text_dim,
                );

                // -- hints --
                let hint = if self.focused {
                    "[Space] play/pause   [Left/Right] word   [Up/Down] speed   [T] theme   [Esc] unfocus"
                } else {
                    "Click here to focus"
                };
                p.text(
                    egui::pos2(cx, rect.bottom() - 8.0),
                    egui::Align2::CENTER_BOTTOM,
                    hint,
                    egui::FontId::proportional(14.0),
                    t.text_muted,
                );
            });
    }
}
