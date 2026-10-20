use egui::{
    pos2, Align2, Button, Color32, FontId, Frame, Layout, Margin, Rect, Response, RichText, Sense,
    Spinner, Stroke, TextureHandle, Ui, Vec2,
};

use super::theme::Palette;
use crate::app::{AppSettings, AppView, InjectionStatus, InjectorApp};
use crate::platform::ProcessInfo;

pub fn header(
    ui: &mut Ui,
    logo: &TextureHandle,
    github: &TextureHandle,
    discord: &TextureHandle,
    view: &AppView,
) -> Option<AppView> {
    let mut next_view = None;
    ui.vertical(|ui| {
        ui.horizontal(|ui| brand(ui, logo));
        ui.horizontal_wrapped(|ui| {
            if let Some(view) = navigation(ui, view) {
                next_view = Some(view);
            }
            ui.add_space(6.0);
            if social_button(ui, github, "Open HeliumClient on GitHub").clicked() {
                open_social_link("https://github.com/26140070-cpu/Helium-Client");
            }
            if social_button(ui, discord, "Join the HeliumClient Discord").clicked() {
                open_social_link("https://discord.gg/G6CwQHBJF6");
            }
        });
    });
    next_view
}

fn social_button(ui: &mut Ui, icon: &TextureHandle, tooltip: &str) -> egui::Response {
    ui.add(egui::Image::new((icon.id(), Vec2::splat(20.0))).sense(Sense::click()))
        .on_hover_text(tooltip)
}

fn open_social_link(url: &str) {
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("cmd.exe")
        .args(["/C", "start", "", url])
        .spawn();
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(url).spawn();
    #[cfg(target_os = "linux")]
    let result = std::process::Command::new("xdg-open").arg(url).spawn();

    if let Err(error) = result {
        log::error!("could not open social link {url}: {error}");
    }
}

fn brand(ui: &mut Ui, logo: &TextureHandle) {
    ui.horizontal(|ui| {
        let (logo_rect, _) = ui.allocate_exact_size(Vec2::splat(42.0), Sense::hover());
        ui.painter()
            .rect_filled(logo_rect, 12.0, Palette::accent_soft());
        ui.painter().image(
            logo.id(),
            logo_rect.shrink(2.0),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("HeliumClient")
                        .size(21.0)
                        .strong()
                        .color(Palette::TEXT),
                );
                ui.label(
                    RichText::new("INJECTOR")
                        .size(10.0)
                        .strong()
                        .color(Palette::ACCENT),
                );
            });
            ui.label(
                RichText::new("Minecraft Java Edition")
                    .size(11.0)
                    .color(Palette::TEXT_DIM),
            );
        });
    });
}

fn navigation(ui: &mut Ui, view: &AppView) -> Option<AppView> {
    let mut next_view = None;
    if nav_button(ui, "News", view == &AppView::News).clicked() {
        next_view = Some(if *view == AppView::News {
            AppView::Main
        } else {
            AppView::News
        });
    }
    ui.add_space(4.0);
    if nav_button(ui, "Settings", view == &AppView::Settings).clicked() {
        next_view = Some(if *view == AppView::Settings {
            AppView::Main
        } else {
            AppView::Settings
        });
    }
    next_view
}

fn nav_button(ui: &mut Ui, icon: &str, active: bool) -> Response {
    let fill = if active {
        Palette::ACCENT
    } else {
        Palette::card()
    };
    let text_color = if active {
        Color32::WHITE
    } else {
        Palette::TEXT
    };
    let button = Button::new(RichText::new(icon).size(13.0).color(text_color))
        .fill(fill)
        .min_size(Vec2::new(82.0, 32.0));
    ui.add(button)
}

pub fn tool_button(ui: &mut Ui, label: &str, enabled: bool) -> Response {
    let button = Button::new(RichText::new(label).color(Palette::TEXT))
        .fill(Palette::card())
        .min_size(Vec2::new(0.0, 32.0));
    ui.add_enabled(enabled, button)
}

