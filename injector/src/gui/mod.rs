mod theme;
mod widgets;

use eframe::{Frame, NativeOptions};
use egui::{
    pos2, Align, Color32, Context, Layout, Margin, Rect, RichText, ScrollArea, TextureHandle,
    ViewportBuilder,
};
use std::time::Duration;

use crate::app::{AppView, InjectorApp};

const PANORAMA_FRAMES: [&[u8]; 6] = [
    include_bytes!("../../assets/panorama/0.png"),
    include_bytes!("../../assets/panorama/1.png"),
    include_bytes!("../../assets/panorama/2.png"),
    include_bytes!("../../assets/panorama/3.png"),
    include_bytes!("../../assets/panorama/4.png"),
    include_bytes!("../../assets/panorama/5.png"),
];

const PANORAMA_FRAME_SECONDS: f64 = 10.0;
const PANORAMA_FADE_SECONDS: f64 = 4.0;

pub fn run() -> Result<(), eframe::Error> {
    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([500.0, 580.0])
            .with_min_inner_size([320.0, 360.0]),
        ..Default::default()
    };
    eframe::run_native(
        "HeliumClient Injector",
        native_options,
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx);
            let logo = load_texture(
                &cc.egui_ctx,
                "helium-logo",
                include_bytes!("../../assets/helium-logo.png"),
            );
            let github = load_texture(
                &cc.egui_ctx,
                "github-logo",
                include_bytes!("../../../assets/github.png"),
            );
            let discord = load_texture(
                &cc.egui_ctx,
                "discord-logo",
                include_bytes!("../../../assets/discord.png"),
            );
            let panoramas = PANORAMA_FRAMES
                .iter()
                .enumerate()
                .map(|(index, bytes)| {
                    load_texture(&cc.egui_ctx, &format!("injector-panorama-{index}"), bytes)
                })
                .collect();
            let mut app = InjectorApp::new();
            if app.settings().auto_scan_on_start {
                app.scan();
            }
            Ok(Box::new(InjectorGui {
                app,
                logo,
                github,
                discord,
                panoramas,
            }))
        }),
    )
}

struct InjectorGui {
    app: InjectorApp,
    logo: TextureHandle,
    github: TextureHandle,
    discord: TextureHandle,
    panoramas: Vec<TextureHandle>,
}

impl eframe::App for InjectorGui {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.app.poll();
        self.app.flush_settings_if_dirty();

        egui::TopBottomPanel::top("header")
            .frame(theme::header_frame())
            .show(ctx, |ui| {
                if let Some(next) =
                    widgets::header(ui, &self.logo, &self.github, &self.discord, self.app.view())
                {
                    self.app.set_view(next);
                }
            });

        if *self.app.view() == AppView::Main {
            egui::TopBottomPanel::top("main_toolbar")
                .frame(theme::toolbar_frame())
                .show(ctx, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        if widgets::tool_button(ui, "↻  Scan", !self.app.is_busy()).clicked() {
                            self.app.scan();
                        }
                        if widgets::tool_button(ui, "📂  Client Folder", true).clicked() {
                            if let Some(folder) = self.app.client_folder() {
                                open_folder(&folder);
                            }
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!("{} found", self.app.processes().len()))
                                    .color(theme::Palette::TEXT_DIM),
                            );
                        });
                    });
                });
        }

        if *self.app.view() == AppView::Main {
            egui::TopBottomPanel::bottom("footer")
                .frame(theme::footer_frame())
                .show(ctx, |ui| {
                    widgets::status_banner(ui, self.app.status());
                    ui.add_space(8.0);
                    if widgets::inject_button(ui, &self.app).clicked() {
                        self.app.start_injection();
                    }
                });
        }

        let panorama_time = ctx.input(|input| input.time);
        let speed = self.app.settings().panorama_speed as f64;
        let overlay = self.app.settings().dark_overlay;

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                paint_panorama(ui, &self.panoramas, panorama_time * speed, overlay);
                ScrollArea::vertical()
                    .id_salt("injector-content")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.with_layout(Layout::top_down(Align::Min), |ui| {
                            ui.set_max_width(ui.available_width().min(760.0));
                            egui::Frame::none()
                                .inner_margin(Margin::symmetric(12.0, 10.0))
                                .show(ui, |ui| match self.app.view() {
                                    AppView::Main => self.process_list(ui),
                                    AppView::Settings => {
                                        let folder = self.app.client_folder();
                                        widgets::settings_panel(
                                            ui,
                                            self.app.settings_mut(),
                                            folder.as_deref(),
                                        );
                                    }
                                    AppView::News => widgets::news_panel(ui),
                                });
                        });
                    });
            });

        ctx.request_repaint_after(Duration::from_millis(16));
        if self.app.is_busy() {
            ctx.request_repaint();
        }
    }
}

