#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod controller;
mod diagnostics;
mod theme;

use controller::{Controller, Navigation};
use eframe::egui::{self, Color32, RichText, Vec2};
use osfr_launcher::{
    LaunchPlan, LaunchSettings, LocalLoginStub, LoginProvider, SessionToken,
    auth::{self, AuthError},
};
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
    profile_id: String,
}

struct RealmLogin {
    profile_id: String,
}

impl RealmLogin {
    fn session_with(
        &self,
        endpoint: &str,
        load_profile: impl FnOnce(&str) -> Result<auth::Profile, AuthError>,
        authenticate: impl FnOnce(&auth::Profile) -> Result<SessionToken, AuthError>,
    ) -> osfr_launcher::Result<SessionToken> {
        if self.profile_id.trim().is_empty() {
            return LocalLoginStub.session(endpoint);
        }
        let profile = load_profile(self.profile_id.trim()).map_err(auth_error_message)?;
        login_profile(&profile, authenticate)
    }
}

impl LoginProvider for RealmLogin {
    fn session(&mut self, endpoint: &str) -> osfr_launcher::Result<SessionToken> {
        self.session_with(endpoint, auth::load_profile, auth::login)
    }
}

fn auth_error_message(error: AuthError) -> String {
    error.to_string()
}

fn login_profile(
    profile: &auth::Profile,
    authenticate: impl FnOnce(&auth::Profile) -> Result<SessionToken, AuthError>,
) -> osfr_launcher::Result<SessionToken> {
    authenticate(profile).map_err(auth_error_message)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FocusTarget {
    RealmsTab,
    SettingsTab,
    RealmRow(usize),
    AddAddress,
    AddRealm,
    LoginProfile,
    Play,
    ClientFolder,
    LocaleFolder,
    WindowWidth,
    WindowHeight,
    ResolutionSmall,
    ResolutionLarge,
    ControllerToggle,
    Back,
}

fn focus_order(settings: bool, realm_count: usize) -> Vec<FocusTarget> {
    let mut order = vec![FocusTarget::RealmsTab, FocusTarget::SettingsTab];
    if settings {
        order.extend([
            FocusTarget::ClientFolder,
            FocusTarget::LocaleFolder,
            FocusTarget::WindowWidth,
            FocusTarget::WindowHeight,
            FocusTarget::ResolutionSmall,
            FocusTarget::ResolutionLarge,
            FocusTarget::ControllerToggle,
            FocusTarget::Back,
        ]);
    } else {
        order.extend((0..realm_count).map(FocusTarget::RealmRow));
        order.extend([
            FocusTarget::AddAddress,
            FocusTarget::AddRealm,
            FocusTarget::Play,
            FocusTarget::LoginProfile,
        ]);
    }
    order
}

fn focus_sequence_matches(actual: &[FocusTarget], settings: bool, realm_count: usize) -> bool {
    actual == focus_order(settings, realm_count)
}

fn next_focus_index(current: Option<usize>, len: usize, forward: bool) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(match current {
        Some(index) if forward => (index + 1) % len,
        Some(0) if !forward => len - 1,
        Some(index) => index - 1,
        None if forward => 0,
        None => len - 1,
    })
}

struct Launcher {
    banner: egui::TextureHandle,
    art: theme::UiArt,
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
    pending: Option<mpsc::Receiver<Result<Child, String>>>,
    child: Option<Child>,
    message: String,
    screenshot: Option<PathBuf>,
    screenshot_requested: bool,
    capture_ok: Rc<Cell<bool>>,
    started: Instant,
    focus_targets: Vec<(FocusTarget, egui::Id)>,
    focus_play_on_start: bool,
    focus_play_requested: bool,
    test_vine_growth: Option<f32>,
}

