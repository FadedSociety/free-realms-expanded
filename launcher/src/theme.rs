//! (R) Launcher art direction and image layout, not recovered historical UI code.
use eframe::egui::{self, Color32, FontId, RichText, Stroke, Vec2};
use std::collections::VecDeque;

pub const INK: Color32 = Color32::from_rgb(243, 249, 255);
pub const DARK_INK: Color32 = Color32::from_rgb(38, 50, 66);
pub const BLUE: Color32 = Color32::from_rgb(7, 42, 91);
pub const FOCUS_GLOW: Color32 = Color32::from_rgb(113, 237, 255);

pub fn unit_uv() -> egui::Rect {
    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0))
}

pub fn display(size: f32) -> FontId {
    FontId::new(size, egui::FontFamily::Name("Lilita One".into()))
}

#[derive(Clone, Copy)]
pub enum ButtonSkin {
    Standard,
    Play,
    Tab { selected: bool },
}

struct GrowingVine {
    texture: egui::TextureHandle,
    source: egui::ColorImage,
    order: Vec<u32>,
    root: [usize; 2],
    last_cutoff: Option<u32>,
}

pub struct VineAnchor {
    pub x: f32,
    pub y: f32,
    pub width: f32,
}

pub struct UiArt {
    backdrop: egui::TextureHandle,
    buttons: [egui::TextureHandle; 4],
    play: [egui::TextureHandle; 2],
    tabs: [egui::TextureHandle; 2],
    panel: egui::TextureHandle,
    list_row: egui::TextureHandle,
    text_field: egui::TextureHandle,
    pips: [egui::TextureHandle; 4],
    plank: egui::TextureHandle,
    icons: [egui::TextureHandle; 3],
    vines: Vec<GrowingVine>,
}

impl UiArt {
    pub fn load(ctx: &egui::Context) -> Result<Self, String> {
        macro_rules! texture {
            ($name:literal) => {
                load_texture(ctx, $name, include_bytes!(concat!("../assets/ui/", $name)))?
            };
        }
        let vines = vec![
            load_vine(
                ctx,
                "vine 1",
                include_bytes!("../assets/ui/vine_1.png"),
                [10, 161],
            )?,
            load_vine(
                ctx,
                "vine 2",
                include_bytes!("../assets/ui/vine_2.png"),
                [11, 180],
            )?,
            load_vine(
                ctx,
                "vine 3",
                include_bytes!("../assets/ui/vine_3.png"),
                [8, 161],
            )?,
            load_vine(
                ctx,
                "vine 4",
                include_bytes!("../assets/ui/vine_4.png"),
                [9, 189],
            )?,
        ];
        Ok(Self {
            backdrop: texture!("backdrop.png"),
            buttons: [
                texture!("button_normal.png"),
                texture!("button_hover.png"),
                texture!("button_pressed.png"),
                texture!("button_disabled.png"),
            ],
            play: [texture!("play_normal.png"), texture!("play_hover.png")],
            tabs: [texture!("tab_selected.png"), texture!("tab_unselected.png")],
            panel: texture!("panel.png"),
            list_row: texture!("list_row.png"),
            text_field: texture!("text_field.png"),
            pips: [
                texture!("pip_online.png"),
                texture!("pip_pending.png"),
                texture!("pip_error.png"),
                texture!("pip_disabled.png"),
            ],
            plank: texture!("plank.png"),
            icons: [
                texture!("icon_leaf_green.png"),
                texture!("icon_leaf_blue.png"),
                texture!("icon_sprig_blue.png"),
            ],
            vines,
        })
    }

    pub fn repaint_vines(&mut self, growth: f32) -> bool {
        let mut changed = false;
        for vine in &mut self.vines {
            let cutoff = growth_cutoff(&vine.order, growth);
            if vine.last_cutoff == Some(cutoff) {
                continue;
            }
            let mut image = vine.source.clone();
            for (pixel, order) in image.pixels.iter_mut().zip(&vine.order) {
                if *order == u32::MAX || *order >= cutoff {
                    *pixel = Color32::TRANSPARENT;
                }
            }
            vine.texture.set(image, egui::TextureOptions::LINEAR);
            vine.last_cutoff = Some(cutoff);
            changed = true;
        }
        changed
    }

