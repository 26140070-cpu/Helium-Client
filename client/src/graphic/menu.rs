use crate::graphic::anim::{self, Easing, SpringCfg};
use crate::graphic::hud;
use crate::graphic::input::LAST_KEY_PRESSED;
use crate::graphic::notification::{Notification, NotificationType};
use crate::graphic::theme;
use crate::module::client::terminal::TerminalModule;
use crate::module::{KeyboardKey, ModuleCategory, ModuleData, ModuleId, ModuleSetting, ModuleType};
use egui::{
    Align, Align2, Button, Color32, Context, FontId, Id, LayerId, Layout, Margin, Order, Painter,
    Pos2, Rect, RichText, Rounding, Sense, Shape, Stroke, Ui, Vec2,
};
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

const PANEL_W: f32 = 168.0;

const GAP: f32 = 14.0;

const TITLE_H: f32 = 30.0;

const ROW_H: f32 = 24.0;

const ROW_STRIDE: f32 = 320.0;

const ORIGIN_X: f32 = 40.0;
const ORIGIN_Y: f32 = 104.0;

static PRESET_WINDOW_OPEN: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static THEME_EDITOR_OPEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static KEYBINDS_WINDOW_OPEN: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static CLIENT_SETTINGS_OPEN: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static PROFILES_WINDOW_OPEN: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static CUSTOM_THEME_NAME: Mutex<String> = Mutex::new(String::new());

type ModuleArc = Arc<Mutex<ModuleType>>;

type ModuleMap = HashMap<ModuleId, ModuleArc>;

pub fn draw(ctx: &Context, progress: f32) {
    draw_backdrop(ctx, progress);

    let registry = crate::state::client().modules.by_id();

    let mut entries: Vec<(ModuleId, ModuleCategory)> = registry
        .iter()
        .map(|(id, arc)| {
            let module = arc.lock().unwrap();
            (*id, module.get_module_data().category)
        })
        .collect();
    entries.sort_by_key(|(id, _)| id.display_name());

    draw_toolbar(ctx, progress);
    let (search, favorites_only) = draw_module_filter(ctx, progress);

    let categories = [
        ModuleCategory::Combat,
        ModuleCategory::Movement,
        ModuleCategory::Render,
        ModuleCategory::Player,
        ModuleCategory::World,
        ModuleCategory::Misc,
        ModuleCategory::Client,
    ];
    let screen_w = ctx.screen_rect().width();
    let mut slot = Pos2::new(ORIGIN_X, ORIGIN_Y);
    let mut visible_modules = 0;

    for category in categories {
        let members: Vec<(String, &ModuleArc)> = entries
            .iter()
            .filter(|(_, cat)| *cat == category)
            .filter_map(|(id, _)| {
                let name = id.display_name();
                if favorites_only && !crate::config::is_favorite(*id) {
                    return None;
                }
                let arc = registry.get(id)?;
                if !search.is_empty() {
                    let module = arc.lock().ok()?;
                    let data = module.get_module_data();
                    if !name.to_lowercase().contains(&search)
                        && !data.description.to_lowercase().contains(&search)
                    {
                        return None;
                    }
                }
                Some((name.to_string(), arc))
            })
            .collect();
        if members.is_empty() {
            continue;
        }
        visible_modules += members.len();

        if slot.x + PANEL_W > screen_w - 20.0 && slot.x > ORIGIN_X {
            slot.x = ORIGIN_X;
            slot.y += ROW_STRIDE;
        }

        draw_panel(ctx, progress, category, &members, slot, &registry);
        slot.x += PANEL_W + GAP;
    }
    if visible_modules == 0 {
        egui::Area::new(Id::new("clickgui_no_modules"))
            .order(Order::Foreground)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_opacity(progress);
                let message = if favorites_only {
                    "No favorites yet — click ☆ on a module to pin it."
                } else if search.is_empty() {
                    "No modules are available."
                } else {
                    "No modules match your search."
                };
                ui.label(RichText::new(message).color(theme::TEXT_DIM()).size(14.0));
            });
    }
}

fn draw_backdrop(ctx: &Context, progress: f32) {
    let painter = ctx.layer_painter(LayerId::new(Order::Middle, Id::new("clickgui_backdrop")));
    let alpha = (170.0 * progress) as u8;
    painter.rect_filled(
        ctx.screen_rect(),
        Rounding::ZERO,
        Color32::from_black_alpha(alpha),
    );
}

