use crate::graphic::anim::{self, Easing};
use crate::graphic::theme;
use crate::log_buffer;
use crate::module::client::terminal::TerminalModule;
use crate::module::ModuleId;
use crate::state::client;
use egui::{Align2, Color32, Context, FontId, Id, RichText, Sense, Stroke};
use std::sync::Mutex;
use std::time::{Duration, Instant};

static COMMAND_INPUT: Mutex<String> = Mutex::new(String::new());
static VISUAL_CLOCK: Mutex<Option<(crate::shell::TerminalVisual, Instant)>> = Mutex::new(None);

pub fn draw(ctx: &Context, progress: f32) {
    if progress <= 0.0 {
        return;
    }

    let screen = ctx.screen_rect();
    let edge_inset = 8.0_f32.min(screen.width() / 4.0).min(screen.height() / 4.0);
    let panel_width = (screen.width() - edge_inset * 2.0).max(0.0);
    let visual = crate::shell::terminal_visual();
    let top = TerminalModule::is_top();
    let max_h = screen.height() * if visual.is_some() { 0.76 } else { 0.42 };
    let h = max_h * progress;
    let bottom_y = screen.max.y - edge_inset - h;
    let top_y = screen.min.y + edge_inset - max_h + h;
    let dock_progress = anim::toggle(
        ctx,
        Id::new("terminal_dock_position"),
        top,
        0.42,
        Easing::InOut,
    );
    let y = bottom_y + (top_y - bottom_y) * dock_progress;

    egui::Area::new(Id::new("helium_terminal"))
        .fixed_pos(egui::pos2(screen.min.x + edge_inset, y))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.set_opacity(progress);
            egui::Frame::none()
                .fill(Color32::from_rgba_unmultiplied(10, 12, 16, 230))
                .stroke(Stroke::new(1.0_f32, theme::BORDER()))
                .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                .show(ui, |ui| {
                    ui.set_min_width(panel_width);
                    ui.set_max_width(panel_width);
                    ui.set_min_height(h.max(40.0) - 16.0);

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Helium Terminal")
                                .color(theme::ACCENT())
                                .strong()
                                .size(14.0),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let clear = ui.add(
                                egui::Button::new(RichText::new("Clear").size(12.0))
                                    .fill(theme::SURFACE())
                                    .stroke(Stroke::new(1.0_f32, theme::BORDER()))
                                    .rounding(4.0),
                            );
                            if clear.clicked() {
                                log_buffer::clear();
                                crate::shell::clear_log_filter();
                            }
                        });
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(if visual.is_some() {
                                "  ·  visual running — type stop to return"
                            } else {
                                "  ·  game / client logs"
                            })
                            .color(theme::TEXT_DIM())
                            .size(12.0),
                        );
                        let filter = crate::shell::log_filter();
                        if !filter.is_empty() {
                            ui.label(
                                RichText::new(format!("filter: {filter}"))
                                    .color(theme::TEXT_DIM())
                                    .size(11.0),
                            );
                        }
                    });
                    ui.add_space(4.0);
                    ui.separator();

                    let font = FontId::monospace(12.0);
                    let lines = log_buffer::snapshot();
                    let filter = crate::shell::log_filter().to_lowercase();
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .max_height(h.max(40.0) - 84.0)
                        .show(ui, |ui| {
                            ui.set_min_width((panel_width - 24.0).max(0.0));
                            if let Some(visual) = visual {
                                draw_visual(ui, visual, h.max(40.0) - 100.0);
                            } else {
                                for line in lines.iter().rev().take(300).rev() {
                                    if !filter.is_empty()
                                        && !line.message.to_lowercase().contains(&filter)
                                        && !line.level.to_lowercase().contains(&filter)
                                    {
                                        continue;
                                    }
                                    let color = match line.level.as_str() {
                                        "ERROR" => theme::DANGER(),
                                        "WARN" => theme::WARN(),
                                        "INFO" => theme::TEXT(),
                                        "DEBUG" => theme::TEXT_DIM(),
                                        _ => theme::TEXT_DIM(),
                                    };
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(format!("[{}]", line.level))
                                                .font(font.clone())
                                                .color(theme::ACCENT()),
                                        );
                                        ui.label(
                                            RichText::new(&line.message)
                                                .font(font.clone())
                                                .color(color),
                                        );
                                    });
                                }
                                if lines.is_empty() {
                                    ui.label(
                                        RichText::new("No logs yet…")
                                            .color(theme::TEXT_DIM())
                                            .italics(),
                                    );
                                } else if !filter.is_empty()
                                    && !lines.iter().any(|line| {
                                        line.message.to_lowercase().contains(&filter)
                                            || line.level.to_lowercase().contains(&filter)
                                    })
                                {
                                    ui.label(
                                        RichText::new("No log messages match this filter.")
                                            .color(theme::TEXT_DIM())
                                            .italics(),
                                    );
                                }
                            }
                        });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(crate::shell::prompt())
                                .font(FontId::monospace(12.0))
                                .color(theme::ACCENT())
                                .strong(),
                        );
                        let mut command = COMMAND_INPUT
                            .lock()
                            .map(|input| input.clone())
                            .unwrap_or_default();
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut command)
                                .font(FontId::monospace(12.0))
                                .hint_text("help | list | toggle killaura | search text")
                                .desired_width(ui.available_width()),
                        );
                        let submit = response.lost_focus()
                            && ui.input(|input| input.key_pressed(egui::Key::Enter));
                        if let Ok(mut input) = COMMAND_INPUT.lock() {
                            *input = command.clone();
                        }
                        if submit {
                            if let Ok(mut input) = COMMAND_INPUT.lock() {
                                *input = String::new();
                            }
                            if command.len() > 4096 {
                                log_buffer::push(
                                    "CMD",
                                    format!("> [large command omitted: {} bytes]", command.len()),
                                );
                            } else {
                                log_buffer::push("CMD", format!("> {command}"));
                            }
                            crate::shell::execute(&command);
                            response.request_focus();
                        }
                    });
                });
        });
}

