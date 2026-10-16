#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod controller;
mod diagnostics;
mod theme;

use controller::{Controller, Navigation};
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use osfr_launcher::{LaunchPlan, LaunchSettings, LocalLoginStub, LoginProvider};
use std::{
    cell::Cell,
    fs::File,
    path::{Path, PathBuf},
    process::Child,
    rc::Rc,
    sync::mpsc,
    time::{Duration, Instant},
};

fn load_banner(ctx: &egui::Context) -> Result<egui::TextureHandle, String> {
    // (R) Fit bundled art to the current limit, including before the first frame.
    let limit = ctx.input(|input| input.max_texture_side) as u32;
    let image = image::load_from_memory(include_bytes!("../../docs/images/banner.png"))
        .map_err(|_| "The bundled banner could not be decoded.".to_owned())?
        .thumbnail(limit, limit)
        .into_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    Ok(ctx.load_texture(
        "Free Realms Expanded banner",
        egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw()),
        egui::TextureOptions::LINEAR,
    ))
}

fn save_capture(path: &Path, screenshot: &egui::ColorImage) -> Result<(), image::ImageError> {
    use image::ImageEncoder;
    let rgba: Vec<u8> = screenshot
        .pixels
        .iter()
        .flat_map(|pixel| pixel.to_array())
        .collect();
    // Exclusive creation also protects against a file appearing after CLI validation.
    let file = File::options().write(true).create_new(true).open(path)?;
    image::codecs::png::PngEncoder::new(file).write_image(
        &rgba,
        screenshot.width() as u32,
        screenshot.height() as u32,
        image::ExtendedColorType::Rgba8,
    )
}

struct Server {
    name: String,
    address: String,
}

struct Launcher {
    banner: egui::TextureHandle,
    servers: Vec<Server>,
    selected: usize,
    new_address: String,
    client: String,
    locale: String,
    settings: LaunchSettings,
    settings_page: bool,
    controller_on: bool,
    controller: Option<Controller>,
    controller_status: String,
    login: Option<Box<dyn LoginProvider + Send>>,
    pending: Option<mpsc::Receiver<(Box<dyn LoginProvider + Send>, Result<Child, String>)>>,
    child: Option<Child>,
    message: String,
    screenshot: Option<PathBuf>,
    screenshot_requested: bool,
    capture_ok: Rc<Cell<bool>>,
    started: Instant,
}

impl Launcher {
    fn new(
        cc: &eframe::CreationContext<'_>,
        screenshot: Option<PathBuf>,
        settings_page: bool,
        capture_ok: Rc<Cell<bool>>,
    ) -> Result<Self, String> {
        let banner = load_banner(&cc.egui_ctx)?;
        theme::install(&cc.egui_ctx);
        let (controller, controller_status) = match Controller::new() {
            Ok(controller) => (Some(controller), "Connect a controller to navigate.".into()),
            Err(error) => (None, error),
        };
        Ok(Self {
            banner,
            servers: vec![Server {
                name: "Local realm".into(),
                address: "127.0.0.1:20042".into(),
            }],
            selected: 0,
            new_address: String::new(),
            client: String::new(),
            locale: String::new(),
            settings: LaunchSettings::default(),
            settings_page,
            controller_on: true,
            controller,
            controller_status,
            login: Some(Box::new(LocalLoginStub)),
            pending: None,
            child: None,
            message: "Choose your client copy in Settings, then select a realm.".into(),
            screenshot,
            screenshot_requested: false,
            capture_ok,
            started: Instant::now(),
        })
    }