fn draw_toolbar(ctx: &Context, progress: f32) {
    let slide = 16.0 * (1.0 - progress);
    egui::Area::new(Id::new("clickgui_toolbar"))
        .order(Order::Tooltip)
        .anchor(Align2::CENTER_TOP, Vec2::new(0.0, 12.0 - slide))
        .show(ctx, |ui| {
            ui.set_opacity(progress);
            ui.set_max_width((ctx.screen_rect().width() - 24.0).max(280.0));
            egui::Frame::none()
                .fill(theme::BASE())
                .stroke(Stroke::new(1.0_f32, theme::BORDER()))
                .rounding(Rounding::same(theme::RADIUS()))
                .inner_margin(Margin::symmetric(12.0, 7.0))
                .shadow(ui.style().visuals.window_shadow)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label(
                            RichText::new("Helium")
                                .size(15.0)
                                .strong()
                                .color(theme::TEXT()),
                        );
                        ui.label(
                            RichText::new("Client")
                                .size(15.0)
                                .strong()
                                .color(theme::ACCENT()),
                        );
                        ui.add_space(16.0);

                        let terminal_open = TerminalModule::is_active();
                        let terminal_button = Button::new(
                            RichText::new(if terminal_open {
                                "Terminal ✓"
                            } else {
                                "Terminal"
                            })
                            .size(12.5)
                            .color(if terminal_open {
                                theme::ACCENT()
                            } else {
                                theme::TEXT_DIM()
                            }),
                        )
                        .fill(theme::ELEVATED());
                        if ui.add(terminal_button).clicked() {
                            toggle_terminal();
                        }

                        ui.add_space(6.0);
                        let panic =
                            Button::new(RichText::new("Panic").size(12.5).color(theme::DANGER()))
                                .fill(theme::ELEVATED());
                        if ui.add(panic).clicked() {
                            std::thread::spawn(crate::graphic::ui_engine::call_panic);
                        }

                        ui.add_space(6.0);
                        let reset_ui = Button::new(
                            RichText::new("Reset UI")
                                .size(12.5)
                                .color(theme::TEXT_DIM()),
                        )
                        .fill(theme::ELEVATED());
                        if ui.add(reset_ui).clicked() {
                            ctx.memory_mut(|mem| mem.reset_areas());
                            ctx.data_mut(|data| data.clear());
                            crate::config::reset_ui_state();
                            crate::config::save();
                        }

                        ui.add_space(6.0);
                        let editing_hud = hud::editing();
                        let hud_editor = Button::new(
                            RichText::new(if editing_hud {
                                "HUD Editor ✓"
                            } else {
                                "HUD Editor"
                            })
                            .size(12.5)
                            .color(if editing_hud {
                                theme::ACCENT()
                            } else {
                                theme::TEXT_DIM()
                            }),
                        )
                        .fill(theme::ELEVATED());
                        if ui
                            .add(hud_editor)
                            .on_hover_text("Drag HUD elements to reposition them")
                            .clicked()
                        {
                            hud::set_editing(!editing_hud);
                        }

                        ui.add_space(6.0);
                        let reset_settings = Button::new(
                            RichText::new("Reset Settings")
                                .size(12.5)
                                .color(theme::TEXT_DIM()),
                        )
                        .fill(theme::ELEVATED());
                        if ui.add(reset_settings).clicked() {
                            crate::state::client().modules.reset_settings();
                            crate::config::save();
                        }

                        ui.add_space(6.0);
                        let profiles = Button::new(
                            RichText::new("Config Profiles")
                                .size(12.5)
                                .color(theme::TEXT_DIM()),
                        )
                        .fill(theme::ELEVATED());
                        if ui.add(profiles).clicked() {
                            PROFILES_WINDOW_OPEN.store(true, Ordering::Relaxed);
                        }

                        ui.add_space(14.0);
                        ui.separator();
                        ui.add_space(8.0);
                        ui.label(RichText::new("Theme").size(12.0).color(theme::TEXT_DIM()));
                        ui.add_space(6.0);
                        let preset_name = theme::selected_theme_name();
                        let presets = Button::new(
                            RichText::new(format!("Theme: {preset_name}"))
                                .size(12.5)
                                .color(theme::ACCENT()),
                        )
                        .fill(theme::ELEVATED());
                        if ui.add(presets).clicked() {
                            PRESET_WINDOW_OPEN.store(true, Ordering::Relaxed);
                        }

                        ui.add_space(6.0);
                        let settings = Button::new(
                            RichText::new("Client Settings")
                                .size(12.5)
                                .color(theme::TEXT_DIM()),
                        )
                        .fill(theme::ELEVATED());
                        if ui.add(settings).clicked() {
                            CLIENT_SETTINGS_OPEN.store(true, Ordering::Relaxed);
                        }

                        ui.add_space(4.0);
                        ui.separator();
                        let github = social_icon(
                            ctx,
                            Id::new("helium_github_icon"),
                            "GitHub",
                            include_bytes!("../../../assets/github.png"),
                        );
                        if ui
                            .add(
                                egui::Image::new((github.id(), Vec2::splat(20.0)))
                                    .sense(Sense::click()),
                            )
                            .on_hover_text("Open HeliumClient on GitHub")
                            .clicked()
                        {
                            open_social_link("https://github.com/26140070-cpu/Helium-Client");
                        }

                        let discord = social_icon(
                            ctx,
                            Id::new("helium_discord_icon"),
                            "Discord",
                            include_bytes!("../../../assets/discord.png"),
                        );
                        if ui
                            .add(
                                egui::Image::new((discord.id(), Vec2::splat(20.0)))
                                    .sense(Sense::click()),
                            )
                            .on_hover_text("Join the HeliumClient Discord")
                            .clicked()
                        {
                            open_social_link("https://discord.gg/G6CwQHBJF6");
                        }

                        ui.add_space(6.0);
                        let conflicts =
                            keybind_conflicts(&crate::state::client().modules.by_id()).len();
                        let keybinds = Button::new(
                            RichText::new(format!("Keybinds ({conflicts})"))
                                .size(12.5)
                                .color(if conflicts > 0 {
                                    theme::WARN()
                                } else {
                                    theme::TEXT_DIM()
                                }),
                        )
                        .fill(theme::ELEVATED());
                        if ui.add(keybinds).clicked() {
                            KEYBINDS_WINDOW_OPEN.store(true, Ordering::Relaxed);
                        }
                    });
                });
        });
    draw_preset_window(ctx);
    draw_theme_editor(ctx);
    draw_keybind_window(ctx);
    draw_client_settings(ctx);
    draw_profiles_window(ctx);
}

fn social_icon(ctx: &Context, id: Id, name: &str, bytes: &[u8]) -> egui::TextureHandle {
    if let Some(texture) = ctx.data(|data| data.get_temp::<egui::TextureHandle>(id)) {
        return texture;
    }

    let image = image::load_from_memory(bytes)
        .unwrap_or_else(|error| panic!("failed to decode embedded {name} icon: {error}"))
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let texture = ctx.load_texture(
        format!("helium_{name}_icon"),
        egui::ColorImage::from_rgba_unmultiplied(size, &image.into_raw()),
        egui::TextureOptions::LINEAR,
    );
    ctx.data_mut(|data| data.insert_temp(id, texture.clone()));
    texture
}