    pub fn draw_backdrop(&self, painter: &egui::Painter, rect: egui::Rect) {
        let [width, height] = self.backdrop.size();
        let source_aspect = width as f32 / height as f32;
        let target_aspect = rect.width() / rect.height().max(1.0);
        let uv = if source_aspect > target_aspect {
            let visible = target_aspect / source_aspect;
            egui::Rect::from_min_max(
                egui::pos2((1.0 - visible) * 0.5, 0.0),
                egui::pos2((1.0 + visible) * 0.5, 1.0),
            )
        } else {
            let visible = source_aspect / target_aspect;
            egui::Rect::from_min_max(
                egui::pos2(0.0, (1.0 - visible) * 0.5),
                egui::pos2(1.0, (1.0 + visible) * 0.5),
            )
        };
        painter.image(self.backdrop.id(), rect, uv, Color32::WHITE);
        painter.rect_filled(rect, 0, Color32::from_rgba_unmultiplied(5, 20, 43, 102));
    }

    pub fn draw_vines(
        &self,
        painter: &egui::Painter,
        banner: egui::Rect,
        anchors: &[VineAnchor; 4],
    ) {
        for (vine, anchor) in self.vines.iter().zip(anchors) {
            let [source_w, source_h] = vine.source.size;
            let width = banner.width() * anchor.width;
            let height = width * source_h as f32 / source_w as f32;
            let root = egui::pos2(
                banner.left() + banner.width() * anchor.x,
                banner.top() + banner.height() * anchor.y,
            );
            let origin = root
                - Vec2::new(
                    width * vine.root[0] as f32 / source_w as f32,
                    height * vine.root[1] as f32 / source_h as f32,
                );
            let rect = egui::Rect::from_min_size(origin, Vec2::new(width, height));
            painter.image(vine.texture.id(), rect, unit_uv(), Color32::WHITE);
        }
    }

    pub fn icon(&self, painter: &egui::Painter, index: usize, rect: egui::Rect) {
        if let Some(texture) = self.icons.get(index) {
            painter.image(texture.id(), rect, unit_uv(), Color32::WHITE);
        }
    }

    pub fn pip(&self, painter: &egui::Painter, index: usize, rect: egui::Rect) {
        if let Some(texture) = self.pips.get(index) {
            painter.image(texture.id(), rect, unit_uv(), Color32::WHITE);
        }
    }
}

fn load_texture(
    ctx: &egui::Context,
    name: &str,
    bytes: &[u8],
) -> Result<egui::TextureHandle, String> {
    let image = image::load_from_memory(bytes)
        .map_err(|_| format!("The bundled {name} art could not be decoded."))?
        .into_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    Ok(ctx.load_texture(
        name,
        egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw()),
        egui::TextureOptions::LINEAR,
    ))
}

fn load_vine(
    ctx: &egui::Context,
    name: &str,
    bytes: &[u8],
    root: [usize; 2],
) -> Result<GrowingVine, String> {
    let rgba = image::load_from_memory(bytes)
        .map_err(|_| format!("The bundled {name} art could not be decoded."))?
        .into_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let source = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
    let order = growth_order(&source, root);
    let texture = ctx.load_texture(
        name,
        egui::ColorImage::filled(size, Color32::TRANSPARENT),
        egui::TextureOptions::LINEAR,
    );
    Ok(GrowingVine {
        texture,
        source,
        order,
        root,
        last_cutoff: Some(0),
    })
}

