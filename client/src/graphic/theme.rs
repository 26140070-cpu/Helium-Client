use egui::{Color32, Context, Margin, Rounding, Stroke, Style, Vec2, Visuals};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LazyLock, Mutex};

#[derive(Clone, Copy, Debug)]
pub struct ThemeLive {
    pub accent: Color32,
    pub accent_dim: Color32,
    pub base: Color32,
    pub surface: Color32,
    pub surface_hover: Color32,
    pub elevated: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub text_muted: Color32,
    pub warn: Color32,
    pub danger: Color32,
    pub radius: f32,
    pub radius_inner: f32,
}

impl Default for ThemeLive {
    fn default() -> Self {
        Self {
            accent: Color32::from_rgb(38, 198, 156),
            accent_dim: Color32::from_rgb(26, 120, 100),
            base: Color32::from_rgb(17, 18, 22),
            surface: Color32::from_rgb(24, 25, 31),
            surface_hover: Color32::from_rgb(33, 35, 43),
            elevated: Color32::from_rgb(29, 31, 38),
            border: Color32::from_rgb(42, 44, 54),
            text: Color32::from_rgb(236, 237, 242),
            text_dim: Color32::from_rgb(150, 152, 162),
            text_muted: Color32::from_rgb(92, 94, 104),
            warn: Color32::from_rgb(240, 170, 60),
            danger: Color32::from_rgb(226, 74, 74),
            radius: 7.0,
            radius_inner: 4.0,
        }
    }
}

static LIVE: Mutex<ThemeLive> = Mutex::new(ThemeLive {
    accent: Color32::from_rgb(38, 198, 156),
    accent_dim: Color32::from_rgb(26, 120, 100),
    base: Color32::from_rgb(17, 18, 22),
    surface: Color32::from_rgb(24, 25, 31),
    surface_hover: Color32::from_rgb(33, 35, 43),
    elevated: Color32::from_rgb(29, 31, 38),
    border: Color32::from_rgb(42, 44, 54),
    text: Color32::from_rgb(236, 237, 242),
    text_dim: Color32::from_rgb(150, 152, 162),
    text_muted: Color32::from_rgb(92, 94, 104),
    warn: Color32::from_rgb(240, 170, 60),
    danger: Color32::from_rgb(226, 74, 74),
    radius: 7.0,
    radius_inner: 4.0,
});

static SELECTED_PRESET: AtomicUsize = AtomicUsize::new(0);
static CUSTOM_THEMES: LazyLock<Mutex<Vec<CustomTheme>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static SELECTED_CUSTOM_THEME: Mutex<Option<String>> = Mutex::new(None);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CustomTheme {
    pub name: String,
    pub accent: [u8; 3],
    pub accent_dim: [u8; 3],
    pub base: [u8; 3],
    pub surface: [u8; 3],
    pub surface_hover: [u8; 3],
    pub elevated: [u8; 3],
    pub border: [u8; 3],
    pub text: [u8; 3],
    pub text_dim: [u8; 3],
    pub text_muted: [u8; 3],
    pub warn: [u8; 3],
    pub danger: [u8; 3],
    pub radius: f32,
    pub radius_inner: f32,
}

impl CustomTheme {
    fn from_live(name: String, theme: ThemeLive) -> Self {
        let rgb = |color: Color32| [color.r(), color.g(), color.b()];
        Self {
            name,
            accent: rgb(theme.accent),
            accent_dim: rgb(theme.accent_dim),
            base: rgb(theme.base),
            surface: rgb(theme.surface),
            surface_hover: rgb(theme.surface_hover),
            elevated: rgb(theme.elevated),
            border: rgb(theme.border),
            text: rgb(theme.text),
            text_dim: rgb(theme.text_dim),
            text_muted: rgb(theme.text_muted),
            warn: rgb(theme.warn),
            danger: rgb(theme.danger),
            radius: theme.radius,
            radius_inner: theme.radius_inner,
        }
    }

    fn to_live(&self) -> ThemeLive {
        ThemeLive {
            accent: rgb(self.accent),
            accent_dim: rgb(self.accent_dim),
            base: rgb(self.base),
            surface: rgb(self.surface),
            surface_hover: rgb(self.surface_hover),
            elevated: rgb(self.elevated),
            border: rgb(self.border),
            text: rgb(self.text),
            text_dim: rgb(self.text_dim),
            text_muted: rgb(self.text_muted),
            warn: rgb(self.warn),
            danger: rgb(self.danger),
            radius: self.radius.clamp(0.0, 16.0),
            radius_inner: self.radius_inner.clamp(0.0, 12.0),
        }
    }
}