fn open_social_link(url: &str) {
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn();
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(url).spawn();
    #[cfg(target_os = "linux")]
    let result = std::process::Command::new("xdg-open").arg(url).spawn();

    if let Err(error) = result {
        log::error!("could not open social link {url}: {error}");
    }
}

fn draw_profiles_window(ctx: &Context) {
    let mut open = PROFILES_WINDOW_OPEN.load(Ordering::Relaxed);
    if !open {
        return;
    }

    let name_id = Id::new("config_profile_name");
    let selected_id = Id::new("config_profile_selected");
    let message_id = Id::new("config_profile_message");
    let mut name = ctx.data(|data| data.get_temp::<String>(name_id).unwrap_or_default());
    let mut selected = ctx.data(|data| data.get_temp::<String>(selected_id).unwrap_or_default());
    let mut message = ctx.data(|data| data.get_temp::<String>(message_id));
    let profiles = crate::config::list_profiles();

    egui::Window::new(RichText::new("Configuration profiles").size(16.0).strong())
        .id(Id::new("config_profiles"))
        .order(Order::Tooltip)
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_size(Vec2::new(380.0, 240.0))
        .max_size(ctx.screen_rect().size() * 0.9)
        .constrain(true)
        .show(ctx, |ui| {
            ui.label(
                RichText::new("Save and switch between module setups.")
                    .size(13.0)
                    .color(theme::TEXT_DIM()),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut name)
                        .hint_text("Profile name")
                        .desired_width(180.0),
                );
                if ui.button("Save").clicked() {
                    match crate::config::save_profile(&name) {
                        Ok(()) => {
                            selected.clone_from(&name);
                            message = Some(format!("Saved profile '{name}'."));
                        }
                        Err(error) => message = Some(error.to_string()),
                    }
                }
            });

            match &profiles {
                Ok(profiles) if !profiles.is_empty() => {
                    egui::ComboBox::from_id_salt("config_profile_selector")
                        .selected_text(if selected.is_empty() {
                            "Select a profile"
                        } else {
                            selected.as_str()
                        })
                        .show_ui(ui, |ui| {
                            for profile in profiles {
                                ui.selectable_value(&mut selected, profile.clone(), profile);
                            }
                        });
                    ui.horizontal(|ui| {
                        if ui.button("Load").clicked() {
                            match crate::config::load_profile(&selected) {
                                Ok(()) => {
                                    crate::config::save();
                                    message = Some(format!("Loaded profile '{selected}'."));
                                }
                                Err(error) => message = Some(error.to_string()),
                            }
                        }
                        if ui.button("Delete").clicked() {
                            match crate::config::delete_profile(&selected) {
                                Ok(()) => {
                                    selected.clear();
                                    message = Some("Deleted profile.".to_string());
                                }
                                Err(error) => message = Some(error.to_string()),
                            }
                        }
                    });
                }
                Ok(_) => {
                    ui.label(
                        RichText::new("No saved profiles yet.")
                            .size(12.0)
                            .color(theme::TEXT_DIM()),
                    );
                }
                Err(error) => {
                    message = Some(format!("Could not list profiles: {error}"));
                    ui.label(RichText::new("Profile list unavailable").color(theme::DANGER()));
                }
            }

            if let Some(message) = &message {
                ui.add_space(6.0);
                ui.label(RichText::new(message).size(12.0).color(theme::TEXT_DIM()));
            }
        });

    ctx.data_mut(|data| {
        data.insert_temp(name_id, name);
        data.insert_temp(selected_id, selected);
        if let Some(message) = message {
            data.insert_temp(message_id, message);
        }
    });
    PROFILES_WINDOW_OPEN.store(open, Ordering::Relaxed);
}

fn toggle_terminal() {
    let Some(module) = crate::state::client().modules.get(ModuleId::Terminal) else {
        log::error!("terminal: module is missing from the registry");
        return;
    };
    let Ok(mut module) = module.lock() else {
        log::error!("terminal: module lock poisoned while toggling from the toolbar");
        return;
    };
    let next = !module.get_module_data().enabled;
    if crate::state::client().modules.is_active() {
        let result = if next {
            module.on_start()
        } else {
            module.on_stop()
        };
        if let Err(error) = result {
            log::error!("terminal: failed to toggle from the toolbar: {error}");
            return;
        }
    }
    module.get_module_data_mut().set_enabled(next);
    drop(module);
    crate::config::save();
}

fn draw_module_filter(ctx: &Context, progress: f32) -> (String, bool) {
    let search_id = Id::new("clickgui_module_search");
    let favorites_id = Id::new("clickgui_favorites_only");
    let mut search = ctx.data(|data| data.get_temp::<String>(search_id).unwrap_or_default());
    let mut favorites_only = ctx.data(|data| data.get_temp::<bool>(favorites_id).unwrap_or(false));
    egui::Area::new(Id::new("clickgui_module_filter"))
        .order(Order::Foreground)
        .anchor(Align2::LEFT_TOP, Vec2::new(14.0, 64.0))
        .show(ctx, |ui| {
            ui.set_opacity(progress);
            egui::Frame::none()
                .fill(theme::BASE())
                .stroke(Stroke::new(1.0_f32, theme::BORDER()))
                .rounding(Rounding::same(theme::RADIUS()))
                .inner_margin(Margin::symmetric(8.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut search)
                                .hint_text("Search modules…")
                                .desired_width(150.0),
                        );
                        ui.checkbox(&mut favorites_only, "★")
                            .on_hover_text("Show favorite modules only");
                    });
                });
        });
    ctx.data_mut(|data| {
        data.insert_temp(search_id, search.clone());
        data.insert_temp(favorites_id, favorites_only);
    });
    (search.trim().to_lowercase(), favorites_only)
}