fn growth_order(image: &egui::ColorImage, root: [usize; 2]) -> Vec<u32> {
    let [width, height] = image.size;
    let mut order = vec![u32::MAX; width * height];
    if root[0] >= width || root[1] >= height {
        return order;
    }
    let root_index = root[1] * width + root[0];
    if image.pixels[root_index].a() == 0 {
        return order;
    }
    order[root_index] = 0;
    let mut queue = VecDeque::from([root_index]);
    while let Some(index) = queue.pop_front() {
        let x = index % width;
        let y = index / width;
        let next_order = order[index] + 1;
        for dy in -1_i32..=1 {
            for dx in -1_i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                    continue;
                }
                let next = ny as usize * width + nx as usize;
                if image.pixels[next].a() > 0 && order[next] == u32::MAX {
                    order[next] = next_order;
                    queue.push_back(next);
                }
            }
        }
    }
    order
}

fn growth_cutoff(order: &[u32], growth: f32) -> u32 {
    if !growth.is_finite() || growth <= 0.0 {
        return 0;
    }
    if growth >= 1.0 {
        return u32::MAX;
    }
    let max = order
        .iter()
        .copied()
        .filter(|d| *d != u32::MAX)
        .max()
        .unwrap_or(0);
    ((max + 1) as f32 * growth).ceil() as u32
}

fn add_quad(mesh: &mut egui::Mesh, texture_size: [usize; 2], dst: egui::Rect, src: egui::Rect) {
    if dst.width() <= 0.0 || dst.height() <= 0.0 || src.width() <= 0.0 || src.height() <= 0.0 {
        return;
    }
    let uv = egui::Rect::from_min_max(
        egui::pos2(
            src.left() / texture_size[0] as f32,
            src.top() / texture_size[1] as f32,
        ),
        egui::pos2(
            src.right() / texture_size[0] as f32,
            src.bottom() / texture_size[1] as f32,
        ),
    );
    mesh.add_rect_with_uv(dst, uv, Color32::WHITE);
}

fn tile_horizontal(
    mesh: &mut egui::Mesh,
    texture_size: [usize; 2],
    dst: egui::Rect,
    src: egui::Rect,
) {
    let mut x = dst.left();
    while x < dst.right() {
        let width = src.width().min(dst.right() - x);
        let used_source = src.width() * width / src.width().max(1.0);
        add_quad(
            mesh,
            texture_size,
            egui::Rect::from_min_size(egui::pos2(x, dst.top()), Vec2::new(width, dst.height())),
            egui::Rect::from_min_size(src.min, Vec2::new(used_source, src.height())),
        );
        x += width;
    }
}

fn tile_vertical(
    mesh: &mut egui::Mesh,
    texture_size: [usize; 2],
    dst: egui::Rect,
    src: egui::Rect,
) {
    let mut y = dst.top();
    while y < dst.bottom() {
        let height = src.height().min(dst.bottom() - y);
        let used_source = src.height() * height / src.height().max(1.0);
        add_quad(
            mesh,
            texture_size,
            egui::Rect::from_min_size(egui::pos2(dst.left(), y), Vec2::new(dst.width(), height)),
            egui::Rect::from_min_size(src.min, Vec2::new(src.width(), used_source)),
        );
        y += height;
    }
}

