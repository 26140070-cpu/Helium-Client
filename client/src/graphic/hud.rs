use crate::graphic::theme;
use crate::mapping::entity::{EntityRef, LivingEntityRef};
use crate::mapping::{entity::player::Player, MappedObject};
use crate::module::client::hud_editor::HudEditorModule;
use egui::{
    Align2, Color32, Context, FontId, Id, LayerId, Order, Painter, Rect, Rounding, Sense, Stroke,
    Vec2,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

const MARGIN: f32 = 10.0;
const GRID_SIZE: f32 = 16.0;

static EDITING: AtomicBool = AtomicBool::new(false);

pub fn editing() -> bool {
    EDITING.load(Ordering::Relaxed)
}

pub fn set_editing(enabled: bool) {
    EDITING.store(enabled, Ordering::Relaxed);
}

pub fn draw(ctx: &Context) {
    draw_widgets(ctx, false);
}

pub fn draw_editor(ctx: &Context) {
    if !editing() {
        return;
    }

    if HudEditorModule::enabled_setting("Grid", true) {
        draw_grid(ctx);
    }
    draw_widgets(ctx, true);
}

fn draw_grid(ctx: &Context) {
    let screen = ctx.screen_rect();
    let painter = ctx.layer_painter(LayerId::new(Order::Middle, Id::new("hud_editor_grid")));
    let color = Color32::from_white_alpha(24);
    let mut x = screen.left();
    while x <= screen.right() {
        painter.line_segment(
            [egui::pos2(x, screen.top()), egui::pos2(x, screen.bottom())],
            Stroke::new(0.5_f32, color),
        );
        x += GRID_SIZE;
    }
    let mut y = screen.top();
    while y <= screen.bottom() {
        painter.line_segment(
            [egui::pos2(screen.left(), y), egui::pos2(screen.right(), y)],
            Stroke::new(0.5_f32, color),
        );
        y += GRID_SIZE;
    }
}

fn draw_widgets(ctx: &Context, editor: bool) {
    let screen = ctx.screen_rect();

    if HudEditorModule::enabled_setting("Watermark", true) {
        let width = measured_width(ctx, "HeliumClient", 17.0 * HudEditorModule::scale()) + 28.0;
        show_widget(
            ctx,
            "watermark",
            [MARGIN, MARGIN],
            Vec2::new(width, 34.0 * HudEditorModule::scale()),
            editor,
            |painter, rect| {
                let accent = accent_color();
                paint_card(painter, rect);
                paint_accent(painter, rect, accent);
                let center = rect.left_center() + Vec2::new(12.0, 0.0);
                let after = painter.text(
                    center,
                    Align2::LEFT_CENTER,
                    "Helium",
                    FontId::proportional(17.0 * HudEditorModule::scale()),
                    theme::TEXT(),
                );
                painter.text(
                    egui::pos2(after.max.x, center.y),
                    Align2::LEFT_CENTER,
                    "Client",
                    FontId::proportional(17.0 * HudEditorModule::scale()),
                    accent,
                );
            },
        );
    }

    if HudEditorModule::enabled_setting("Arraylist", true) {
        let mut names: Vec<String> = crate::state::client()
            .modules
            .handles()
            .iter()
            .filter_map(|module| {
                let module = module.lock().ok()?;
                let data = module.get_module_data();
                data.enabled.then(|| data.name().to_string())
            })
            .collect();
        names.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
        let row_height = 20.0 * HudEditorModule::scale();
        let width = names
            .iter()
            .map(|name| measured_width(ctx, name, 13.0 * HudEditorModule::scale()))
            .fold(88.0, f32::max)
            + 20.0;
        let height = (names.len().max(1) as f32 * row_height) + 8.0;
        show_widget(
            ctx,
            "arraylist",
            [screen.right() - width - MARGIN, MARGIN],
            Vec2::new(width, height),
            editor,
            |painter, rect| {
                paint_card(painter, rect);
                for (index, name) in names.iter().enumerate() {
                    let row = Rect::from_min_size(
                        egui::pos2(
                            rect.left() + 9.0,
                            rect.top() + 4.0 + index as f32 * row_height,
                        ),
                        Vec2::new(rect.width() - 18.0, row_height),
                    );
                    painter.text(
                        row.left_center(),
                        Align2::LEFT_CENTER,
                        name,
                        FontId::proportional(13.0 * HudEditorModule::scale()),
                        theme::TEXT(),
                    );
                    painter.rect_filled(
                        Rect::from_min_size(
                            egui::pos2(rect.right() - 3.0, row.top()),
                            Vec2::new(2.0, row.height()),
                        ),
                        Rounding::ZERO,
                        accent_color(),
                    );
                }
            },
        );
    }

    if let Ok(Some(player)) = crate::state::minecraft().player() {
        if HudEditorModule::enabled_setting("Coordinates", true) {
            if let Ok(pos) = player.get_position() {
                draw_text_widget(
                    ctx,
                    "coordinates",
                    [MARGIN, screen.bottom() - 80.0],
                    format!("XYZ  {:.1}, {:.1}, {:.1}", pos.x(), pos.y(), pos.z()),
                    editor,
                );
            }
        }

        if HudEditorModule::enabled_setting("Speed", true) {
            if let Ok(delta) = player.get_delta_movement() {
                let speed = (delta.x() * delta.x() + delta.z() * delta.z()).sqrt() * 20.0;
                draw_text_widget(
                    ctx,
                    "speed",
                    [MARGIN, screen.bottom() - 44.0],
                    format!("Speed  {speed:.2} m/s"),
                    editor,
                );
            }
        }

        if HudEditorModule::enabled_setting("Direction", true) {
            if let Ok(yaw) = player.get_yaw() {
                let direction = match (((yaw.rem_euclid(360.0) + 45.0) / 90.0) as i32) % 4 {
                    0 => "South",
                    1 => "West",
                    2 => "North",
                    _ => "East",
                };
                draw_text_widget(
                    ctx,
                    "direction",
                    [screen.center().x - 55.0, screen.bottom() - 44.0],
                    format!("Facing  {direction}"),
                    editor,
                );
            }
        }

        if HudEditorModule::enabled_setting("Player Status", true) {
            if let (Ok(sprinting), Ok(on_ground)) = (player.is_sprinting(), player.on_ground()) {
                draw_text_widget(
                    ctx,
                    "player_status",
                    [screen.center().x + 45.0, screen.bottom() - 44.0],
                    format!(
                        "{}  ·  {}",
                        if sprinting { "Sprinting" } else { "Walking" },
                        if on_ground { "Ground" } else { "Air" }
                    ),
                    editor,
                );
            }
        }
    }

    if HudEditorModule::enabled_setting("FPS", true) {
        let fps = ctx.input(|input| (1.0 / input.stable_dt.max(0.001)).round() as u32);
        draw_text_widget(
            ctx,
            "fps",
            [screen.right() - 100.0, screen.bottom() - 44.0],
            format!("FPS  {fps}"),
            editor,
        );
    }

    if HudEditorModule::enabled_setting("Target HUD", true) {
        if let Some(target) = nearest_player_target() {
            draw_target_widget(ctx, target, editor);
        } else if editor {
            draw_target_widget(
                ctx,
                TargetInfo {
                    name: "Target HUD".to_string(),
                    health: 20.0,
                    max_health: 20.0,
                },
                true,
            );
        }
    }
}

#[derive(Clone)]
struct TargetInfo {
    name: String,
    health: f32,
    max_health: f32,
}

static TARGET_CACHE: LazyLock<Mutex<Option<(Instant, Option<TargetInfo>)>>> =
    LazyLock::new(|| Mutex::new(None));

fn nearest_player_target() -> Option<TargetInfo> {
    if let Ok(cache) = TARGET_CACHE.lock() {
        if let Some((refreshed, target)) = cache.as_ref() {
            if refreshed.elapsed().as_millis() < 250 {
                return target.clone();
            }
        }
    }

    let target = find_nearest_player_target();
    if let Ok(mut cache) = TARGET_CACHE.lock() {
        *cache = Some((Instant::now(), target.clone()));
    }
    target
}

fn find_nearest_player_target() -> Option<TargetInfo> {
    let minecraft = crate::state::minecraft();
    let player = minecraft.player().ok()??;
    let world = minecraft.world().ok()??;
    let own_id = player.id().ok()?;
    let origin = player.get_position().ok()?;
    let mut nearest: Option<(f64, TargetInfo)> = None;

    for entity in world.get_entities().ok()? {
        if entity.id().ok()? == own_id || !entity.instance_of::<Player>() {
            continue;
        }
        let Some(living) = entity.as_living() else {
            continue;
        };
        let (Ok(health), Ok(max_health), Ok(position), Ok(name)) = (
            living.get_health(),
            living.get_max_health(),
            entity.get_position(),
            entity.get_name().and_then(|name| name.get_string()),
        ) else {
            continue;
        };
        if health <= 0.0 {
            continue;
        }
        let dx = position.x() - origin.x();
        let dy = position.y() - origin.y();
        let dz = position.z() - origin.z();
        let distance = dx * dx + dy * dy + dz * dz;
        if nearest
            .as_ref()
            .is_none_or(|(current, _)| distance < *current)
        {
            nearest = Some((
                distance,
                TargetInfo {
                    name,
                    health,
                    max_health,
                },
            ));
        }
    }

    nearest.map(|(_, target)| target)
}

fn draw_target_widget(ctx: &Context, target: TargetInfo, editor: bool) {
    let width = measured_width(ctx, &target.name, 14.0).max(112.0) + 30.0;
    let rect_size = Vec2::new(width, 48.0);
    let screen = ctx.screen_rect();
    show_widget(
        ctx,
        "target",
        [screen.center().x - width / 2.0, screen.center().y + 45.0],
        rect_size,
        editor,
        |painter, rect| {
            paint_card(painter, rect);
            paint_accent(painter, rect, accent_color());
            painter.text(
                rect.left_top() + Vec2::new(12.0, 14.0),
                Align2::LEFT_CENTER,
                &target.name,
                FontId::proportional(14.0),
                theme::TEXT(),
            );
            let health_ratio = (target.health / target.max_health.max(1.0)).clamp(0.0, 1.0);
            let bar = Rect::from_min_size(
                egui::pos2(rect.left() + 12.0, rect.bottom() - 13.0),
                Vec2::new(rect.width() - 24.0, 5.0),
            );
            painter.rect_filled(bar, Rounding::same(2.0), Color32::from_gray(45));
            painter.rect_filled(
                Rect::from_min_size(bar.min, Vec2::new(bar.width() * health_ratio, bar.height())),
                Rounding::same(2.0),
                accent_color(),
            );
            painter.text(
                rect.right_top() + Vec2::new(-8.0, 14.0),
                Align2::RIGHT_CENTER,
                format!("{:.1}/{:.1}", target.health, target.max_health),
                FontId::proportional(10.0),
                theme::TEXT_DIM(),
            );
        },
    );
}

fn draw_text_widget(ctx: &Context, id: &str, default: [f32; 2], text: String, editor: bool) {
    let size = Vec2::new(measured_width(ctx, &text, 13.0) + 24.0, 30.0);
    show_widget(ctx, id, default, size, editor, |painter, rect| {
        paint_card(painter, rect);
        painter.text(
            rect.left_center() + Vec2::new(11.0, 0.0),
            Align2::LEFT_CENTER,
            text,
            FontId::proportional(13.0),
            theme::TEXT(),
        );
    });
}

fn show_widget(
    ctx: &Context,
    id: &str,
    default: [f32; 2],
    size: Vec2,
    editor: bool,
    paint: impl FnOnce(&Painter, Rect),
) {
    let screen = ctx.screen_rect();
    let stored = crate::config::hud_pos(id).unwrap_or(default);
    let area_id = Id::new("hud_widget").with(id);
    let area = egui::Area::new(area_id)
        .order(if editor {
            Order::Foreground
        } else {
            Order::Background
        })
        .fixed_pos(egui::pos2(stored[0], stored[1]))
        .show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(
                size,
                if editor {
                    Sense::drag()
                } else {
                    Sense::hover()
                },
            );
            paint(ui.painter(), rect);
            if editor {
                ui.painter().rect_stroke(
                    rect,
                    Rounding::same(4.0),
                    Stroke::new(
                        if response.hovered() { 1.5_f32 } else { 1.0_f32 },
                        accent_color(),
                    ),
                );
            }
            response
        });

    if !editor {
        return;
    }

    let response = area.inner;
    if response.dragged() {
        let mut next = egui::pos2(stored[0], stored[1]) + response.drag_delta();
        if HudEditorModule::enabled_setting("Grid", true) {
            next.x = (next.x / GRID_SIZE).round() * GRID_SIZE;
            next.y = (next.y / GRID_SIZE).round() * GRID_SIZE;
        }
        next.x = next
            .x
            .clamp(screen.left(), (screen.right() - size.x).max(screen.left()));
        next.y = next
            .y
            .clamp(screen.top(), (screen.bottom() - size.y).max(screen.top()));
        crate::config::set_hud_pos(id, [next.x, next.y]);
        ctx.request_repaint();
    }
    if response.drag_stopped() {
        crate::config::save();
    }
}