impl Launcher {
    fn new(
        cc: &eframe::CreationContext<'_>,
        screenshot: Option<PathBuf>,
        settings_page: bool,
        capture_ok: Rc<Cell<bool>>,
        focus_play_on_start: bool,
        test_vine_growth: Option<f32>,
    ) -> Result<Self, String> {
        let banner = load_banner(&cc.egui_ctx)?;
        theme::install(&cc.egui_ctx);
        let art = theme::UiArt::load(&cc.egui_ctx)?;
        let (controller, controller_status) = match Controller::new() {
            Ok(controller) => (Some(controller), "Connect a controller to navigate.".into()),
            Err(error) => (None, error),
        };
        Ok(Self {
            banner,
            art,
            servers: vec![Server {
                name: "Local realm".into(),
                address: "127.0.0.1:20042".into(),
                profile_id: String::new(),
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
            pending: None,
            child: None,
            message: "Choose your client copy in Settings, then select a realm.".into(),
            screenshot,
            screenshot_requested: false,
            capture_ok,
            started: Instant::now(),
            focus_targets: Vec::new(),
            focus_play_on_start,
            focus_play_requested: false,
            test_vine_growth,
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
        let client = PathBuf::from(&self.client);
        let locale = (!self.locale.trim().is_empty()).then(|| PathBuf::from(&self.locale));
        let server = &self.servers[self.selected];
        let address = server.address.clone();
        let profile_id = server.profile_id.trim().to_owned();
        let settings = self.settings;
        let (sender, receiver) = mpsc::channel();
        self.pending = Some(receiver);
        self.message = "Preparing client files and backups...".into();
        // One bounded preparation job; keep the window responsive during locale I/O.
        std::thread::spawn(move || {
            let mut login: Box<dyn LoginProvider + Send> = Box::new(RealmLogin { profile_id });
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
            let _ = sender.send(result);
        });
    }

    fn record_focus(&mut self, target: FocusTarget, response: &egui::Response) {
        self.focus_targets.push((target, response.id));
    }

    fn focus_target(&self, ctx: &egui::Context, target: FocusTarget) {
        if let Some((_, id)) = self.focus_targets.iter().find(|(item, _)| *item == target) {
            ctx.memory_mut(|memory| memory.request_focus(*id));
        }
    }

    fn move_controller_focus(&self, ctx: &egui::Context, forward: bool) {
        let current = ctx.memory(|memory| {
            self.focus_targets
                .iter()
                .position(|(_, id)| memory.has_focus(*id))
        });
        if let Some(next) = next_focus_index(current, self.focus_targets.len(), forward) {
            let id = self.focus_targets[next].1;
            ctx.memory_mut(|memory| memory.request_focus(id));
        }
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
                    self.move_controller_focus(
                        ctx,
                        matches!(action, Navigation::Next | Navigation::Right),
                    );
                }
                Navigation::Back => {
                    self.settings_page = false;
                    self.focus_target(ctx, FocusTarget::RealmsTab);
                }
                Navigation::Settings => {
                    self.settings_page = !self.settings_page;
                    self.focus_target(
                        ctx,
                        if self.settings_page {
                            FocusTarget::SettingsTab
                        } else {
                            FocusTarget::RealmsTab
                        },
                    );
                }
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
        ui.horizontal(|ui| {
            let (icon, _) = ui.allocate_exact_size(Vec2::splat(27.0), egui::Sense::hover());
            self.art.icon(ui.painter(), 0, icon);
            ui.label(RichText::new("Realms").font(theme::display(22.0)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.small("Status / players: not connected yet");
            });
        });
        let row_height = if compact { 56.0 } else { 60.0 };
        let row_width = ui.available_width();
        for index in 0..self.servers.len() {
            let server = &self.servers[index];
            let name = if index == self.selected {
                format!("Selected: {}", server.name)
            } else {
                server.name.clone()
            };
            let address = server.address.clone();
            let row = theme::row_button(
                ui,
                &self.art,
                &name,
                &address,
                Vec2::new(row_width, row_height),
            );
            self.record_focus(FocusTarget::RealmRow(index), &row);
            if row.clicked() {
                self.selected = index;
            }
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let add_width = 142.0;
            let field_width = (ui.available_width() - add_width - 8.0).max(160.0);
            let field = theme::text_field(
                ui,
                &self.art,
                &mut self.new_address,
                "Add host:port",
                Vec2::new(field_width, 48.0),
                261,
            );
            self.record_focus(FocusTarget::AddAddress, &field);
            let add = theme::button(
                ui,
                &self.art,
                "Add realm",
                Vec2::new(add_width, 48.0),
                self.servers.len() < 16,
                theme::ButtonSkin::Standard,
            );
            self.record_focus(FocusTarget::AddRealm, &add);
            if add.clicked() {
                match osfr_launcher::validate_endpoint(&self.new_address) {
                    Ok(()) => {
                        self.servers.push(Server {
                            name: "Custom realm".into(),
                            address: self.new_address.clone(),
                            profile_id: String::new(),
                        });
                        self.selected = self.servers.len() - 1;
                        self.new_address.clear();
                        self.message = "Realm added. Set a stored key-pair profile ID or use local loopback login.".into();
                    }
                    Err(error) => self.message = error,
                }
            }
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let label = if self.child.is_some() {
                "GAME RUNNING"
            } else if self.pending.is_some() {
                "PREPARING..."
            } else {
                "PLAY NOW!"
            };
            let play = theme::button(
                ui,
                &self.art,
                label,
                if compact {
                    Vec2::new(208.0, 56.0)
                } else {
                    Vec2::new(224.0, 64.0)
                },
                self.pending.is_none()
                    && self.child.is_none()
                    && (self.screenshot.is_none() || self.focus_play_on_start),
                theme::ButtonSkin::Play,
            );
            self.record_focus(FocusTarget::Play, &play);
            if self.focus_play_on_start && !self.focus_play_requested {
                ui.ctx().memory_mut(|memory| memory.request_focus(play.id));
                self.focus_play_requested = true;
            }
            if self.focus_play_on_start && !play.has_focus() {
                for shape in theme::focus_shapes(play.rect) {
                    ui.painter().add(shape);
                }
            }
            if play.clicked() {
                self.play();
            }
            ui.vertical(|ui| {
                ui.label(RichText::new("Login profile ID").strong());
                let profile = theme::text_field(
                    ui,
                    &self.art,
                    &mut self.servers[self.selected].profile_id,
                    "Optional; blank uses local login",
                    Vec2::new(ui.available_width().max(220.0), 38.0),
                    64,
                );
                self.record_focus(FocusTarget::LoginProfile, &profile);
                if self.servers[self.selected].profile_id.trim().is_empty() {
                    ui.small("Local compatibility login; loopback only.");
                } else {
                    ui.small("Stored key-pair profile; broker login on Play.");
                }
            });
        });
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        let compact = ui.ctx().content_rect().height() < 650.0;
        theme::heading(
            ui,
            "Make yourself at home",
            "Use a separate, writable copy of your supplied client.",
        );
        if !compact {
            ui.label(RichText::new("Client folder").strong());
        }
        let client = theme::text_field(
            ui,
            &self.art,
            &mut self.client,
            "Folder containing FreeRealms.exe",
            Vec2::new(ui.available_width(), 48.0),
            32768,
        );
        self.record_focus(FocusTarget::ClientFolder, &client);
        if !compact {
            ui.label(RichText::new("Locale overlay folder (optional)").strong());
        }
        let locale = theme::text_field(
            ui,
            &self.art,
            &mut self.locale,
            "Folder with en_us_data.dat and en_us_data.dir",
            Vec2::new(ui.available_width(), 48.0),
            32768,
        );
        self.record_focus(FocusTarget::LocaleFolder, &locale);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Game window").strong());
            ui.add_space(3.0);
            let width = ui.add_sized(
                Vec2::new(126.0, 44.0),
                egui::DragValue::new(&mut self.settings.width)
                    .range(640..=3840)
                    .suffix(" px"),
            );
            self.record_focus(FocusTarget::WindowWidth, &width);
            theme::focus_outline(ui, &width);
            ui.label("x");
            let height = ui.add_sized(
                Vec2::new(126.0, 44.0),
                egui::DragValue::new(&mut self.settings.height)
                    .range(480..=2160)
                    .suffix(" px"),
            );
            self.record_focus(FocusTarget::WindowHeight, &height);
            theme::focus_outline(ui, &height);
            for (target, label, width, height) in [
                (FocusTarget::ResolutionSmall, "1024 x 768", 1024, 768),
                (FocusTarget::ResolutionLarge, "1280 x 720", 1280, 720),
            ] {
                let preset = theme::button(
                    ui,
                    &self.art,
                    label,
                    Vec2::new(130.0, 44.0),
                    true,
                    theme::ButtonSkin::Standard,
                );
                self.record_focus(target, &preset);
                if preset.clicked() {
                    self.settings = LaunchSettings { width, height };
                }
            }
        });
        let controller = ui
            .add_sized(
                Vec2::new(ui.available_width(), 44.0),
                egui::Checkbox::new(&mut self.controller_on, "Controller navigation"),
            )
            .on_hover_text(
                "Launcher navigation only, using the existing layout's 0.25 deadzone. In-game controls use the separate Python bridge; this launcher does not start it.",
            );
        self.record_focus(FocusTarget::ControllerToggle, &controller);
        theme::focus_outline(ui, &controller);
        if compact {
            ui.small(format!(
                "{} | D-pad/stick: move | A: select | B: back | LB: next | Start: settings",
                self.controller_status
            ));
            ui.small("Settings are session-only; Play keeps configuration and locale backups.");
        } else {
            ui.label(&self.controller_status);
            ui.small("Left stick / D-pad: move | A: select | B: back | LB: next | Start: settings");
            ui.small(
                "Settings are session-only. Play backs up configuration and replaced locale files.",
            );
        }
        let back = theme::button(
            ui,
            &self.art,
            "Back to realms",
            Vec2::new(178.0, 46.0),
            true,
            theme::ButtonSkin::Standard,
        );
        self.record_focus(FocusTarget::Back, &back);
        if back.clicked() {
            self.settings_page = false;
        }
    }
}