/// Fixed corners, repeated edge strips, and a stretched centre keep bump spacing stable.
fn nine_slice(
    texture: &egui::TextureHandle,
    rect: egui::Rect,
    corner_x: f32,
    corner_y: f32,
) -> egui::Shape {
    let [width, height] = texture.size();
    let sw = width as f32;
    let sh = height as f32;
    let cx = corner_x.min(sw / 3.0).min(rect.width() * 0.5);
    let cy = corner_y.min(sh / 3.0).min(rect.height() * 0.5);
    let sx = [0.0, cx, sw - cx, sw];
    let sy = [0.0, cy, sh - cy, sh];
    let dx = [
        rect.left(),
        rect.left() + cx,
        rect.right() - cx,
        rect.right(),
    ];
    let dy = [
        rect.top(),
        rect.top() + cy,
        rect.bottom() - cy,
        rect.bottom(),
    ];
    let mut mesh = egui::Mesh::with_texture(texture.id());
    for (row, col) in [(0, 0), (0, 2), (2, 0), (2, 2)] {
        add_quad(
            &mut mesh,
            [width, height],
            egui::Rect::from_min_max(
                egui::pos2(dx[col], dy[row]),
                egui::pos2(dx[col + 1], dy[row + 1]),
            ),
            egui::Rect::from_min_max(
                egui::pos2(sx[col], sy[row]),
                egui::pos2(sx[col + 1], sy[row + 1]),
            ),
        );
    }
    tile_horizontal(
        &mut mesh,
        [width, height],
        egui::Rect::from_min_max(egui::pos2(dx[1], dy[0]), egui::pos2(dx[2], dy[1])),
        egui::Rect::from_min_max(egui::pos2(sx[1], sy[0]), egui::pos2(sx[2], sy[1])),
    );
    tile_horizontal(
        &mut mesh,
        [width, height],
        egui::Rect::from_min_max(egui::pos2(dx[1], dy[2]), egui::pos2(dx[2], dy[3])),
        egui::Rect::from_min_max(egui::pos2(sx[1], sy[2]), egui::pos2(sx[2], sy[3])),
    );
    tile_vertical(
        &mut mesh,
        [width, height],
        egui::Rect::from_min_max(egui::pos2(dx[0], dy[1]), egui::pos2(dx[1], dy[2])),
        egui::Rect::from_min_max(egui::pos2(sx[0], sy[1]), egui::pos2(sx[1], sy[2])),
    );
    tile_vertical(
        &mut mesh,
        [width, height],
        egui::Rect::from_min_max(egui::pos2(dx[2], dy[1]), egui::pos2(dx[3], dy[2])),
        egui::Rect::from_min_max(egui::pos2(sx[2], sy[1]), egui::pos2(sx[3], sy[2])),
    );
    add_quad(
        &mut mesh,
        [width, height],
        egui::Rect::from_min_max(egui::pos2(dx[1], dy[1]), egui::pos2(dx[2], dy[2])),
        egui::Rect::from_min_max(egui::pos2(sx[1], sy[1]), egui::pos2(sx[2], sy[2])),
    );
    egui::Shape::mesh(mesh)
}

pub fn focus_shapes(rect: egui::Rect) -> [egui::Shape; 2] {
    [
        egui::Shape::rect_stroke(
            rect.expand(4.0),
            18,
            Stroke::new(4.0_f32, FOCUS_GLOW),
            egui::StrokeKind::Outside,
        ),
        egui::Shape::rect_stroke(
            rect.expand(8.0),
            20,
            Stroke::new(1.5_f32, Color32::WHITE),
            egui::StrokeKind::Outside,
        ),
    ]
}

pub fn focus_outline(ui: &egui::Ui, response: &egui::Response) {
    if response.has_focus() {
        for shape in focus_shapes(response.rect) {
            ui.painter().add(shape);
        }
    }
}

fn button_texture<'a>(
    art: &'a UiArt,
    skin: ButtonSkin,
    enabled: bool,
    hovered: bool,
    pressed: bool,
) -> (&'a egui::TextureHandle, f32, f32) {
    match skin {
        ButtonSkin::Tab { selected } => (&art.tabs[if selected { 0 } else { 1 }], 42.0, 22.0),
        ButtonSkin::Play if enabled && !pressed => (&art.play[usize::from(hovered)], 56.0, 30.0),
        ButtonSkin::Play if !enabled => (&art.buttons[3], 48.0, 22.0),
        ButtonSkin::Play => (&art.buttons[2], 48.0, 22.0),
        ButtonSkin::Standard if !enabled => (&art.buttons[3], 44.0, 22.0),
        ButtonSkin::Standard if pressed => (&art.buttons[2], 44.0, 22.0),
        ButtonSkin::Standard if hovered => (&art.buttons[1], 44.0, 22.0),
        ButtonSkin::Standard => (&art.buttons[0], 44.0, 22.0),
    }
}

