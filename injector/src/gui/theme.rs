use egui::{
    Color32, Context, FontData, FontDefinitions, FontFamily, Frame, Margin, Rounding, Stroke, Vec2,
};

pub struct Palette;

impl Palette {
    pub fn bg() -> Color32 {
        Color32::from_rgba_unmultiplied(0x14, 0x15, 0x1a, 0xe8)
    }

    pub fn panel() -> Color32 {
        Color32::from_rgba_unmultiplied(0x1b, 0x1c, 0x24, 0xe8)
    }

    pub fn card() -> Color32 {
        Color32::from_rgba_unmultiplied(0x23, 0x25, 0x2f, 0xe8)
    }

    pub fn card_hover() -> Color32 {
        Color32::from_rgba_unmultiplied(0x2c, 0x2e, 0x3b, 0xf2)
    }

    pub const ACCENT: Color32 = Color32::from_rgb(0xa7, 0x79, 0xff);

    pub fn accent_soft() -> Color32 {
        Color32::from_rgba_unmultiplied(0x3b, 0x2d, 0x58, 0xf0)
    }

    pub const TEXT: Color32 = Color32::from_rgb(0xe6, 0xe7, 0xee);

    pub const TEXT_DIM: Color32 = Color32::from_rgb(0x8a, 0x8d, 0x9c);

    pub const OK: Color32 = Color32::from_rgb(0x4a, 0xd2, 0x95);

    pub const WARN: Color32 = Color32::from_rgb(0xe6, 0xb4, 0x50);

    pub const ERR: Color32 = Color32::from_rgb(0xe5, 0x6b, 0x6b);
}

pub fn apply(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "fingerpaint".to_owned(),
        FontData::from_static(include_bytes!("../../assets/fingerpaint.ttf")).into(),
    );
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "fingerpaint".to_owned());
    ctx.set_fonts(fonts);

    let mut style = (*ctx.style()).clone();
    let v = &mut style.visuals;

    v.dark_mode = true;
    v.panel_fill = Palette::bg();
    v.window_fill = Palette::bg();
    v.extreme_bg_color = Palette::panel();
    v.override_text_color = Some(Palette::TEXT);
    v.selection.bg_fill = Palette::ACCENT.linear_multiply(0.45);
    v.selection.stroke = Stroke::new(1.0_f32, Palette::ACCENT);
    v.hyperlink_color = Palette::ACCENT;

    let w = &mut v.widgets;
    w.noninteractive.rounding = Rounding::same(8.0);
    w.inactive.rounding = Rounding::same(8.0);
    w.hovered.rounding = Rounding::same(8.0);
    w.active.rounding = Rounding::same(8.0);
    w.inactive.bg_fill = Palette::card();
    w.inactive.weak_bg_fill = Palette::card();
    w.inactive.fg_stroke = Stroke::new(1.0_f32, Palette::TEXT);
    w.hovered.bg_fill = Palette::card_hover();
    w.hovered.weak_bg_fill = Palette::card_hover();
    w.hovered.fg_stroke = Stroke::new(1.0_f32, Palette::TEXT);
    w.active.bg_fill = Palette::ACCENT;
    w.active.weak_bg_fill = Palette::ACCENT;

    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(14.0, 8.0);

    ctx.set_style(style);
}

pub fn header_frame() -> Frame {
    Frame::none()
        .fill(Color32::from_rgba_unmultiplied(0x19, 0x17, 0x22, 0xe8))
        .inner_margin(Margin::symmetric(12.0, 10.0))
}

pub fn toolbar_frame() -> Frame {
    Frame::none()
        .fill(Color32::from_rgba_unmultiplied(0x19, 0x17, 0x22, 0xe8))
        .inner_margin(Margin::symmetric(12.0, 8.0))
}

pub fn footer_frame() -> Frame {
    Frame::none()
        .fill(Color32::from_rgba_unmultiplied(0x19, 0x17, 0x22, 0xf0))
        .inner_margin(Margin::symmetric(12.0, 10.0))
}
