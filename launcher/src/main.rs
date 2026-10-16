#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod controller;

use controller::{Controller, Navigation};
use eframe::egui::{self, Color32, FontId, RichText, Stroke, Vec2};
use osfr_launcher::{LaunchPlan, LaunchSettings, LocalLoginStub, LoginProvider};
use std::{
    path::{Path, PathBuf},
    process::Child,
    sync::mpsc,
    time::{Duration, Instant},
};

const GREEN: Color32 = Color32::from_rgb(77, 145, 27);
const INK: Color32 = Color32::from_rgb(36, 57, 28);
const CREAM: Color32 = Color32::from_rgb(247, 240, 214);

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
    started: Instant,
}

impl Launcher {
    fn new(
        cc: &eframe::CreationContext<'_>,
        screenshot: Option<PathBuf>,
        settings_page: bool,
    ) -> Result<Self, String> {
        let image = image::load_from_memory(include_bytes!("../../docs/images/banner.png"))
            .map_err(|_| "The bundled banner could not be decoded.".to_owned())?
            .into_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let banner = cc.egui_ctx.load_texture(
            "Free Realms Expanded banner",
            egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw()),
            egui::TextureOptions::LINEAR,
        );
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::light();
        style.visuals.override_text_color = Some(INK);
        style.visuals.panel_fill = CREAM;
        style.visuals.window_fill = CREAM;
        style.visuals.selection.bg_fill = GREEN;
        style.visuals.selection.stroke = Stroke::new(2.0, Color32::WHITE);
        style.visuals.widgets.active.bg_stroke = Stroke::new(2.0, GREEN);
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(2.0, GREEN);
        style.spacing.item_spacing = Vec2::new(12.0, 10.0);
        style.spacing.button_padding = Vec2::new(18.0, 10.0);
        style
            .text_styles
            .insert(egui::TextStyle::Body, FontId::proportional(17.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, FontId::proportional(18.0));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, FontId::proportional(27.0));
        cc.egui_ctx.set_style(style);
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
        ui.heading("Pick a realm. Start an adventure.");
        ui.label("Your supplied client, our local revival.");
        ui.add_space(7.0);
        egui::Frame::new().fill(Color32::from_rgb(225, 231, 192)).stroke(Stroke::new(2.0, Color32::from_rgb(132, 153, 82))).corner_radius(10).inner_margin(16).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| { ui.strong("REALMS"); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.small("Status / players: not connected yet"); }); });
            ui.separator();
            for (index, server) in self.servers.iter().enumerate() {
                ui.horizontal(|ui| {
                    if ui.selectable_label(index == self.selected, RichText::new(&server.name).strong()).clicked() { self.selected = index; }
                    ui.monospace(&server.address);
                    ui.label(RichText::new("● Status pending").color(Color32::from_rgb(108, 94, 39)));
                });
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.new_address).hint_text("Add host:port").char_limit(261).desired_width(220.0));
                if ui.add_enabled(self.servers.len() < 16, egui::Button::new("Add realm")).clicked() {
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
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let play = egui::Button::new(
                RichText::new(if self.child.is_some() {
                    "GAME RUNNING"
                } else {
                    "PLAY"
                })
                .size(30.0)
                .strong()
                .color(Color32::WHITE),
            )
            .fill(GREEN)
            .stroke(Stroke::new(3.0, Color32::from_rgb(173, 209, 68)))
            .corner_radius(16)
            .min_size(Vec2::new(220.0, 64.0));
            if ui
                .add_enabled(
                    self.pending.is_none() && self.child.is_none() && self.screenshot.is_none(),
                    play,
                )
                .clicked()
            {
                self.play();
            }
            ui.vertical(|ui| {
                ui.strong("Login: local stub · W83 pending");
                ui.small("Literal loopback only. No keys or passwords collected.");
                if self.screenshot.is_some() {
                    ui.small("Review capture: Play disabled.");
                }
            });
        });
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Make yourself at home");
        ui.label("Use a separate, writable copy of your supplied client.");
        ui.add_space(6.0);
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
        ui.horizontal(|ui| {
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
                if ui.button(label).clicked() {
                    self.settings = LaunchSettings { width, height };
                }
            }
        });
        ui.checkbox(&mut self.controller_on, "Controller navigation");
        ui.label(&self.controller_status);
        ui.small("Left stick / D-pad: move · A: select · B: back · LB: next · Start: settings");
        ui.small("The existing gamepad layout supplies the 0.25 deadzone and button roles.");
        ui.small("In-game controller input still uses the separate Python bridge; this skeleton does not start it.");
        ui.separator();
        ui.small("Settings last for this launcher session. Play backs up configuration and any replaced locale files.");
        ui.small(
            "Windowed mode, no maximization, muted audio: same defaults as the minimal launcher.",
        );
        if ui.button("Back to realms").clicked() {
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
            .frame(egui::Frame::new().fill(CREAM).inner_margin(22))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let width = ui.available_width();
                    ui.add(
                        egui::Image::new(&self.banner)
                            .fit_to_exact_size(Vec2::new(width, width / 3.0)),
                    );
                    ui.horizontal(|ui| {
                        if ui.selectable_label(!self.settings_page, "Realms").clicked() {
                            self.settings_page = false;
                        }
                        if ui
                            .selectable_label(self.settings_page, "Settings")
                            .clicked()
                        {
                            self.settings_page = true;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.small("W86 · launcher preview");
                        });
                    });
                    ui.separator();
                    if self.settings_page {
                        self.settings_ui(ui);
                    } else {
                        self.home(ui);
                    }
                    ui.add_space(12.0);
                    ui.label(RichText::new(&self.message).strong());
                    ui.separator();
                    ui.small(
                        "A non-commercial fan revival · not affiliated with Daybreak or Sony.",
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
                let rgba: Vec<u8> = screenshot
                    .pixels
                    .iter()
                    .flat_map(|pixel| pixel.to_array())
                    .collect();
                if let Err(error) = image::save_buffer(
                    path,
                    &rgba,
                    screenshot.width() as u32,
                    screenshot.height() as u32,
                    image::ColorType::Rgba8,
                ) {
                    eprintln!("Launcher screenshot failed: {error}");
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else if self.started.elapsed() > Duration::from_secs(15) {
                eprintln!("Launcher screenshot timed out.");
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

fn main() -> eframe::Result {
    let mut screenshot = None;
    let mut settings_page = false;
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--screenshot" {
            screenshot = args.next().map(PathBuf::from);
            if screenshot.is_none() {
                eprintln!("--screenshot requires a PNG output path.");
                return Ok(());
            }
        } else if arg == "--settings-page" {
            settings_page = true;
        } else {
            eprintln!(
                "Usage: osfr-launcher [--settings-page] [--screenshot existing-folder/output.png]"
            );
            return Ok(());
        }
    }
    if let Some(path) = &screenshot {
        if path.extension().is_none_or(|ext| ext != "png")
            || !path.parent().unwrap_or(Path::new(".")).is_dir()
            || path.exists()
        {
            eprintln!("Screenshot needs an unused PNG path in an existing output folder.");
            return Ok(());
        }
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([940.0, 790.0])
            .with_min_inner_size([720.0, 620.0])
            .with_title("Free Realms Expanded — Launcher"),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Free Realms Expanded — Launcher",
        options,
        Box::new(move |cc| {
            Ok(Box::new(
                Launcher::new(cc, screenshot, settings_page).map_err(std::io::Error::other)?,
            ))
        }),
    )
}