/// Keep egui's real Button for keyboard focus, Space/Enter, accesskit and disabled semantics.
pub fn button(
    ui: &mut egui::Ui,
    art: &UiArt,
    text: &str,
    size: Vec2,
    enabled: bool,
    skin: ButtonSkin,
) -> egui::Response {
    let painter = ui.painter().clone();
    let slot = painter.add(egui::Shape::Noop);
    let font = display(if matches!(skin, ButtonSkin::Play) {
        30.0
    } else {
        19.0
    });
    let response = ui
        .allocate_ui_with_layout(
            size,
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                ui.visuals_mut().disabled_alpha = 1.0;
                ui.add_enabled(
                    enabled,
                    egui::Button::new(RichText::new(text).font(font.clone()).color(Color32::WHITE))
                        .frame(false)
                        .min_size(size),
                )
            },
        )
        .inner;
    let (texture, cx, cy) = button_texture(
        art,
        skin,
        enabled,
        response.hovered(),
        response.is_pointer_button_down_on(),
    );
    let mut shapes = vec![nine_slice(texture, response.rect.shrink(1.0), cx, cy)];
    if response.has_focus() {
        shapes.extend(focus_shapes(response.rect));
    }
    // A dark outline keeps the white caption clear on both bright and shaded green pixels.
    let outline_color = Color32::from_rgb(7, 18, 15);
    let outline = painter.layout_no_wrap(text.to_owned(), font, outline_color);
    for offset in [
        Vec2::new(-1.2, -1.2),
        Vec2::new(0.0, -1.5),
        Vec2::new(1.2, -1.2),
        Vec2::new(-1.5, 0.0),
        Vec2::new(1.5, 0.0),
        Vec2::new(-1.2, 1.2),
        Vec2::new(0.0, 1.5),
        Vec2::new(1.2, 1.2),
    ] {
        shapes.push(egui::Shape::galley(
            response.rect.center() - outline.size() / 2.0 + offset,
            outline.clone(),
            outline_color,
        ));
    }
    painter.set(slot, egui::Shape::Vec(shapes));
    response
}

pub fn row_button(
    ui: &mut egui::Ui,
    art: &UiArt,
    name: &str,
    address: &str,
    size: Vec2,
) -> egui::Response {
    let painter = ui.painter().clone();
    let slot = painter.add(egui::Shape::Noop);
    let accessible = format!("{name}, {address}, status pending");
    let response = ui
        .allocate_ui_with_layout(
            size,
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                ui.add(
                    egui::Button::new(RichText::new(accessible).color(Color32::TRANSPARENT))
                        .frame(false)
                        .min_size(size),
                )
            },
        )
        .inner;
    let mut shapes = vec![nine_slice(
        &art.list_row,
        response.rect.shrink(1.0),
        48.0,
        17.0,
    )];
    if response.has_focus() {
        shapes.extend(focus_shapes(response.rect));
    }
    painter.set(slot, egui::Shape::Vec(shapes));
    let rect = response.rect;
    art.pip(
        &painter,
        1,
        egui::Rect::from_center_size(
            egui::pos2(rect.left() + 28.0, rect.center().y),
            Vec2::splat(24.0),
        ),
    );
    let copy = painter.with_clip_rect(egui::Rect::from_min_max(
        egui::pos2(rect.left() + 50.0, rect.top()),
        egui::pos2(rect.right() - 122.0, rect.bottom()),
    ));
    copy.text(
        egui::pos2(rect.left() + 50.0, rect.top() + 7.0),
        egui::Align2::LEFT_TOP,
        name,
        display(19.0),
        INK,
    );
    copy.text(
        egui::pos2(rect.left() + 50.0, rect.top() + 31.0),
        egui::Align2::LEFT_TOP,
        address,
        FontId::monospace(14.0),
        INK,
    );
    painter.text(
        egui::pos2(rect.right() - 16.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        "Status pending",
        FontId::proportional(12.0),
        Color32::from_rgb(226, 242, 255),
    );
    response
}