pub fn process_card(ui: &mut Ui, proc: &ProcessInfo, selected: bool, show_pid: bool) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 56.0), Sense::click());

    let bg = if selected {
        Palette::accent_soft()
    } else if response.hovered() {
        Palette::card_hover()
    } else {
        Palette::card()
    };

    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, 10.0, bg);
    if selected {
        painter.rect_stroke(rect, 10.0, Stroke::new(1.5_f32, Palette::ACCENT));
    }

    let dot_color = if selected {
        Palette::ACCENT
    } else {
        Palette::TEXT_DIM
    };
    painter.circle_filled(rect.left_center() + Vec2::new(18.0, 0.0), 4.0, dot_color);

    let text_x = rect.left() + 34.0;
    let title = if show_pid {
        format!("PID {}", proc.pid)
    } else {
        proc.info.clone()
    };
    let subtitle = if show_pid {
        proc.info.as_str()
    } else {
        &format!("PID {}", proc.pid)
    };

    painter.text(
        pos2(text_x, rect.center().y - 9.0),
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(15.0),
        Palette::TEXT,
    );
    painter.text(
        pos2(text_x, rect.center().y + 9.0),
        Align2::LEFT_CENTER,
        subtitle,
        FontId::proportional(12.0),
        Palette::TEXT_DIM,
    );

    response
}

pub fn empty_state(ui: &mut Ui) {
    ui.add_space(24.0);
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new("No Minecraft instances found")
                .size(14.0)
                .color(Palette::TEXT_DIM),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new("Start Minecraft and scan again.")
                .size(12.0)
                .color(Palette::TEXT_DIM),
        );
    });
}

pub fn section_heading(ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                RichText::new("Game instances")
                    .size(18.0)
                    .strong()
                    .color(Palette::TEXT),
            );
            ui.label(
                RichText::new("Select a running Minecraft process")
                    .size(11.0)
                    .color(Palette::TEXT_DIM),
            );
        });
    });
}

pub fn status_banner(ui: &mut Ui, status: &InjectionStatus) {
    let (color, icon) = match status {
        InjectionStatus::Idle => (Palette::TEXT_DIM, "●"),
        InjectionStatus::Scanning => (Palette::ACCENT, "◌"),
        InjectionStatus::Injecting(_) => (Palette::WARN, "◌"),
        InjectionStatus::Done(_) => (Palette::OK, "✔"),
        InjectionStatus::Failed(_) => (Palette::ERR, "✖"),
    };
    let busy = matches!(
        status,
        InjectionStatus::Scanning | InjectionStatus::Injecting(_)
    );

    Frame::none()
        .fill(Palette::bg())
        .rounding(8.0)
        .inner_margin(Margin::symmetric(12.0, 10.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if busy {
                    ui.add(Spinner::new().size(14.0).color(color));
                } else {
                    ui.label(RichText::new(icon).size(14.0).color(color));
                }
                ui.label(RichText::new(status.message()).color(Palette::TEXT));
            });
        });
}

pub fn inject_button(ui: &mut Ui, app: &InjectorApp) -> Response {
    let enabled = app.selected_pid().is_some() && !app.is_busy();
    let label = if app.is_busy() {
        "Injecting…"
    } else {
        "Inject Client"
    };
    let button = Button::new(
        RichText::new(label)
            .size(15.0)
            .strong()
            .color(Color32::WHITE),
    )
    .fill(Palette::ACCENT)
    .min_size(Vec2::new(ui.available_width(), 42.0));
    ui.add_enabled(enabled, button)
}

