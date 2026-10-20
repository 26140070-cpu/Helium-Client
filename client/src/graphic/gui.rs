use crate::graphic::anim::{self, Easing};
use crate::graphic::input::GUI_OPEN;
use crate::graphic::theme;
use crate::graphic::{esp, hud, menu, notification, terminal};
use egui::{Context, Id};
use std::sync::atomic::Ordering;

pub fn render_all(ctx: &Context) {
    theme::apply(ctx);
    esp::draw(ctx);

    let open = GUI_OPEN.load(Ordering::Relaxed);
    let progress = anim::toggle(ctx, Id::new("clickgui_open"), open, 0.26, Easing::OutBack);
    let fade = anim::toggle(ctx, Id::new("clickgui_fade"), open, 0.20, Easing::OutCubic);
    if open && hud::editing() {
        hud::draw_editor(ctx);
    } else {
        hud::draw(ctx);
    }
    if fade > 0.0 {
        let screen = ctx.screen_rect();
        let dim = (fade * 0.45).clamp(0.0, 0.45);
        ctx.layer_painter(egui::LayerId::new(
            egui::Order::Background,
            Id::new("gui_dim"),
        ))
        .rect_filled(
            screen,
            0.0,
            egui::Color32::from_black_alpha((dim * 255.0) as u8),
        );
        menu::draw(ctx, progress.clamp(0.0, 1.0));
    }

    let term_p = terminal::open_progress(ctx);
    if term_p > 0.0 {
        terminal::draw(ctx, term_p);
    }

    notification::draw(ctx);
    anim::gc(ctx);
}