pub fn text_field(
    ui: &mut egui::Ui,
    art: &UiArt,
    value: &mut String,
    hint: &str,
    size: Vec2,
    max_chars: usize,
) -> egui::Response {
    let painter = ui.painter().clone();
    let slot = painter.add(egui::Shape::Noop);
    let inner = ui.allocate_ui_with_layout(
        size,
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.add_space(18.0);
            ui.visuals_mut().override_text_color = Some(DARK_INK);
            ui.add_sized(
                Vec2::new((size.x - 36.0).max(40.0), (size.y - 12.0).max(32.0)),
                egui::TextEdit::singleline(value)
                    .hint_text(hint)
                    .char_limit(max_chars)
                    .frame(false),
            )
        },
    );
    let response = inner.inner;
    let rect = inner.response.rect;
    let mut shapes = vec![nine_slice(&art.text_field, rect.shrink(1.0), 24.0, 16.0)];
    painter.set(slot, egui::Shape::Vec(shapes));
    if response.has_focus() {
        shapes = focus_shapes(rect).into();
        for shape in shapes {
            painter.add(shape);
        }
    }
    response
}

pub fn panel() -> egui::Frame {
    egui::Frame::new()
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE)
        .corner_radius(0)
        .inner_margin(18)
}

pub fn panel_shape(art: &UiArt, rect: egui::Rect) -> egui::Shape {
    nine_slice(&art.panel, rect, 88.0, 24.0)
}

pub fn plank(ui: &mut egui::Ui, art: &UiArt, text: &str, height: f32) {
    let size = Vec2::new(ui.available_width(), height.max(44.0));
    let painter = ui.painter().clone();
    let slot = painter.add(egui::Shape::Noop);
    let inner = ui.allocate_ui_with_layout(
        size,
        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
        |ui| {
            ui.add(
                egui::Label::new(
                    RichText::new(text)
                        .color(Color32::from_rgb(255, 242, 208))
                        .size(14.0),
                )
                .wrap(),
            )
        },
    );
    let rect = inner.response.rect;
    painter.set(slot, nine_slice(&art.plank, rect, 60.0, 32.0));
}

pub fn install(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "Lilita One".into(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/LilitaOne-Regular.ttf")).into(),
    );
    fonts.families.insert(
        egui::FontFamily::Name("Lilita One".into()),
        vec!["Lilita One".into(), "Ubuntu-Light".into()],
    );
    ctx.set_fonts(fonts);
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.override_text_color = Some(INK);
    style.visuals.panel_fill = BLUE;
    style.visuals.window_fill = BLUE;
    style.visuals.extreme_bg_color = Color32::from_rgb(246, 249, 255);
    style.visuals.selection.bg_fill = Color32::from_rgb(34, 126, 215);
    style.visuals.selection.stroke = Stroke::new(1.5_f32, Color32::WHITE);
    style.visuals.hyperlink_color = Color32::from_rgb(164, 229, 255);
    for widget in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.noninteractive,
    ] {
        widget.bg_fill = Color32::from_rgba_unmultiplied(18, 82, 143, 190);
        widget.weak_bg_fill = widget.bg_fill;
        widget.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(77, 170, 238));
        widget.corner_radius = 8.into();
    }
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgba_unmultiplied(33, 111, 181, 210);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(2.0_f32, FOCUS_GLOW);
    style.visuals.widgets.active.bg_fill = Color32::from_rgba_unmultiplied(12, 72, 130, 230);
    style.visuals.widgets.active.bg_stroke = Stroke::new(2.0_f32, Color32::WHITE);
    style.visuals.widgets.hovered.corner_radius = 8.into();
    style.visuals.widgets.active.corner_radius = 8.into();
    style.spacing.item_spacing = Vec2::new(8.0, 6.0);
    style.spacing.button_padding = Vec2::new(14.0, 8.0);
    style.spacing.interact_size = Vec2::new(44.0, 44.0);
    style.spacing.scroll = egui::style::ScrollStyle::solid();
    style
        .text_styles
        .insert(egui::TextStyle::Body, FontId::proportional(16.0));
    style
        .text_styles
        .insert(egui::TextStyle::Small, FontId::proportional(13.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, FontId::proportional(18.0));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, display(30.0));
    ctx.set_style_of(egui::Theme::Dark, style.clone());
    ctx.set_style_of(egui::Theme::Light, style);
}