fn measured_width(ctx: &Context, text: &str, size: f32) -> f32 {
    ctx.fonts(|fonts| {
        fonts
            .layout_no_wrap(text.to_string(), FontId::proportional(size), theme::TEXT())
            .size()
            .x
    })
}

fn paint_card(painter: &Painter, rect: Rect) {
    painter.rect_filled(
        rect,
        Rounding::same(theme::RADIUS()),
        Color32::from_black_alpha(HudEditorModule::array_opacity()),
    );
    painter.rect_stroke(
        rect,
        Rounding::same(theme::RADIUS()),
        Stroke::new(1.0_f32, theme::BORDER()),
    );
}

fn paint_accent(painter: &Painter, rect: Rect, accent: Color32) {
    painter.rect_filled(
        Rect::from_min_size(rect.min, Vec2::new(3.0, rect.height())),
        Rounding {
            nw: theme::RADIUS(),
            sw: theme::RADIUS(),
            ne: 0.0,
            se: 0.0,
        },
        accent,
    );
}

fn accent_color() -> Color32 {
    if HudEditorModule::rainbow() {
        let t = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f32())
            .unwrap_or(0.0)
            * 0.35)
            .fract();
        let (r, g, b) = hsv_to_rgb(t, 0.85, 1.0);
        return Color32::from_rgb(r, g, b);
    }
    let a = HudEditorModule::accent();
    Color32::from_rgba_unmultiplied(
        (a[0] * 255.0) as u8,
        (a[1] * 255.0) as u8,
        (a[2] * 255.0) as u8,
        (a[3] * 255.0) as u8,
    )
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match (i as i32) % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}