fn draw_preset_window(ctx: &Context) {
    let mut open = PRESET_WINDOW_OPEN.load(Ordering::Relaxed);
    if !open {
        return;
    }

    egui::Window::new(RichText::new("Appearance").size(16.0).strong())
        .id(Id::new("theme_preset_window"))
        .order(Order::Tooltip)
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_pos(ctx.screen_rect().center() - Vec2::new(240.0, 240.0))
        .default_size(Vec2::new(480.0, 480.0))
        .max_size(ctx.screen_rect().size() * 0.9)
        .constrain(true)
        .show(ctx, |ui| {
            ui.label(
                RichText::new("Pick a ready-made palette or make your own.")
                    .size(14.0)
                    .color(theme::TEXT_DIM()),
            );
            ui.add_space(8.0);
            if ui
                .add(
                    Button::new(
                        RichText::new("＋  Create or edit a custom theme")
                            .size(14.0)
                            .strong()
                            .color(theme::ACCENT()),
                    )
                    .fill(theme::ELEVATED())
                    .min_size(Vec2::new(ui.available_width(), 34.0)),
                )
                .clicked()
            {
                THEME_EDITOR_OPEN.store(true, Ordering::Relaxed);
            }
            ui.add_space(8.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
                    for (index, preset) in theme::PRESETS.iter().enumerate() {
                        let size = Vec2::new(96.0, 68.0);
                        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
                        let accent =
                            Color32::from_rgb(preset.accent[0], preset.accent[1], preset.accent[2]);
                        let base =
                            Color32::from_rgb(preset.base[0], preset.base[1], preset.base[2]);
                        let selected = theme::selected_custom_theme().is_none()
                            && theme::selected_preset() == index;
                        paint_preset_gradient(ui.painter(), rect, accent, base);
                        let label_rect =
                            Rect::from_min_max(Pos2::new(rect.min.x, rect.max.y - 19.0), rect.max);
                        ui.painter().rect_filled(
                            label_rect,
                            Rounding::ZERO,
                            Color32::from_black_alpha(150),
                        );
                        ui.painter().text(
                            label_rect.center(),
                            Align2::CENTER_CENTER,
                            preset.name,
                            FontId::proportional(12.5),
                            Color32::WHITE,
                        );
                        ui.painter().rect_stroke(
                            rect,
                            Rounding::same(4.0),
                            Stroke::new(
                                if selected { 2.0_f32 } else { 1.0_f32 },
                                if selected {
                                    theme::ACCENT()
                                } else {
                                    theme::BORDER()
                                },
                            ),
                        );
                        if response.clicked() {
                            theme::set_preset(index);
                            crate::config::save();
                        }
                        response.on_hover_text(format!("Apply {} theme", preset.name));
                    }
                });
            });
        });
    PRESET_WINDOW_OPEN.store(open, Ordering::Relaxed);
}

fn draw_theme_editor(ctx: &Context) {
    let mut open = THEME_EDITOR_OPEN.load(Ordering::Relaxed);
    if !open {
        return;
    }
    let themes = theme::custom_themes();
    let mut live = theme::live();
    let mut changed = false;
    let mut apply_name = None;
    let mut save_requested = false;
    let mut theme_name = CUSTOM_THEME_NAME
        .lock()
        .map(|name| name.clone())
        .unwrap_or_default();
    let mut save_error = ctx.data(|data| data.get_temp::<String>(Id::new("theme_save_error")));

    egui::Window::new(RichText::new("Custom theme").size(16.0).strong())
        .id(Id::new("custom_theme_editor"))
        .order(Order::Tooltip)
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_pos(ctx.screen_rect().center() - Vec2::new(250.0, 230.0))
        .default_size(Vec2::new(500.0, 460.0))
        .max_size(ctx.screen_rect().size() * 0.9)
        .constrain(true)
        .show(ctx, |ui| {
            ui.label(
                RichText::new("Adjust the main colors. Changes preview immediately.")
                    .size(14.0)
                    .color(theme::TEXT_DIM()),
            );
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(label("Load saved theme"));
                egui::ComboBox::from_id_salt("saved_custom_theme_selector")
                    .selected_text(
                        theme::selected_custom_theme()
                            .unwrap_or_else(|| "Choose a custom theme".to_string()),
                    )
                    .show_ui(ui, |ui| {
                        for custom in &themes {
                            if ui.selectable_label(false, &custom.name).clicked() {
                                apply_name = Some(custom.name.clone());
                            }
                        }
                    });
            });
            ui.add_space(8.0);
            egui::ScrollArea::vertical()
                .max_height((ctx.screen_rect().height() * 0.55).max(180.0))
                .show(ui, |ui| {
                    egui::CollapsingHeader::new(RichText::new("Interface").size(14.0).strong())
                        .default_open(true)
                        .show(ui, |ui| {
                            theme_color_row(ui, &mut changed, "Accent", &mut live.accent);
                            theme_color_row(ui, &mut changed, "Background", &mut live.base);
                            theme_color_row(ui, &mut changed, "Panel", &mut live.surface);
                            theme_color_row(
                                ui,
                                &mut changed,
                                "Panel hover",
                                &mut live.surface_hover,
                            );
                            theme_color_row(ui, &mut changed, "Raised surface", &mut live.elevated);
                            theme_color_row(ui, &mut changed, "Border", &mut live.border);
                        });
                    egui::CollapsingHeader::new(RichText::new("Text").size(14.0).strong()).show(
                        ui,
                        |ui| {
                            theme_color_row(ui, &mut changed, "Primary", &mut live.text);
                            theme_color_row(ui, &mut changed, "Secondary", &mut live.text_dim);
                            theme_color_row(ui, &mut changed, "Muted", &mut live.text_muted);
                        },
                    );
                    egui::CollapsingHeader::new(RichText::new("Status colors").size(14.0).strong())
                        .show(ui, |ui| {
                            theme_color_row(ui, &mut changed, "Accent dim", &mut live.accent_dim);
                            theme_color_row(ui, &mut changed, "Warning", &mut live.warn);
                            theme_color_row(ui, &mut changed, "Danger", &mut live.danger);
                        });
                    egui::CollapsingHeader::new(RichText::new("Fine tuning").size(14.0).strong())
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Inner corner roundness")
                                        .size(14.0)
                                        .color(theme::TEXT_DIM()),
                                );
                                changed |= ui
                                    .add(
                                        egui::Slider::new(&mut live.radius_inner, 0.0..=12.0)
                                            .show_value(true),
                                    )
                                    .changed();
                            });
                        });
                });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(label("Corner roundness"));
                changed |= ui
                    .add(egui::Slider::new(&mut live.radius, 0.0..=16.0).show_value(true))
                    .changed();
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(label("Theme name"));
                ui.add(
                    egui::TextEdit::singleline(&mut theme_name)
                        .hint_text("My theme")
                        .desired_width(160.0),
                );
                if ui.button("Save custom theme").clicked() {
                    save_requested = true;
                }
            });
            if let Some(error) = &save_error {
                ui.label(RichText::new(error).color(theme::DANGER()).size(11.0));
            }
        });
    if changed {
        theme::set_live(live);
    }
    if save_requested {
        match theme::save_custom_theme(&theme_name) {
            Ok(()) => {
                save_error = None;
                crate::config::save();
            }
            Err(error) => save_error = Some(error.to_string()),
        }
    }
    if let Ok(mut name) = CUSTOM_THEME_NAME.lock() {
        *name = theme_name;
    }
    ctx.data_mut(|data| {
        if let Some(error) = save_error {
            data.insert_temp(Id::new("theme_save_error"), error);
        } else {
            data.remove::<String>(Id::new("theme_save_error"));
        }
    });
    if let Some(name) = apply_name {
        if theme::apply_custom_theme(&name) {
            crate::config::save();
        }
    }
    THEME_EDITOR_OPEN.store(open, Ordering::Relaxed);
}