pub fn heading(ui: &mut egui::Ui, title: &str, detail: &str) {
    let size = if ui.ctx().content_rect().height() < 650.0 {
        26.0
    } else {
        30.0
    };
    ui.heading(RichText::new(title).font(display(size)).color(INK));
    ui.label(RichText::new(detail).color(INK));
    ui.add_space(2.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transparent_root(bytes: &[u8], root: [usize; 2]) -> (egui::ColorImage, Vec<u32>) {
        let image = image::load_from_memory(bytes).unwrap().into_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let color = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
        let order = growth_order(&color, root);
        (color, order)
    }

    #[test]
    fn focus_glow_is_painted_over_the_button_skin() {
        let ctx = egui::Context::default();
        install(&ctx);
        let art = UiArt::load(&ctx).unwrap();
        let mut response = None;
        let draw = |ctx: &egui::Context, response: &mut Option<egui::Response>| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(720.0, 620.0),
                    )),
                    focused: true,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        *response = Some(button(
                            ui,
                            &art,
                            "PLAY NOW!",
                            Vec2::new(224.0, 56.0),
                            true,
                            ButtonSkin::Play,
                        ));
                    });
                },
            )
        };
        let _ = draw(&ctx, &mut response);
        ctx.memory_mut(|memory| memory.request_focus(response.as_ref().unwrap().id));
        let output = draw(&ctx, &mut response);
        assert!(response.as_ref().unwrap().has_focus());
        assert!(output.shapes.iter().any(|shape| match &shape.shape {
            egui::Shape::Vec(shapes) => shapes.iter().any(|shape| matches!(shape,
                egui::Shape::Rect(rect) if rect.stroke.color == FOCUS_GLOW && rect.stroke.width == 4.0)),
            _ => false,
        }), "controller focus must draw the bright glow above the image skin");
    }

    #[test]
    fn vine_growth_starts_at_root_and_reaches_every_opaque_pixel() {
        let vines = [
            (
                include_bytes!("../assets/ui/vine_1.png").as_slice(),
                [10, 161],
            ),
            (
                include_bytes!("../assets/ui/vine_2.png").as_slice(),
                [11, 180],
            ),
            (
                include_bytes!("../assets/ui/vine_3.png").as_slice(),
                [8, 161],
            ),
            (
                include_bytes!("../assets/ui/vine_4.png").as_slice(),
                [9, 189],
            ),
        ];
        for (bytes, root) in vines {
            let (image, order) = transparent_root(bytes, root);
            let root_index = root[1] * image.size[0] + root[0];
            assert_eq!(order[root_index], 0, "vine root must grow first");
            assert!(
                image
                    .pixels
                    .iter()
                    .zip(&order)
                    .all(|(pixel, distance)| pixel.a() == 0 || *distance != u32::MAX)
            );
            let empty = order
                .iter()
                .filter(|distance| {
                    **distance != u32::MAX && **distance < growth_cutoff(&order, 0.0)
                })
                .count();
            assert_eq!(empty, 0, "growth 0 must reveal no pixels");
            let full = growth_cutoff(&order, 1.0);
            assert!(
                image
                    .pixels
                    .iter()
                    .zip(&order)
                    .all(|(pixel, distance)| pixel.a() == 0 || *distance < full)
            );
            let counts: Vec<_> = [0.0, 0.3, 0.8, 1.0]
                .map(|growth| {
                    let cutoff = growth_cutoff(&order, growth);
                    order
                        .iter()
                        .filter(|distance| **distance != u32::MAX && **distance < cutoff)
                        .count()
                })
                .into();
            assert!(counts.windows(2).all(|pair| pair[0] <= pair[1]));
        }
    }
}