    fn play(&mut self) {
        // A screenshot run cannot prepare files or launch a child, even by pad.
        if self.screenshot.is_some() || self.pending.is_some() || self.child.is_some() {
            return;
        }
        if self.client.trim().is_empty() {
            self.settings_page = true;
            self.message = "Choose the client folder before playing.".into();
            return;
        }
        let Some(mut login) = self.login.take() else {
            return;
        };
        let client = PathBuf::from(&self.client);
        let locale = (!self.locale.trim().is_empty()).then(|| PathBuf::from(&self.locale));
        let address = self.servers[self.selected].address.clone();
        let settings = self.settings;
        let (sender, receiver) = mpsc::channel();
        self.pending = Some(receiver);
        self.message = "Preparing client files and backups…".into();
        // One bounded preparation job; keep the window responsive during locale I/O.
        std::thread::spawn(move || {
            let result = LaunchPlan::build(
                &client,
                &address,
                locale.as_deref(),
                settings,
                login.as_mut(),
            )
            .and_then(|plan| {
                plan.apply()?;
                plan.launch()
            });
            let _ = sender.send((login, result));
        });
    }

    fn navigation(&mut self, ctx: &egui::Context) {
        let focused = ctx.input(|i| i.focused);
        let actions = self
            .controller
            .as_mut()
            .map(|pad| pad.poll(self.controller_on, focused))
            .unwrap_or_default();
        if let Some(pad) = &self.controller {
            self.controller_status = if !self.controller_on {
                "Controller navigation off."
            } else if pad.connected {
                "Controller connected."
            } else {
                "No controller connected. Keyboard and mouse are ready."
            }
            .into();
        }
        for action in actions {
            match action {
                Navigation::Previous | Navigation::Next | Navigation::Left | Navigation::Right => {
                    let direction = match action {
                        Navigation::Previous => egui::FocusDirection::Previous,
                        Navigation::Next => egui::FocusDirection::Next,
                        Navigation::Left => egui::FocusDirection::Left,
                        _ => egui::FocusDirection::Right,
                    };
                    ctx.memory_mut(|memory| memory.move_focus(direction));
                }
                Navigation::Back => self.settings_page = false,
                Navigation::Settings => self.settings_page = !self.settings_page,
                Navigation::Confirm => {
                    // egui's existing Space activation handles the focused widget.
                    ctx.input_mut(|i| {
                        for pressed in [true, false] {
                            i.events.push(egui::Event::Key {
                                key: egui::Key::Space,
                                physical_key: None,
                                pressed,
                                repeat: false,
                                modifiers: egui::Modifiers::NONE,
                            });
                        }
                    });
                }
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.settings_page = false;
        }
    }

    fn home(&mut self, ui: &mut egui::Ui) {
        let compact = ui.ctx().content_rect().height() < 650.0;
        theme::heading(
            ui,
            "Your next adventure starts here",
            "Choose a realm, then jump into Free Realms.",
        );
        egui::Frame::new().fill(Color32::from_rgb(235, 231, 179)).corner_radius(12).inner_margin(if compact { 8 } else { 12 }).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Realms").font(theme::display(23.0)));
                ui.small("Status / players: not connected yet");
            });
            for (index, server) in self.servers.iter().enumerate() {
                let selected = index == self.selected;
                ui.horizontal(|ui| {
                    let mut label = egui::text::LayoutJob::default();
                    label.append(&format!("{}{}\n", if selected { "Selected: " } else { "" }, server.name), 0.0,
                        egui::TextFormat { font_id: theme::display(20.0), color: theme::INK, ..Default::default() });
                    label.append(&server.address, 0.0, egui::TextFormat {
                        font_id: egui::FontId::monospace(14.0), color: theme::INK, ..Default::default()
                    });
                    let row = ui.add(egui::Button::new(label).wrap()
                        .min_size(Vec2::new((ui.available_width() - 135.0).max(180.0), if compact { 44.0 } else { 52.0 }))
                        .fill(if selected { Color32::from_rgb(216, 235, 156) } else { theme::PAPER })
                        .stroke(Stroke::new(if selected { 2.0_f32 } else { 1.0_f32 },
                            if selected { Color32::from_rgb(85, 127, 30) } else { Color32::from_rgb(160, 146, 86) }))
                        .corner_radius(10));
                    if row.has_focus() {
                        ui.painter().rect_stroke(row.rect.expand(2.0), 12, Stroke::new(3.0_f32, theme::ORANGE), egui::StrokeKind::Outside);
                    }
                    if row.clicked() { self.selected = index; }
                    let (pip, _) = ui.allocate_exact_size(Vec2::splat(12.0), egui::Sense::hover());
                    ui.painter().circle_filled(pip.center(), 5.0, Color32::from_rgb(187, 116, 14));
                    ui.painter().circle_filled(pip.center() - Vec2::new(1.0, 1.5), 1.5, theme::PAPER);
                    ui.small("Status pending");
                });
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.new_address).hint_text("Add host:port").char_limit(261)
                    .desired_width((ui.available_width() - 145.0).max(160.0)));
                if theme::button(ui, "Add realm", Vec2::new(125.0, 38.0), self.servers.len() < 16, false).clicked() {
                    match osfr_launcher::validate_endpoint(&self.new_address) {
                        Ok(()) => {
                            self.servers.push(Server { name: "Custom realm".into(), address: self.new_address.clone() });
                            self.selected = self.servers.len() - 1;
                            self.new_address.clear();
                            self.message = "Realm added. Remote authentication is unavailable in this skeleton.".into();
                        }
                        Err(error) => self.message = error,
                    }
                }
            });
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let label = if self.child.is_some() {
                "GAME RUNNING"
            } else if self.pending.is_some() {
                "PREPARING…"
            } else {
                "PLAY NOW!"
            };
            if theme::button(
                ui,
                label,
                if compact {
                    Vec2::new(208.0, 56.0)
                } else {
                    Vec2::new(224.0, 64.0)
                },
                self.pending.is_none() && self.child.is_none() && self.screenshot.is_none(),
                false,
            )
            .clicked()
            {
                self.play();
            }
            ui.vertical(|ui| {
                ui.strong("Local login preview");
                ui.small("Loopback only · key-pair login pending (W83)");
                if self.screenshot.is_some() {
                    ui.small("Review capture: Play disabled.");
                }
            });
        });
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        theme::heading(
            ui,
            "Make yourself at home",
            "Use a separate, writable copy of your supplied client.",
        );
        ui.strong("Client folder");
        ui.add(
            egui::TextEdit::singleline(&mut self.client)
                .hint_text("Folder containing FreeRealms.exe")
                .char_limit(32768)
                .desired_width(f32::INFINITY),
        );
        ui.strong("Locale overlay folder (optional)");
        ui.add(
            egui::TextEdit::singleline(&mut self.locale)
                .hint_text("Folder with en_us_data.dat and en_us_data.dir")
                .char_limit(32768)
                .desired_width(f32::INFINITY),
        );
        ui.horizontal_wrapped(|ui| {
            ui.strong("Game window");
            ui.add(
                egui::DragValue::new(&mut self.settings.width)
                    .range(640..=3840)
                    .suffix(" px"),
            );
            ui.label("×");
            ui.add(
                egui::DragValue::new(&mut self.settings.height)
                    .range(480..=2160)
                    .suffix(" px"),
            );
            for (label, width, height) in [("1024 × 768", 1024, 768), ("1280 × 720", 1280, 720)] {
                if theme::button(ui, label, Vec2::new(134.0, 36.0), true, false).clicked() {
                    self.settings = LaunchSettings { width, height };
                }
            }
        });
        ui.checkbox(&mut self.controller_on, "Controller navigation").on_hover_text(
            "Launcher navigation only, using the existing layout's 0.25 deadzone. In-game controls use the separate Python bridge; this launcher does not start it."
        );
        ui.label(&self.controller_status);
        ui.small("Left stick / D-pad: move · A: select · B: back · LB: next · Start: settings");
        ui.separator();
        ui.small(
            "Settings are session-only. Play backs up configuration and replaced locale files.",
        )
        .on_hover_text(
            "Windowed mode, no maximization, muted audio: same defaults as the minimal launcher.",
        );
        if theme::button(ui, "Back to realms", Vec2::new(178.0, 40.0), true, false).clicked() {
            self.settings_page = false;
        }
    }
}