fn theme_color_row(ui: &mut Ui, changed: &mut bool, name: &str, color: &mut Color32) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(name).size(14.0).color(theme::TEXT_DIM()));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            *changed |= egui::color_picker::color_edit_button_srgba(
                ui,
                color,
                egui::color_picker::Alpha::Opaque,
            )
            .changed();
        });
    });
}

fn draw_keybind_window(ctx: &Context) {
    let mut open = KEYBINDS_WINDOW_OPEN.load(Ordering::Relaxed);
    if !open {
        return;
    }
    let conflicts = keybind_conflicts(&crate::state::client().modules.by_id());
    egui::Window::new(RichText::new("Keybinds").size(16.0).strong())
        .id(Id::new("keybind_conflicts"))
        .order(Order::Tooltip)
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_pos(ctx.screen_rect().center() - Vec2::new(210.0, 170.0))
        .default_size(Vec2::new(420.0, 340.0))
        .max_size(ctx.screen_rect().size() * 0.9)
        .constrain(true)
        .show(ctx, |ui| {
            if conflicts.is_empty() {
                ui.label(
                    RichText::new("No keybind conflicts detected.")
                        .size(15.0)
                        .color(theme::ACCENT())
                        .strong(),
                );
            } else {
                ui.label(
                    RichText::new(format!(
                        "{} duplicate key{}",
                        conflicts.len(),
                        if conflicts.len() == 1 { "" } else { "s" }
                    ))
                    .size(15.0)
                    .color(theme::WARN())
                    .strong(),
                );
                ui.add_space(6.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (key, modules) in &conflicts {
                        egui::Frame::none()
                            .fill(theme::SURFACE())
                            .rounding(Rounding::same(5.0))
                            .inner_margin(Margin::symmetric(10.0, 8.0))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(format!("Key: {key}"))
                                        .size(14.0)
                                        .strong()
                                        .color(theme::ACCENT()),
                                );
                                for module in modules {
                                    ui.label(
                                        RichText::new(format!("• {module}"))
                                            .size(14.0)
                                            .color(theme::TEXT()),
                                    );
                                }
                            });
                        ui.add_space(5.0);
                    }
                });
                ui.label(
                    RichText::new(
                        "Give each module a different key to avoid toggling them together.",
                    )
                    .color(theme::TEXT_DIM())
                    .size(13.0),
                );
            }
        });
    KEYBINDS_WINDOW_OPEN.store(open, Ordering::Relaxed);
}