pub fn live() -> ThemeLive {
    LIVE.lock().map(|g| *g).unwrap_or_default()
}

pub fn set_live(t: ThemeLive) {
    if let Ok(mut g) = LIVE.lock() {
        *g = t;
    }
    if let Ok(mut selected) = SELECTED_CUSTOM_THEME.lock() {
        *selected = None;
    }
}

pub fn ACCENT() -> Color32 {
    live().accent
}
pub fn ACCENT_DIM() -> Color32 {
    live().accent_dim
}
pub fn BASE() -> Color32 {
    live().base
}
pub fn SURFACE() -> Color32 {
    live().surface
}
pub fn SURFACE_HOVER() -> Color32 {
    live().surface_hover
}
pub fn ELEVATED() -> Color32 {
    live().elevated
}
pub fn BORDER() -> Color32 {
    live().border
}
pub fn TEXT() -> Color32 {
    live().text
}
pub fn TEXT_DIM() -> Color32 {
    live().text_dim
}
pub fn TEXT_MUTED() -> Color32 {
    live().text_muted
}
pub fn WARN() -> Color32 {
    live().warn
}
pub fn DANGER() -> Color32 {
    live().danger
}
pub fn RADIUS() -> f32 {
    live().radius
}
pub fn RADIUS_INNER() -> f32 {
    live().radius_inner
}
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub name: &'static str,
    pub accent: [u8; 3],
    pub base: [u8; 3],
    pub surface: [u8; 3],
}

pub const PRESETS: &[Palette] = &[
    Palette {
        name: "Helium",
        accent: [38, 198, 156],
        base: [17, 18, 22],
        surface: [24, 25, 31],
    },
    Palette {
        name: "Midnight",
        accent: [88, 140, 255],
        base: [10, 12, 20],
        surface: [18, 22, 34],
    },
    Palette {
        name: "Crimson",
        accent: [232, 72, 85],
        base: [18, 12, 14],
        surface: [30, 18, 20],
    },
    Palette {
        name: "Ocean",
        accent: [0, 184, 212],
        base: [8, 16, 24],
        surface: [14, 28, 40],
    },
    Palette {
        name: "Violet",
        accent: [171, 110, 255],
        base: [16, 12, 24],
        surface: [28, 22, 40],
    },
    Palette {
        name: "Amber",
        accent: [255, 179, 0],
        base: [18, 16, 10],
        surface: [32, 28, 16],
    },
    Palette {
        name: "Mono",
        accent: [220, 220, 220],
        base: [12, 12, 12],
        surface: [22, 22, 22],
    },
    Palette {
        name: "Rose",
        accent: [255, 105, 180],
        base: [18, 12, 16],
        surface: [30, 20, 26],
    },
    Palette {
        name: "Forest",
        accent: [112, 190, 95],
        base: [12, 19, 14],
        surface: [22, 34, 25],
    },
    Palette {
        name: "Sky",
        accent: [105, 190, 255],
        base: [12, 19, 29],
        surface: [22, 34, 49],
    },
    Palette {
        name: "Lavender",
        accent: [190, 150, 255],
        base: [19, 14, 29],
        surface: [34, 26, 49],
    },
    Palette {
        name: "Mint",
        accent: [105, 235, 185],
        base: [11, 21, 19],
        surface: [20, 38, 34],
    },
    Palette {
        name: "Lava",
        accent: [255, 112, 65],
        base: [25, 13, 10],
        surface: [44, 23, 17],
    },
    Palette {
        name: "Sakura",
        accent: [255, 145, 190],
        base: [25, 14, 21],
        surface: [43, 25, 36],
    },
    Palette {
        name: "Arctic",
        accent: [130, 225, 235],
        base: [10, 20, 25],
        surface: [19, 36, 44],
    },
    Palette {
        name: "Gold",
        accent: [255, 205, 80],
        base: [24, 19, 10],
        surface: [42, 34, 18],
    },
    Palette {
        name: "Amethyst",
        accent: [145, 90, 230],
        base: [17, 12, 25],
        surface: [30, 21, 43],
    },
    Palette {
        name: "Cyber",
        accent: [245, 55, 220],
        base: [17, 10, 24],
        surface: [31, 18, 44],
    },
    Palette {
        name: "Lime",
        accent: [190, 235, 55],
        base: [18, 21, 10],
        surface: [33, 39, 18],
    },
    Palette {
        name: "Cobalt",
        accent: [55, 115, 245],
        base: [10, 14, 25],
        surface: [18, 25, 44],
    },
    Palette {
        name: "Copper",
        accent: [215, 130, 75],
        base: [22, 15, 12],
        surface: [39, 27, 21],
    },
    Palette {
        name: "Aqua",
        accent: [40, 225, 210],
        base: [9, 21, 22],
        surface: [17, 38, 39],
    },
    Palette {
        name: "Plum",
        accent: [200, 90, 160],
        base: [22, 12, 21],
        surface: [39, 21, 37],
    },
    Palette {
        name: "Ember",
        accent: [255, 90, 45],
        base: [25, 13, 10],
        surface: [44, 22, 17],
    },
    Palette {
        name: "Glacier",
        accent: [125, 205, 255],
        base: [11, 17, 25],
        surface: [20, 31, 44],
    },
    Palette {
        name: "Toxic",
        accent: [125, 235, 45],
        base: [13, 20, 10],
        surface: [23, 37, 17],
    },
    Palette {
        name: "Royal",
        accent: [245, 190, 55],
        base: [17, 14, 25],
        surface: [30, 25, 43],
    },
    Palette {
        name: "Neon Blue",
        accent: [35, 220, 255],
        base: [9, 16, 25],
        surface: [16, 29, 44],
    },
    Palette {
        name: "Coral",
        accent: [255, 125, 110],
        base: [25, 15, 15],
        surface: [44, 27, 26],
    },
    Palette {
        name: "Sapphire",
        accent: [75, 130, 220],
        base: [11, 15, 23],
        surface: [20, 27, 41],
    },
    Palette {
        name: "Peach",
        accent: [255, 165, 115],
        base: [25, 17, 14],
        surface: [44, 31, 25],
    },
    Palette {
        name: "Graphite",
        accent: [165, 175, 190],
        base: [13, 15, 18],
        surface: [25, 28, 34],
    },
];