fn draw_visual(ui: &mut egui::Ui, visual: crate::shell::TerminalVisual, height: f32) {
    let elapsed = match VISUAL_CLOCK.lock() {
        Ok(mut clock) => {
            let entry = clock.get_or_insert_with(|| (visual, Instant::now()));
            if entry.0 != visual {
                *entry = (visual, Instant::now());
            }
            entry.1.elapsed().as_secs_f32()
        }
        Err(_) => 0.0,
    };
    let size = egui::vec2(ui.available_width(), height.max(120.0));
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);

    match visual {
        crate::shell::TerminalVisual::Matrix => {
            ui.ctx().request_repaint_after(Duration::from_millis(65));
            let cell_w = 9.0;
            let cell_h = 14.0;
            let columns = (rect.width() / cell_w).floor() as usize;
            let rows = (rect.height() / cell_h).floor() as usize;
            let glyphs = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ#$%&*+-<>?";
            let tick = (elapsed * 12.0) as u32;
            for column in 0..columns {
                let column_seed = matrix_hash(column as u32);
                let streams = 1 + (column_seed % 3 == 0) as u32;
                for stream in 0..streams {
                    let seed = matrix_hash(column_seed.wrapping_add(stream * 7919));
                    let trail = 8.0 + (seed % 1500) as f32 / 100.0;
                    let speed = 3.0 + ((seed >> 8) % 70) as f32 / 10.0;
                    let cycle = rows as f32 + trail;
                    let offset = ((seed >> 16) as f32 / u16::MAX as f32) * cycle;
                    let head = (elapsed * speed + offset) % cycle - trail;
                    for row in 0..rows {
                        let distance = head - row as f32;
                        if !(0.0..trail).contains(&distance) {
                            continue;
                        }
                        let flicker =
                            matrix_hash(seed ^ (row as u32).wrapping_mul(31337) ^ tick / 3);
                        let glyph = glyphs[(flicker as usize) % glyphs.len()];
                        let color = if distance < 1.0 {
                            Color32::from_rgb(210, 255, 220)
                        } else {
                            let fade = 1.0 - distance / trail;
                            let intensity = (24.0 + fade * 195.0) as u8;
                            let blue = (12.0 + fade * 48.0) as u8;
                            Color32::from_rgb(18, intensity, blue)
                        };
                        painter.text(
                            egui::pos2(
                                rect.left() + column as f32 * cell_w,
                                rect.top() + row as f32 * cell_h,
                            ),
                            Align2::LEFT_TOP,
                            char::from(glyph),
                            FontId::monospace(12.0),
                            color,
                        );
                    }
                }
            }
        }
        crate::shell::TerminalVisual::Bonsai => {
            ui.ctx().request_repaint_after(Duration::from_millis(250));
            const TREE: &[&str] = &[
                "                 _",
                "            _.-'` `'-._",
                "         .-'  _   _    '-.",
                "       .'    (_) (_)      '.",
                "      /   _.-.  /\\  .-._     \\",
                "     ;   /   / /  \\ \\   \\     ;",
                "     |  ;   /_/    \\_\\   ;    |",
                "     ;  |      _||_      |    ;",
                "      \\  '.   / || \\   .'   /",
                "       '.  '-.__||__.-'   .'",
                "         '-._   ||   _.-'",
                "              `-||-'",
                "                ||",
                "             ___||___",
                "           .'   ||   '.",
                "          /_____| |_____\\",
            ];
            let line_h = 17.0;
            let total_h = TREE.len() as f32 * line_h;
            let top = rect.center().y - total_h / 2.0;
            let font = FontId::monospace(15.0);
            for (index, line) in TREE.iter().enumerate() {
                let color = if index >= 11 {
                    Color32::from_rgb(174, 125, 75)
                } else {
                    Color32::from_rgb(85, 220, 125)
                };
                painter.text(
                    egui::pos2(rect.center().x, top + index as f32 * line_h),
                    Align2::CENTER_TOP,
                    *line,
                    font.clone(),
                    color,
                );
            }
            for index in 0..9 {
                let phase = elapsed * 1.4 + index as f32 * 2.17;
                let x = rect.center().x + phase.sin() * (rect.width() * 0.34);
                let y = rect.center().y + (phase * 0.7).cos() * (total_h * 0.6);
                painter.text(
                    egui::pos2(x, y),
                    Align2::CENTER_CENTER,
                    if index % 3 == 0 { "*" } else { "+" },
                    FontId::monospace(13.0),
                    Color32::from_rgb(220, 205, 115),
                );
            }
        }
    }
}

fn matrix_hash(mut value: u32) -> u32 {
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^ (value >> 16)
}

pub fn open_progress(ctx: &Context) -> f32 {
    let open = client()
        .modules
        .get(ModuleId::Terminal)
        .and_then(|arc| arc.lock().ok().map(|m| m.get_module_data().enabled))
        .unwrap_or(false);
    anim::toggle(ctx, Id::new("terminal_open"), open, 0.22, Easing::Out)
}