fn draw_client_settings(ctx: &Context) {
    let mut open = CLIENT_SETTINGS_OPEN.load(Ordering::Relaxed);
    if !open {
        return;
    }
    let capture_id = Id::new("gui_key_capture");
    let mut capturing = ctx.data(|data| data.get_temp::<bool>(capture_id).unwrap_or(false));
    let mut capture_started = false;
    let mut capture_cancelled = false;
    let mut changed_key = None;
    egui::Window::new(RichText::new("Client settings").size(16.0).strong())
        .id(Id::new("client_settings"))
        .order(Order::Tooltip)
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_pos(ctx.screen_rect().center() - Vec2::new(190.0, 130.0))
        .default_size(Vec2::new(380.0, 250.0))
        .max_size(ctx.screen_rect().size() * 0.9)
        .constrain(true)
        .show(ctx, |ui| {
            ui.label(
                RichText::new("Personalize how you open the ClickGUI.")
                    .size(14.0)
                    .color(theme::TEXT_DIM()),
            );
            ui.add_space(12.0);
            egui::Frame::none()
                .fill(theme::SURFACE())
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::symmetric(12.0, 10.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("Open ClickGUI")
                            .size(15.0)
                            .strong()
                            .color(theme::TEXT()),
                    );
                    ui.label(
                        RichText::new(
                            "Choose a key that is easy to reach and not used by another action.",
                        )
                        .size(13.0)
                        .color(theme::TEXT_DIM()),
                    );
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "Current key: {}",
                                format_keyboard_key(crate::config::gui_open_key())
                            ))
                            .size(15.0)
                            .color(theme::ACCENT()),
                        );
                        let caption = if capturing {
                            "Press any key…"
                        } else {
                            "Change key"
                        };
                        if ui.button(RichText::new(caption).size(14.0)).clicked() {
                            capturing = true;
                            capture_started = true;
                        }
                    });
                    if capturing {
                        ui.label(
                            RichText::new(
                                "Press the key you want to use, or close this window to cancel.",
                            )
                            .size(13.0)
                            .color(theme::WARN()),
                        );
                    }
                });
            ui.add_space(10.0);
            if ui
                .button(RichText::new("Restore Right Shift").size(14.0))
                .clicked()
            {
                changed_key = Some(-1);
            }
            ui.label(
                RichText::new(
                    "If the saved key is empty or invalid, Right Shift is used automatically.",
                )
                .size(13.0)
                .color(theme::TEXT_DIM()),
            );
        });

    if capture_started {
        crate::graphic::input::begin_gui_key_capture();
    }
    if capturing && !capture_started {
        let pressed = LAST_KEY_PRESSED.swap(-1, Ordering::Relaxed);
        if pressed >= 0 {
            changed_key = Some(pressed);
            capturing = false;
            crate::graphic::input::finish_gui_key_capture();
        }
    }
    if changed_key.is_some() {
        let key = changed_key.unwrap_or(-1);
        crate::config::set_gui_open_key(key);
        crate::config::save();
        capturing = false;
        crate::graphic::input::finish_gui_key_capture();
    }
    if !open && capturing {
        capture_cancelled = true;
        capturing = false;
        crate::graphic::input::finish_gui_key_capture();
    }
    if capture_cancelled {
        LAST_KEY_PRESSED.store(-1, Ordering::Relaxed);
    }
    ctx.data_mut(|data| data.insert_temp(capture_id, capturing));
    CLIENT_SETTINGS_OPEN.store(open, Ordering::Relaxed);
}

fn keybind_conflicts(registry: &ModuleMap) -> Vec<(String, Vec<String>)> {
    let mut bindings: HashMap<KeyboardKey, Vec<String>> = HashMap::new();
    let gui_key = KeyboardKey::from(crate::config::gui_open_key());
    if gui_key != KeyboardKey::KeyNone {
        bindings
            .entry(gui_key)
            .or_default()
            .push("ClickGUI".to_string());
    }
    for (id, module) in registry {
        let Ok(module) = module.lock() else {
            continue;
        };
        let key = module.get_module_data().key_bind;
        if key != KeyboardKey::KeyNone {
            bindings
                .entry(key)
                .or_default()
                .push(id.display_name().to_string());
        }
    }
    let mut conflicts = bindings
        .into_iter()
        .filter(|(_, modules)| modules.len() > 1)
        .map(|(key, mut modules)| {
            modules.sort();
            (key.to_string(), modules)
        })
        .collect::<Vec<_>>();
    conflicts.sort_by(|left, right| left.0.cmp(&right.0));
    conflicts
}

fn format_keyboard_key(key: i32) -> String {
    let key_name = KeyboardKey::from(key).to_string();
    if key_name == "None" {
        format!("Key {key}")
    } else {
        key_name
    }
}

fn paint_preset_gradient(painter: &Painter, rect: Rect, start: Color32, end: Color32) {
    const STEPS: usize = 32;
    let step_width = rect.width() / STEPS as f32;
    for step in 0..STEPS {
        let x0 = rect.min.x + step as f32 * step_width;
        let x1 = if step + 1 == STEPS {
            rect.max.x
        } else {
            x0 + step_width + 0.5
        };
        painter.rect_filled(
            Rect::from_min_max(Pos2::new(x0, rect.min.y), Pos2::new(x1, rect.max.y)),
            Rounding::ZERO,
            theme::lerp_color(start, end, step as f32 / (STEPS - 1) as f32),
        );
    }
}

fn draw_panel(
    ctx: &Context,
    progress: f32,
    category: ModuleCategory,
    members: &[(String, &ModuleArc)],
    slot: Pos2,
    registry: &ModuleMap,
) {
    let name = category.display_name();

    let target = crate::config::panel_pos(category)
        .map(|p| Pos2::new(p[0], p[1]))
        .unwrap_or(slot);
    let pos = anim::spring_pos(
        ctx,
        Id::new("panel_pos").with(name),
        target,
        SpringCfg::PANEL,
    );

    let slide = 28.0 * (1.0 - progress);
    let render_pos = pos + Vec2::new(0.0, slide);

    egui::Area::new(Id::new("clickgui_panel").with(name))
        .current_pos(render_pos)
        .order(Order::Foreground)
        .show(ctx, |ui| {
            ui.set_opacity(progress.clamp(0.0, 1.0));
            ui.ctx().set_pixels_per_point(ui.ctx().pixels_per_point());
            egui::Frame::none()
                .fill(theme::BASE())
                .stroke(Stroke::new(1.0_f32, theme::BORDER()))
                .rounding(Rounding::same(theme::RADIUS()))
                .shadow(ui.style().visuals.window_shadow)
                .show(ui, |ui| {
                    ui.set_min_width(PANEL_W);
                    ui.set_max_width(PANEL_W);

                    let max_rows_h =
                        (ctx.screen_rect().height() - render_pos.y - TITLE_H - 16.0).max(ROW_H);
                    ui.set_max_height(max_rows_h + TITLE_H + 8.0);

                    draw_title_bar(ui, category);

                    let mut scroll = egui::style::ScrollStyle::floating();
                    scroll.bar_width = 5.0;
                    ui.style_mut().spacing.scroll = scroll;

                    let active_fg = ui.visuals().widgets.active.fg_stroke.color;
                    ui.visuals_mut().widgets.active.fg_stroke.color = theme::ACCENT();
                    egui::ScrollArea::vertical()
                        .id_salt(name)
                        .max_height(max_rows_h)
                        .auto_shrink([false, true])
                        .drag_to_scroll(false)
                        .show(ui, |ui| {
                            ui.visuals_mut().widgets.active.fg_stroke.color = active_fg;
                            ui.set_min_width(PANEL_W);
                            ui.set_max_width(PANEL_W);
                            for (module_name, arc) in members {
                                draw_module_row(ui, module_name, arc, registry);
                            }
                            ui.add_space(6.0);
                        });
                });
        });
}