impl eframe::App for Launcher {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(33));
        self.navigation(ctx);
        if let Some(receiver) = &self.pending {
            match receiver.try_recv() {
                Ok((login, result)) => {
                    self.login = Some(login);
                    self.pending = None;
                    match result {
                        Ok(child) => {
                            self.child = Some(child);
                            self.message = "Client started. Backups are in its .friend-launcher-backups folder.".into();
                        }
                        Err(error) => self.message = error,
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.pending = None;
                    self.message =
                        "Client preparation stopped. Restart the launcher to try again.".into();
                }
                Err(mpsc::TryRecvError::Empty) => (),
            }
        }
        if let Some(child) = self.child.as_mut() {
            match child.try_wait() {
                Ok(Some(_)) => {
                    self.child = None;
                    self.message = "The client has closed.".into();
                }
                Err(_) => {
                    self.child = None;
                    self.message = "Could not read client status.".into();
                }
                Ok(None) => (),
            }
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::FOREST).inner_margin(16))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let width = ui.available_width().min(1080.0);
                    // Keep the whole banner, with space for controls on smaller windows.
                    ui.set_max_width(width);
                    let compact = ctx.content_rect().height() < 650.0;
                    if compact {
                        ui.spacing_mut().item_spacing.y = 4.0;
                    }
                    let height = (width / 3.0).min(if compact {
                        if self.settings_page { 64.0 } else { 80.0 }
                    } else if self.settings_page {
                        (ui.available_height() * 0.18).clamp(80.0, 145.0)
                    } else {
                        (ui.available_height() * 0.25).clamp(100.0, 230.0)
                    });
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.add(
                            egui::Image::new(&self.banner)
                                .fit_to_exact_size(Vec2::new(height * 3.0, height)),
                        );
                    });
                    ui.horizontal(|ui| {
                        if theme::button(
                            ui,
                            "Realms",
                            Vec2::new(126.0, 40.0),
                            true,
                            !self.settings_page,
                        )
                        .clicked()
                        {
                            self.settings_page = false;
                        }
                        if theme::button(
                            ui,
                            "Settings",
                            Vec2::new(126.0, 40.0),
                            true,
                            self.settings_page,
                        )
                        .clicked()
                        {
                            self.settings_page = true;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new("Launcher preview")
                                    .color(theme::PAPER)
                                    .size(13.0),
                            );
                        });
                    });
                    ui.add_space(4.0);
                    let panel = theme::panel()
                        .inner_margin(if compact { 12 } else { 16 })
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            if self.settings_page {
                                self.settings_ui(ui);
                            } else {
                                self.home(ui);
                            }
                            ui.add_space(12.0);
                            egui::Frame::new()
                                .fill(Color32::from_rgb(244, 225, 169))
                                .corner_radius(8)
                                .inner_margin(9)
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.label(RichText::new(&self.message).strong());
                                });
                        });
                    ui.painter().rect_stroke(
                        panel.response.rect.shrink(3.0),
                        15,
                        Stroke::new(1.0_f32, Color32::from_rgb(255, 253, 232)),
                        egui::StrokeKind::Inside,
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(
                            "A non-commercial fan revival · not affiliated with Daybreak or Sony.",
                        )
                        .size(13.0)
                        .color(Color32::from_rgb(220, 225, 182)),
                    );
                });
            });
        if let Some(path) = &self.screenshot {
            if !self.screenshot_requested && self.started.elapsed() > Duration::from_millis(600) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                self.screenshot_requested = true;
            }
            let screenshot = ctx.input(|i| {
                i.events.iter().find_map(|e| {
                    if let egui::Event::Screenshot { image, .. } = e {
                        Some(image.clone())
                    } else {
                        None
                    }
                })
            });
            if let Some(screenshot) = screenshot {
                match save_capture(path, &screenshot) {
                    Ok(()) => {
                        self.capture_ok.set(true);
                        log::info!(
                            "Screenshot saved: {}x{}",
                            screenshot.width(),
                            screenshot.height()
                        );
                    }
                    Err(error) => {
                        log::error!("Launcher screenshot failed: {error}");
                        eprintln!("Launcher screenshot failed: {error}");
                    }
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else if self.started.elapsed() > Duration::from_secs(15) {
                log::error!("Launcher screenshot timed out");
                eprintln!("Launcher screenshot timed out.");
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    diagnostics::init()?;
    let mut screenshot = None;
    let mut settings_page = false;
    let mut window_size = [940.0, 790.0];
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--screenshot" {
            screenshot = args.next().map(PathBuf::from);
            if screenshot.is_none() {
                return Err("--screenshot requires a PNG output path.".into());
            }
        } else if arg == "--settings-page" {
            settings_page = true;
        } else if arg == "--window-size" {
            let size = args
                .next()
                .and_then(|s| s.into_string().ok())
                .ok_or("--window-size requires WIDTHxHEIGHT in logical points.")?;
            let (width, height) = size
                .split_once('x')
                .ok_or("--window-size requires WIDTHxHEIGHT in logical points.")?;
            let settings = LaunchSettings {
                width: width.parse()?,
                height: height.parse()?,
            };
            settings.validate()?;
            window_size = [settings.width as f32, settings.height as f32];
        } else {
            return Err(
                "Usage: osfr-launcher [--settings-page] [--window-size WIDTHxHEIGHT] [--screenshot existing-folder/output.png]"
                    .into(),
            );
        }
    }
    if let Some(path) = &screenshot {
        if path.extension().is_none_or(|ext| ext != "png")
            || !path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."))
                .is_dir()
            || path.exists()
        {
            return Err("Screenshot needs an unused PNG path in an existing output folder.".into());
        }
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(window_size)
            .with_min_inner_size([640.0, 480.0])
            .with_title("Free Realms Expanded — Launcher"),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    let capture_requested = screenshot.is_some();
    let capture_ok = Rc::new(Cell::new(false));
    let captured = capture_ok.clone();
    let result = eframe::run_native(
        "Free Realms Expanded — Launcher",
        options,
        Box::new(move |cc| {
            Ok(Box::new(
                Launcher::new(cc, screenshot, settings_page, captured)
                    .map_err(std::io::Error::other)?,
            ))
        }),
    );
    if let Err(error) = &result {
        log::error!("run_native failed: {error}");
    } else {
        log::info!("run_native returned successfully");
    }
    result?;
    if capture_requested && !capture_ok.get() {
        return Err("Launcher closed without saving its screenshot.".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_banner_fits_startup_texture_limit() {
        // CreationContext still has egui's default limit before its first frame.
        let ctx = egui::Context::default();
        let banner = load_banner(&ctx).unwrap();
        let limit = ctx.input(|input| input.max_texture_side);
        let [width, height] = banner.size();
        assert!(width > 0 && height > 0 && width <= limit && height <= limit);
        assert!((width as f32 / height as f32 - 3.0).abs() < 0.01);
    }

    #[test]
    fn capture_refuses_existing_files() {
        let path = std::env::temp_dir().join(format!("osfr-capture-{}.png", std::process::id()));
        let _owned = File::options()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        let image = egui::ColorImage::filled([2, 2], Color32::GREEN);
        assert!(save_capture(&path, &image).is_err());
        assert_eq!(std::fs::metadata(&path).unwrap().len(), 0);
        drop(_owned);
        std::fs::remove_file(path).unwrap();
    }
}