impl InjectorGui {
    fn process_list(&mut self, ui: &mut egui::Ui) {
        widgets::section_heading(ui);
        if self.app.processes().is_empty() {
            widgets::empty_state(ui);
            return;
        }

        let show_pid = self.app.settings().show_pid_in_cards;
        let selected = self.app.selected_pid();
        let mut clicked = None;
        for proc in self.app.processes() {
            let is_selected = selected == Some(proc.pid);
            if widgets::process_card(ui, proc, is_selected, show_pid).clicked() {
                clicked = Some(proc.pid);
            }
            ui.add_space(6.0);
        }

        if let Some(pid) = clicked {
            self.app.select(pid);
        }
    }
}

fn open_folder(path: &std::path::Path) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(path).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}

fn paint_panorama(ui: &mut egui::Ui, frames: &[TextureHandle], time: f64, overlay: f32) {
    let rect = ui.max_rect();

    let mut uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    if rect.width() > rect.height() {
        let visible_height = rect.height() / rect.width();
        uv.min.y = (1.0 - visible_height) * 0.5;
        uv.max.y = uv.min.y + visible_height;
    } else {
        let visible_width = rect.width() / rect.height();
        uv.min.x = (1.0 - visible_width) * 0.5;
        uv.max.x = uv.min.x + visible_width;
    }

    let drift = ((time * 0.012).sin() as f32) * 0.018;
    let max_shift = (uv.max.x - uv.min.x) * 0.04;
    let shift = drift.clamp(-max_shift, max_shift);
    uv.min.x = (uv.min.x + shift).max(0.0);
    uv.max.x = (uv.max.x + shift).min(1.0);

    let frame_count = frames.len().max(1);
    let cycle_seconds = PANORAMA_FRAME_SECONDS * frame_count as f64;
    let cycle_time = time.rem_euclid(cycle_seconds);
    let frame_position = cycle_time / PANORAMA_FRAME_SECONDS;
    let current = frame_position.floor() as usize % frame_count;
    let next = (current + 1) % frame_count;
    let frame_time = cycle_time % PANORAMA_FRAME_SECONDS;

    let fade_start = PANORAMA_FRAME_SECONDS - PANORAMA_FADE_SECONDS;
    let t = ((frame_time - fade_start) / PANORAMA_FADE_SECONDS).clamp(0.0, 1.0);
    let smooth = t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    let next_alpha = (smooth * 255.0).round() as u8;

    let painter = ui.painter();

    painter.image(frames[current].id(), rect, uv, Color32::WHITE);

    if next_alpha > 0 {
        painter.image(
            frames[next].id(),
            rect,
            uv,
            Color32::from_white_alpha(next_alpha),
        );
    }

    let alpha = (overlay.clamp(0.0, 1.0) * 255.0).round() as u8;
    painter.rect_filled(rect, 0.0, Color32::from_black_alpha(alpha));
}

fn load_texture(ctx: &Context, name: &str, bytes: &[u8]) -> TextureHandle {
    let image = image::load_from_memory(bytes)
        .unwrap_or_else(|error| panic!("failed to decode embedded {name} image: {error}"))
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let pixels = image.into_raw();
    ctx.load_texture(
        name,
        egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
        egui::TextureOptions::LINEAR,
    )
}