fn rgb(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

fn dim_rgb(c: [u8; 3], factor: f32) -> Color32 {
    Color32::from_rgb(
        (c[0] as f32 * factor) as u8,
        (c[1] as f32 * factor) as u8,
        (c[2] as f32 * factor) as u8,
    )
}

pub fn selected_preset() -> usize {
    SELECTED_PRESET.load(Ordering::Relaxed)
}

pub fn set_preset(index: usize) {
    let index = index.min(PRESETS.len().saturating_sub(1));
    SELECTED_PRESET.store(index, Ordering::Relaxed);
    if let Ok(mut selected) = SELECTED_CUSTOM_THEME.lock() {
        *selected = None;
    }
    let preset = PRESETS[index];
    let accent = rgb(preset.accent);
    let surface = rgb(preset.surface);
    set_live(ThemeLive {
        accent,
        accent_dim: dim_rgb(preset.accent, 0.55),
        base: rgb(preset.base),
        surface,
        surface_hover: Color32::from_rgb(
            preset.surface[0].saturating_add(10),
            preset.surface[1].saturating_add(10),
            preset.surface[2].saturating_add(12),
        ),
        elevated: Color32::from_rgb(
            preset.surface[0].saturating_add(6),
            preset.surface[1].saturating_add(6),
            preset.surface[2].saturating_add(8),
        ),
        border: Color32::from_rgb(
            preset.surface[0].saturating_add(18),
            preset.surface[1].saturating_add(18),
            preset.surface[2].saturating_add(20),
        ),
        text: Color32::from_rgb(236, 237, 242),
        text_dim: Color32::from_rgb(150, 152, 162),
        text_muted: Color32::from_rgb(92, 94, 104),
        warn: Color32::from_rgb(240, 170, 60),
        danger: Color32::from_rgb(226, 74, 74),
        radius: 7.0,
        radius_inner: 4.0,
    });
}

pub fn custom_themes() -> Vec<CustomTheme> {
    CUSTOM_THEMES
        .lock()
        .map(|themes| themes.clone())
        .unwrap_or_default()
}

pub fn set_custom_themes(themes: Vec<CustomTheme>) {
    if let Ok(mut current) = CUSTOM_THEMES.lock() {
        *current = themes
            .into_iter()
            .filter(|theme| valid_theme_name(&theme.name))
            .take(32)
            .collect();
    }
}

pub fn selected_custom_theme() -> Option<String> {
    SELECTED_CUSTOM_THEME
        .lock()
        .ok()
        .and_then(|selected| selected.clone())
}

pub fn selected_theme_name() -> String {
    selected_custom_theme().unwrap_or_else(|| PRESETS[selected_preset()].name.to_string())
}

pub fn apply_custom_theme(name: &str) -> bool {
    let Some(theme) = custom_themes()
        .into_iter()
        .find(|theme| theme.name.eq_ignore_ascii_case(name))
    else {
        return false;
    };
    set_live(theme.to_live());
    if let Ok(mut selected) = SELECTED_CUSTOM_THEME.lock() {
        *selected = Some(theme.name);
    }
    true
}

pub fn save_custom_theme(name: &str) -> Result<(), &'static str> {
    let name = name.trim();
    if !valid_theme_name(name) {
        return Err("Theme name must be 1-32 characters and contain no control characters.");
    }
    let theme = CustomTheme::from_live(name.to_string(), live());
    let mut themes = CUSTOM_THEMES
        .lock()
        .map_err(|_| "Custom theme storage is unavailable.")?;
    if let Some(existing) = themes
        .iter_mut()
        .find(|existing| existing.name.eq_ignore_ascii_case(name))
    {
        *existing = theme;
    } else if themes.len() < 32 {
        themes.push(theme);
    } else {
        return Err("You can save up to 32 custom themes.");
    }
    drop(themes);
    let mut selected = SELECTED_CUSTOM_THEME
        .lock()
        .map_err(|_| "Selected theme state is unavailable.")?;
    *selected = Some(name.to_string());
    Ok(())
}