impl eframe::App for Launcher {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let elapsed = self.started.elapsed();
        let t = self
            .test_vine_growth
            .unwrap_or_else(|| (elapsed.as_secs_f32() / 1.5).clamp(0.0, 1.0));
        let growth = 1.0 - (1.0 - t).powi(3);
        self.art.repaint_vines(growth);
        let growing = self.test_vine_growth.is_none() && elapsed < Duration::from_millis(1500);
        let capture_waiting = self.screenshot.is_some() && !self.screenshot_requested;
        if self.controller_on || self.pending.is_some() || growing || capture_waiting {
            ctx.request_repaint_after(Duration::from_millis(33));
        }
        self.navigation(ctx);
        if let Some(receiver) = &self.pending {
            match receiver.try_recv() {
                Ok(result) => {
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
        let rendered_settings = self.settings_page;
        let rendered_realms = self.servers.len();
        self.focus_targets.clear();
        self.art.draw_backdrop(
            &ctx.layer_painter(egui::LayerId::background()),
            ctx.content_rect(),
        );
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::TRANSPARENT).inner_margin(10))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    let width = ui.available_width().min(1080.0);
                    ui.set_max_width(width);
                    let compact = ctx.content_rect().height() < 650.0;
                    if compact {
                        ui.spacing_mut().item_spacing.y = 4.0;
                    }
                    let banner_height = (width / 3.0).min(if compact {
                        if rendered_settings { 34.0 } else { 82.0 }
                    } else if rendered_settings {
                        118.0
                    } else {
                        220.0
                    });
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        let banner_rect = ui
                            .allocate_exact_size(
                                Vec2::new(banner_height * 3.0, banner_height),
                                egui::Sense::hover(),
                            )
                            .0;
                        ui.painter().image(
                            self.banner.id(),
                            banner_rect,
                            theme::unit_uv(),
                            Color32::WHITE,
                        );
                        self.art.draw_vines(ui.painter(), banner_rect, &theme::VINE_ANCHORS);
                    });
                    ui.add_space(2.0);
                    let panel_slot = ui.painter().add(egui::Shape::Noop);
                    let panel = theme::panel()
                        .inner_margin(if compact { 14 } else { 18 })
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                let realms = theme::button(
                                    ui,
                                    &self.art,
                                    "Realms",
                                    Vec2::new(132.0, 48.0),
                                    true,
                                    theme::ButtonSkin::Tab { selected: !rendered_settings },
                                );
                                self.record_focus(FocusTarget::RealmsTab, &realms);
                                self.art.icon(
                                    ui.painter(),
                                    0,
                                    egui::Rect::from_center_size(
                                        egui::pos2(realms.rect.left() + 23.0, realms.rect.center().y),
                                        Vec2::new(22.0, 24.0),
                                    ),
                                );
                                if realms.clicked() {
                                    self.settings_page = false;
                                }
                                let settings = theme::button(
                                    ui,
                                    &self.art,
                                    "Settings",
                                    Vec2::new(142.0, 48.0),
                                    true,
                                    theme::ButtonSkin::Tab { selected: rendered_settings },
                                );
                                self.record_focus(FocusTarget::SettingsTab, &settings);
                                self.art.icon(
                                    ui.painter(),
                                    1,
                                    egui::Rect::from_center_size(
                                        egui::pos2(settings.rect.left() + 23.0, settings.rect.center().y),
                                        Vec2::new(21.0, 24.0),
                                    ),
                                );
                                if settings.clicked() {
                                    self.settings_page = true;
                                }
                            });
                            ui.add_space(5.0);
                            if rendered_settings {
                                self.settings_ui(ui);
                            } else {
                                self.home(ui);
                            }
                            ui.add_space(if compact { 2.0 } else { 8.0 });
                            theme::plank(ui, &self.art, &self.message, if compact { 44.0 } else { 48.0 });
                        });
                    ui.painter().set(
                        panel_slot,
                        theme::panel_shape(&self.art, panel.response.rect),
                    );
                    let actual: Vec<_> = self.focus_targets.iter().map(|(target, _)| *target).collect();
                    debug_assert!(focus_sequence_matches(
                        &actual,
                        rendered_settings,
                        rendered_realms,
                    ));
                    if !compact {
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("A non-commercial fan revival | not affiliated with Daybreak or Sony.")
                                .size(12.0)
                                .color(Color32::from_rgb(225, 236, 244)),
                        );
                    }
                });
            });
        if let Some(path) = &self.screenshot {
            let capture_delay = if self.test_vine_growth.is_some() {
                600
            } else {
                1700
            };
            if !self.screenshot_requested
                && self.started.elapsed() > Duration::from_millis(capture_delay)
            {
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
    let raw_args: Vec<_> = std::env::args_os().skip(1).collect();
    if let Some(result) =
        osfr_launcher::key_command(raw_args.iter().map(|arg| arg.to_str().unwrap_or("")))
    {
        match result {
            Ok(output) => {
                println!("{output}");
                return Ok(());
            }
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(2);
            }
        }
    }
    diagnostics::init()?;
    let mut screenshot = None;
    let mut settings_page = false;
    let mut focus_play = false;
    let mut test_vine_growth = None;
    let mut window_size = [940.0, 790.0];
    let mut args = raw_args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--screenshot" {
            screenshot = args.next().map(PathBuf::from);
            if screenshot.is_none() {
                return Err("--screenshot requires a PNG output path.".into());
            }
        } else if arg == "--settings-page" {
            settings_page = true;
        } else if arg == "--test-focus-play" {
            focus_play = true;
        } else if arg == "--test-vine-growth-ms" {
            let millis = args
                .next()
                .and_then(|s| s.into_string().ok())
                .ok_or("--test-vine-growth-ms requires a value from 0 to 1500.")?
                .parse::<u64>()?;
            if millis > 1500 {
                return Err("--test-vine-growth-ms requires a value from 0 to 1500.".into());
            }
            test_vine_growth = Some(millis as f32 / 1500.0);
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
            .with_title("Free Realms Expanded - Launcher"),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    let capture_requested = screenshot.is_some();
    let capture_ok = Rc::new(Cell::new(false));
    let captured = capture_ok.clone();
    let result = eframe::run_native(
        "Free Realms Expanded - Launcher",
        options,
        Box::new(move |cc| {
            Ok(Box::new(
                Launcher::new(
                    cc,
                    screenshot,
                    settings_page,
                    captured,
                    focus_play,
                    test_vine_growth,
                )
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
    fn play_profile_login_calls_authenticator_and_maps_each_error_message() {
        let profile = auth::Profile::new(
            "fixture",
            "Fixture realm",
            "127.0.0.1:20043",
            [42; 32],
            "acct_001",
        )
        .unwrap();
        let cases = [
            (
                AuthError::ServerUnreachable,
                "Server unreachable. Check the broker or secure tunnel.",
            ),
            (
                AuthError::WrongServerIdentity,
                "Wrong server identity. Check the trusted server profile.",
            ),
            (
                AuthError::Refused,
                "Login refused. Check key enrolment or try again later.",
            ),
            (
                AuthError::TryLater,
                "Server closed the login request. Try again later.",
            ),
            (
                AuthError::Timeout,
                "Login timed out. Check the broker or try again later.",
            ),
            (
                AuthError::InvalidReply,
                "Server returned an invalid login reply.",
            ),
            (
                AuthError::InvalidProfile,
                "Invalid server profile. Check the trusted profile settings.",
            ),
            (
                AuthError::KeyStorage,
                "Cannot access the private profile files. Check their permissions.",
            ),
            (
                AuthError::RandomUnavailable,
                "The operating system could not generate a secure key.",
            ),
        ];
        for (error, message) in cases {
            let mut called = false;
            let mut loaded = false;
            let login = RealmLogin {
                profile_id: "fixture".into(),
            };
            let result = login.session_with(
                "127.0.0.1:20042",
                |id| {
                    loaded = true;
                    assert_eq!(id, "fixture");
                    Ok(profile.clone())
                },
                |selected| {
                    called = true;
                    assert_eq!(selected, &profile);
                    Err(error)
                },
            );
            assert!(loaded, "PLAY must load the selected realm's profile");
            assert!(called, "PLAY must call the selected profile's login");
            match result {
                Err(error) => assert_eq!(error, message),
                Ok(_) => panic!("PLAY unexpectedly accepted an authentication error"),
            }
        }
    }

    #[test]
    fn controller_focus_walk_covers_every_home_and_settings_control() {
        let cases = [
            (
                false,
                2,
                vec![
                    FocusTarget::RealmsTab,
                    FocusTarget::SettingsTab,
                    FocusTarget::RealmRow(0),
                    FocusTarget::RealmRow(1),
                    FocusTarget::AddAddress,
                    FocusTarget::AddRealm,
                    FocusTarget::Play,
                    FocusTarget::LoginProfile,
                ],
            ),
            (
                true,
                2,
                vec![
                    FocusTarget::RealmsTab,
                    FocusTarget::SettingsTab,
                    FocusTarget::ClientFolder,
                    FocusTarget::LocaleFolder,
                    FocusTarget::WindowWidth,
                    FocusTarget::WindowHeight,
                    FocusTarget::ResolutionSmall,
                    FocusTarget::ResolutionLarge,
                    FocusTarget::ControllerToggle,
                    FocusTarget::Back,
                ],
            ),
        ];
        for (settings, realm_count, expected) in cases {
            assert_eq!(focus_order(settings, realm_count), expected);
            let mut broken = expected.clone();
            broken.pop();
            assert!(!focus_sequence_matches(&broken, settings, realm_count));
            let mut reordered = expected.clone();
            reordered.swap(0, 1);
            assert!(!focus_sequence_matches(&reordered, settings, realm_count));

            let ctx = egui::Context::default();
            let mut actual = Vec::new();
            let mut path = String::new();
            let mut locale = String::new();
            let mut width = 1024;
            let mut height = 768;
            let mut controller = false;
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(900.0, 700.0),
                    )),
                    focused: true,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        for target in &expected {
                            let response = match target {
                                FocusTarget::AddAddress
                                | FocusTarget::LoginProfile
                                | FocusTarget::ClientFolder => {
                                    ui.add(egui::TextEdit::singleline(&mut path))
                                }
                                FocusTarget::LocaleFolder => {
                                    ui.add(egui::TextEdit::singleline(&mut locale))
                                }
                                FocusTarget::WindowWidth => {
                                    ui.add(egui::DragValue::new(&mut width))
                                }
                                FocusTarget::WindowHeight => {
                                    ui.add(egui::DragValue::new(&mut height))
                                }
                                FocusTarget::ControllerToggle => {
                                    ui.checkbox(&mut controller, "Controller navigation")
                                }
                                target => ui.add(egui::Button::new(format!("{target:?}"))),
                            };
                            actual.push((*target, response.id));
                        }
                    });
                },
            );
            assert_eq!(
                actual.iter().map(|(target, _)| *target).collect::<Vec<_>>(),
                expected
            );
            let mut current = None;
            for index in 0..actual.len() {
                let next = next_focus_index(current, actual.len(), true).unwrap();
                assert_eq!(next, index);
                ctx.memory_mut(|memory| memory.request_focus(actual[next].1));
                assert!(ctx.memory(|memory| memory.has_focus(actual[next].1)));
                current = Some(next);
            }
            let previous = next_focus_index(Some(0), actual.len(), false).unwrap();
            assert_eq!(actual[previous].0, *expected.last().unwrap());
        }
    }

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