fn draw_title_bar(ui: &mut Ui, category: ModuleCategory) {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(PANEL_W, TITLE_H), Sense::drag());
    if response.dragged() {
        let delta = ui.ctx().input(|i| i.pointer.delta());
        let current = crate::config::panel_pos(category)
            .map(|p| Pos2::new(p[0], p[1]))
            .unwrap_or(rect.min);
        let next = current + delta;
        crate::config::set_panel_pos(category, [next.x, next.y]);
    }

    let painter = ui.painter();
    painter.rect_filled(
        rect,
        Rounding {
            nw: theme::RADIUS(),
            ne: theme::RADIUS(),
            sw: 0.0,
            se: 0.0,
        },
        theme::ELEVATED(),
    );
    painter.text(
        rect.left_center() + Vec2::new(12.0, 0.0),
        Align2::LEFT_CENTER,
        category.display_name(),
        FontId::proportional(13.5),
        theme::TEXT(),
    );
    let underline = Rect::from_min_size(
        rect.left_bottom() - Vec2::new(0.0, 2.0),
        Vec2::new(PANEL_W, 2.0),
    );
    painter.rect_filled(underline, Rounding::ZERO, theme::ACCENT());
}

fn draw_module_row(ui: &mut Ui, name: &str, arc: &ModuleArc, registry: &ModuleMap) {
    let mut module = arc.lock().unwrap();
    let enabled = module.get_module_data().enabled;
    let id = module.get_module_data().id;

    let (rect, response) = ui.allocate_exact_size(Vec2::new(PANEL_W, ROW_H), Sense::click());
    let arrow_zone = Rect::from_min_size(
        Pos2::new(rect.max.x - 24.0, rect.min.y),
        Vec2::new(24.0, ROW_H),
    );
    let favorite_zone = Rect::from_min_size(
        Pos2::new(rect.max.x - 48.0, rect.min.y),
        Vec2::new(22.0, ROW_H),
    );

    let arrow_response = ui.interact(arrow_zone, Id::new("row_arrow").with(name), Sense::click());
    let favorite_response = ui.interact(
        favorite_zone,
        Id::new("row_favorite").with(name),
        Sense::click(),
    );

    let expanded = crate::config::is_expanded(id);
    let mut collapse = egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        Id::new("row_collapse").with(name),
        expanded,
    );

    let ctx = ui.ctx();
    let hover = anim::toggle(
        ctx,
        Id::new("row_hov").with(name),
        response.hovered(),
        0.16,
        Easing::OutCubic,
    );
    let enable = anim::toggle(
        ctx,
        Id::new("row_en").with(name),
        enabled,
        0.22,
        Easing::OutBack,
    );

    {
        let painter = ui.painter();
        let hover_bg = theme::lerp_color(
            Color32::TRANSPARENT,
            theme::with_alpha(theme::SURFACE_HOVER(), 0.95),
            hover,
        );
        painter.rect_filled(rect, Rounding::same(3.0 * hover), hover_bg);
        if hover > 0.01 {
            painter.rect_stroke(
                rect.shrink(0.5),
                Rounding::same(3.0),
                Stroke::new(
                    1.0 * hover,
                    theme::with_alpha(theme::ACCENT(), hover * 0.55),
                ),
            );
        }
        if enable > 0.001 {
            let bar = Rect::from_center_size(
                Pos2::new(rect.min.x + 1.5, rect.center().y),
                Vec2::new(3.0, ROW_H * enable),
            );
            painter.rect_filled(bar, Rounding::ZERO, theme::ACCENT());
        }
        painter.text(
            Pos2::new(rect.min.x + 12.0, rect.center().y),
            Align2::LEFT_CENTER,
            name,
            FontId::proportional(13.0),
            theme::lerp_color(theme::TEXT_DIM(), theme::TEXT(), hover.max(enable)),
        );
        let favorite = crate::config::is_favorite(id);
        painter.text(
            favorite_zone.center(),
            Align2::CENTER_CENTER,
            if favorite { "★" } else { "☆" },
            FontId::proportional(14.0),
            if favorite || favorite_response.hovered() {
                theme::WARN()
            } else {
                theme::TEXT_MUTED()
            },
        );
    }

    let mut favorite_changed = false;
    if favorite_response.clicked() {
        crate::config::set_favorite(id, !crate::config::is_favorite(id));
        favorite_changed = true;
    } else if arrow_response.clicked() || response.secondary_clicked() {
        crate::config::set_expanded(id, !expanded);
    } else if response.clicked() {
        let next = !enabled;
        module.get_module_data_mut().set_enabled(next);

        if crate::state::client().modules.is_active() {
            let _ = if next {
                module.on_start()
            } else {
                module.on_stop()
            };
        }
    }

    collapse.set_open(crate::config::is_expanded(id));
    let openness = collapse.openness(ui.ctx());
    let chevron_color = if arrow_response.hovered() {
        theme::TEXT()
    } else {
        theme::TEXT_MUTED()
    };
    paint_chevron(ui.painter(), arrow_zone.center(), openness, chevron_color);

    collapse.show_body_unindented(ui, |ui| {
        draw_settings(ui, module.get_module_data_mut(), arc, registry);
    });
    collapse.store(ui.ctx());
    drop(module);
    if favorite_changed {
        crate::config::save();
    }
}