pub fn settings_panel(
    ui: &mut Ui,
    settings: &mut AppSettings,
    client_folder: Option<&std::path::Path>,
) {
    let compact = ui.available_width() < 440.0;
    ui.add_space(8.0);
    ui.label(
        RichText::new("Settings")
            .size(20.0)
            .strong()
            .color(Palette::TEXT),
    );
    ui.label(
        RichText::new("Changes are saved automatically")
            .size(12.0)
            .color(Palette::TEXT_DIM),
    );
    ui.add_space(16.0);

    Frame::none()
        .fill(Palette::card())
        .rounding(12.0)
        .inner_margin(Margin::same(16.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new("Behaviour")
                    .size(13.0)
                    .strong()
                    .color(Palette::ACCENT),
            );
            ui.add_space(8.0);

            ui.checkbox(
                &mut settings.auto_scan_on_start,
                RichText::new("Auto-scan on launch").color(Palette::TEXT),
            );
            ui.checkbox(
                &mut settings.auto_select_first,
                RichText::new("Auto-select first process").color(Palette::TEXT),
            );
            ui.checkbox(
                &mut settings.show_pid_in_cards,
                RichText::new("Show PID in process cards").color(Palette::TEXT),
            );
        });

    ui.add_space(12.0);

    Frame::none()
        .fill(Palette::card())
        .rounding(12.0)
        .inner_margin(Margin::same(16.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new("Panorama")
                    .size(13.0)
                    .strong()
                    .color(Palette::ACCENT),
            );
            ui.add_space(8.0);

            if compact {
                ui.label(RichText::new("Speed").color(Palette::TEXT_DIM));
                ui.add(
                    egui::Slider::new(&mut settings.panorama_speed, 0.4..=2.5)
                        .show_value(true)
                        .suffix("×"),
                );
                ui.label(RichText::new("Overlay").color(Palette::TEXT_DIM));
                ui.add(egui::Slider::new(&mut settings.dark_overlay, 0.15..=0.75).show_value(true));
            } else {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Speed").color(Palette::TEXT_DIM));
                    ui.add(
                        egui::Slider::new(&mut settings.panorama_speed, 0.4..=2.5)
                            .show_value(true)
                            .suffix("×"),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Overlay").color(Palette::TEXT_DIM));
                    ui.add(
                        egui::Slider::new(&mut settings.dark_overlay, 0.15..=0.75).show_value(true),
                    );
                });
            }
        });

    ui.add_space(12.0);

    Frame::none()
        .fill(Palette::card())
        .rounding(12.0)
        .inner_margin(Margin::same(16.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new("Client Folder")
                    .size(13.0)
                    .strong()
                    .color(Palette::ACCENT),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new("Folder that contains libagent_loader & libclient")
                    .size(11.0)
                    .color(Palette::TEXT_DIM),
            );
            ui.add_space(8.0);

            if let Some(path) = client_folder {
                let path_str = path.display().to_string();
                ui.label(
                    RichText::new(&path_str)
                        .size(12.0)
                        .color(Palette::TEXT)
                        .monospace(),
                );
                ui.add_space(8.0);
                if tool_button(ui, "📂  Open Client Folder", true).clicked() {
                    open_folder(path);
                }
            } else {
                ui.label(RichText::new("Could not resolve client folder").color(Palette::ERR));
            }
        });
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

struct ChangelogEntry {
    version: &'static str,
    date: &'static str,
    tag: &'static str,
    items: &'static [&'static str],
}

const CHANGELOG: &[ChangelogEntry] = &[ChangelogEntry {
    version: "1.0.0",
    date: "Public Release",
    tag: "NEW",
    items: &[
        "First public release: GUI, TUI and headless injector modes",
        "Windows DLL injection, Linux SO injection and MacOS DYLIB injection",
        "Responsive injector UI with animated panorama, custom font and saved settings",
        "Friends with middle-click management, AutoEat and improved AutoTotem",
        "Surround, Hole ESP, XRay and Blink",
        "Named client configuration profiles",
        "Thank you for downloading Helium Client with love, Dona",
    ],
}];

pub fn news_panel(ui: &mut Ui) {
    ui.add_space(8.0);
    ui.label(
        RichText::new("Changelogs & News")
            .size(20.0)
            .strong()
            .color(Palette::TEXT),
    );
    ui.label(
        RichText::new("What's new in HeliumClient Injector")
            .size(12.0)
            .color(Palette::TEXT_DIM),
    );
    ui.add_space(14.0);

    for entry in CHANGELOG {
        changelog_card(ui, entry);
        ui.add_space(10.0);
    }
}

fn changelog_card(ui: &mut Ui, entry: &ChangelogEntry) {
    let tag_color = match entry.tag {
        "NEW" => Palette::OK,
        "FIX" => Palette::WARN,
        "IMP" => Palette::ACCENT,
        _ => Palette::TEXT_DIM,
    };

    Frame::none()
        .fill(Palette::card())
        .rounding(12.0)
        .inner_margin(Margin::same(14.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("v{}", entry.version))
                        .size(15.0)
                        .strong()
                        .color(Palette::TEXT),
                );
                ui.label(
                    RichText::new(entry.tag)
                        .size(11.0)
                        .strong()
                        .color(tag_color),
                );
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(entry.date)
                            .size(11.0)
                            .color(Palette::TEXT_DIM),
                    );
                });
            });
            ui.add_space(6.0);
            for item in entry.items {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("•").color(Palette::ACCENT));
                    ui.label(RichText::new(*item).size(13.0).color(Palette::TEXT));
                });
            }
        });
}