fn valid_theme_name(name: &str) -> bool {
    !name.trim().is_empty() && name.trim().len() <= 32 && !name.chars().any(char::is_control)
}

pub fn apply(ctx: &Context) {
    let t = live();

    let mut style = Style::default();
    let mut v = Visuals::dark();

    v.window_fill = t.base;
    v.panel_fill = t.base;
    v.window_stroke = Stroke::new(1.0_f32, t.border);
    v.window_rounding = Rounding::same(t.radius);
    v.window_shadow = egui::epaint::Shadow {
        offset: Vec2::new(0.0, 6.0),
        blur: 24.0,
        spread: 0.0,
        color: Color32::from_black_alpha(140),
    };
    v.popup_shadow = v.window_shadow;

    v.selection.bg_fill = t.accent;
    v.selection.stroke = Stroke::NONE;
    v.hyperlink_color = t.accent;

    let w = &mut v.widgets;

    w.noninteractive.bg_fill = t.surface;
    w.noninteractive.bg_stroke = Stroke::new(1.0_f32, t.border);
    w.noninteractive.fg_stroke = Stroke::new(1.0_f32, t.text_dim);

    w.inactive.bg_fill = t.surface;
    w.inactive.weak_bg_fill = t.surface;
    w.inactive.bg_stroke = Stroke::new(1.0_f32, t.border);
    w.inactive.fg_stroke = Stroke::new(1.0_f32, t.text_dim);
    w.inactive.rounding = Rounding::same(t.radius_inner);
    w.inactive.expansion = 0.0;

    w.hovered.bg_fill = t.surface_hover;
    w.hovered.weak_bg_fill = t.surface_hover;
    w.hovered.bg_stroke = Stroke::new(1.0_f32, t.border);
    w.hovered.fg_stroke = Stroke::new(1.0_f32, t.text);
    w.hovered.rounding = Rounding::same(t.radius_inner);
    w.hovered.expansion = 1.0;

    w.active.bg_fill = t.accent;
    w.active.weak_bg_fill = t.accent;
    w.active.bg_stroke = Stroke::NONE;
    w.active.fg_stroke = Stroke::new(1.0_f32, Color32::BLACK);
    w.active.rounding = Rounding::same(t.radius_inner);
    w.active.expansion = -1.0;

    w.open.bg_fill = t.surface_hover;
    w.open.bg_stroke = Stroke::new(1.0_f32, t.border);
    w.open.fg_stroke = Stroke::new(1.0_f32, t.text);

    style.visuals = v;
    style.spacing.item_spacing = Vec2::new(6.0, 6.0);
    style.spacing.window_margin = Margin::same(0.0);
    style.spacing.button_padding = Vec2::new(8.0, 4.0);
    style.spacing.interact_size.y = 16.0;
    style.animation_time = 0.18;

    ctx.set_style(style);
}

pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(
        mix(a.r(), b.r()),
        mix(a.g(), b.g()),
        mix(a.b(), b.b()),
        mix(a.a(), b.a()),
    )
}

pub fn with_alpha(color: Color32, factor: f32) -> Color32 {
    let a = (color.a() as f32 * factor.clamp(0.0, 1.0)).round() as u8;
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a)
}