fn paint_chevron(painter: &Painter, center: Pos2, open: f32, color: Color32) {
    const S: f32 = 3.6;
    let angle = open.clamp(0.0, 1.0) * std::f32::consts::FRAC_PI_2;
    let (sin, cos) = angle.sin_cos();
    let rotate = |v: Vec2| Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos);
    let points = [
        Vec2::new(S, 0.0),
        Vec2::new(-S * 0.7, -S),
        Vec2::new(-S * 0.7, S),
    ]
    .into_iter()
    .map(|v| center + rotate(v))
    .collect::<Vec<_>>();
    painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
}

fn draw_settings(ui: &mut Ui, data: &mut ModuleData, arc: &ModuleArc, registry: &ModuleMap) {
    egui::Frame::none()
        .fill(theme::SURFACE())
        .inner_margin(Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.set_min_width(PANEL_W - 20.0);
            ui.set_max_width(PANEL_W - 20.0);
            ui.spacing_mut().item_spacing.y = 7.0;

            keybind_row(ui, data, arc, registry);

            for setting in &mut data.settings {
                match setting {
                    ModuleSetting::Toggle { name, value } => {
                        ui.checkbox(value, label(name));
                    }
                    ModuleSetting::Slider {
                        name,
                        value,
                        min,
                        max,
                    } => {
                        labelled_value(ui, name, &format!("{value:.2}"));
                        ui.spacing_mut().slider_width = ui.available_width();
                        ui.add(egui::Slider::new(value, *min..=*max).show_value(false));
                    }
                    ModuleSetting::Choice {
                        name,
                        value,
                        options,
                    } => {
                        ui.label(label(name));
                        egui::ComboBox::from_id_salt(Id::new("choice").with(name.as_str()))
                            .width(ui.available_width())
                            .selected_text(
                                RichText::new(
                                    options.get(*value).map(String::as_str).unwrap_or("—"),
                                )
                                .size(12.0),
                            )
                            .show_ui(ui, |ui| {
                                for (idx, option) in options.iter().enumerate() {
                                    ui.selectable_value(value, idx, option.as_str());
                                }
                            });
                    }
                    ModuleSetting::Color { name, value } => {
                        ui.horizontal(|ui| {
                            ui.label(label(name));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let mut rgba = egui::Rgba::from_rgba_unmultiplied(
                                    value[0], value[1], value[2], 1.0,
                                );
                                let changed = egui::color_picker::color_edit_button_rgba(
                                    ui,
                                    &mut rgba,
                                    egui::color_picker::Alpha::Opaque,
                                )
                                .changed();
                                if changed {
                                    let [r, g, b, _] = rgba.to_array();
                                    value[0] = r;
                                    value[1] = g;
                                    value[2] = b;
                                }
                            });
                        });
                    }
                }
            }
            ui.add_space(1.0);
        });
}

fn keybind_row(ui: &mut Ui, data: &mut ModuleData, arc: &ModuleArc, registry: &ModuleMap) {
    ui.horizontal(|ui| {
        ui.label(label("Bind"));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let bind_id = Id::new("kb_listen").with(data.name());
            let listening = ui.data(|d| d.get_temp::<bool>(bind_id).unwrap_or(false));

            let caption = if listening {
                if capture_keybind(data, arc, registry) {
                    ui.data_mut(|d| d.insert_temp(bind_id, false));
                }
                "press…".to_string()
            } else {
                data.key_bind.to_string()
            };

            let color = if listening {
                theme::ACCENT()
            } else {
                theme::TEXT_DIM()
            };
            let button = Button::new(RichText::new(caption).size(12.0).color(color))
                .fill(theme::ELEVATED())
                .stroke(Stroke::NONE);
            if ui.add(button).clicked() {
                let next = !listening;
                ui.data_mut(|d| d.insert_temp(bind_id, next));
                if next {
                    LAST_KEY_PRESSED.store(-1, Ordering::Relaxed);
                }
            }
        });
    });
}

fn capture_keybind(data: &mut ModuleData, arc: &ModuleArc, registry: &ModuleMap) -> bool {
    let pressed = LAST_KEY_PRESSED.swap(-1, Ordering::Relaxed);
    if pressed == -1 {
        return false;
    }
    let key = KeyboardKey::from(pressed);

    if key == KeyboardKey::KeyEscape {
        data.key_bind = KeyboardKey::KeyNone;
        return true;
    }

    for other in registry.values() {
        if Arc::ptr_eq(arc, other) {
            continue;
        }
        let owner = other.lock().unwrap();
        if owner.get_module_data().key_bind == key {
            let owner_name = owner.get_module_data().name().to_string();
            drop(owner);
            Notification::send(
                NotificationType::Warning,
                "Keybind in use",
                &format!("'{owner_name}' is bound to {key}"),
            );
            return true;
        }
    }

    data.key_bind = key;
    true
}

fn label(text: &str) -> RichText {
    RichText::new(text).size(12.0).color(theme::TEXT_DIM())
}

fn labelled_value(ui: &mut Ui, name: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(label(name));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(value).size(12.0).color(theme::ACCENT()));
        });
    });
}
