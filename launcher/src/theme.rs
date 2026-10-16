//! (R) Launcher art direction, not recovered historical launcher behaviour.
use eframe::egui::{self, Color32, FontId, RichText, Stroke, Vec2};

pub const FOREST: Color32 = Color32::from_rgb(22, 49, 26);
pub const INK: Color32 = Color32::from_rgb(43, 63, 28);
pub const PAPER: Color32 = Color32::from_rgb(255, 244, 207);
pub const GOLD: Color32 = Color32::from_rgb(248, 185, 61);
pub const ORANGE: Color32 = Color32::from_rgb(204, 84, 13);

pub fn display(size: f32) -> FontId {
    FontId::new(size, egui::FontFamily::Name("Lilita One".into()))
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
    style.visuals = egui::Visuals::light();
    style.visuals.override_text_color = Some(INK);
    style.visuals.panel_fill = FOREST;
    style.visuals.window_fill = PAPER;
    style.visuals.extreme_bg_color = Color32::from_rgb(255, 253, 235);
    style.visuals.selection.bg_fill = Color32::from_rgb(202, 229, 143);
    style.visuals.selection.stroke = Stroke::new(1.5_f32, INK);
    style.visuals.hyperlink_color = ORANGE;
    for widget in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.noninteractive,
    ] {
        widget.bg_fill = Color32::from_rgb(242, 224, 170);
        widget.weak_bg_fill = widget.bg_fill;
        widget.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(157, 134, 68));
        widget.corner_radius = 8.into();
    }
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(237, 242, 190);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(2.0_f32, ORANGE);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(222, 238, 167);
    style.visuals.widgets.active.bg_stroke = Stroke::new(3.0_f32, ORANGE);
    style.visuals.widgets.hovered.corner_radius = 8.into();
    style.visuals.widgets.active.corner_radius = 8.into();
    style.spacing.item_spacing = Vec2::new(10.0, 8.0);
    style.spacing.button_padding = Vec2::new(18.0, 8.0);
    style.spacing.interact_size.y = 34.0;
    style.spacing.scroll = egui::style::ScrollStyle::solid();
    style
        .text_styles
        .insert(egui::TextStyle::Body, FontId::proportional(16.0));
    style
        .text_styles
        .insert(egui::TextStyle::Small, FontId::proportional(13.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, FontId::proportional(16.0));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, display(30.0));
    // The OS theme can arrive after AppCreator; both themes must keep the skin.
    ctx.set_style_of(egui::Theme::Dark, style.clone());
    ctx.set_style_of(egui::Theme::Light, style);
}

pub fn panel() -> egui::Frame {
    egui::Frame::new()
        .fill(PAPER)
        .stroke(Stroke::new(2.0_f32, GOLD))
        .corner_radius(18)
        .inner_margin(16)
        .shadow(egui::epaint::Shadow {
            offset: [0, 4],
            blur: 0,
            spread: 0,
            color: Color32::from_rgb(10, 30, 14),
        })
}

/// Keep egui's Button for focus, accessibility, Space/Enter and disabled semantics.
pub fn button(
    ui: &mut egui::Ui,
    text: &str,
    size: Vec2,
    enabled: bool,
    orange: bool,
) -> egui::Response {
    let background = ui.painter().add(egui::Shape::Noop);
    let foreground = if enabled {
        Color32::WHITE
    } else {
        Color32::from_rgb(236, 239, 221)
    };
    let font = display(if size.y >= 56.0 { 32.0 } else { 20.0 });
    let response = ui
        .allocate_ui_with_layout(
            size,
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                ui.visuals_mut().disabled_alpha = 1.0;
                ui.add_enabled(
                    enabled,
                    egui::Button::new(RichText::new(text).font(font.clone()).color(foreground))
                        .frame(false)
                        .min_size(size),
                )
            },
        )
        .inner;
    let painter = ui.painter().clone();
    // Painting underneath the standard widget keeps its text and accesskit node intact.
    let mut shapes = Vec::new();
    let (top, bottom, rim) = if !enabled {
        (
            Color32::from_rgb(102, 122, 78),
            Color32::from_rgb(87, 109, 65),
            Color32::from_rgb(64, 83, 45),
        )
    } else if orange {
        (
            Color32::from_rgb(255, 190, 63),
            Color32::from_rgb(198, 74, 10),
            Color32::from_rgb(116, 48, 9),
        )
    } else {
        (
            Color32::from_rgb(161, 229, 54),
            Color32::from_rgb(42, 114, 13),
            Color32::from_rgb(27, 75, 12),
        )
    };
    let hover =
        ui.ctx()
            .animate_bool_with_time(response.id.with("gloss"), response.hovered(), 0.12);
    let lift = if response.is_pointer_button_down_on() {
        2.0
    } else {
        0.0
    };
    let rect = response.rect.shrink(3.0).translate(Vec2::new(0.0, lift));
    let radius = (rect.height() / 2.0) as u8;
    shapes.push(egui::Shape::rect_filled(
        rect.translate(Vec2::new(0.0, 3.0)),
        radius,
        rim,
    ));
    shapes.push(egui::Shape::rect_filled(rect, radius, bottom));
    // Rounded upper highlight and warm inner bevel, like the banner's green pill.
    let gloss = egui::Rect::from_min_max(
        rect.min + Vec2::splat(2.0),
        egui::pos2(rect.max.x - 2.0, rect.center().y + 1.0),
    );
    shapes.push(egui::Shape::rect_filled(
        gloss,
        egui::CornerRadius {
            nw: radius,
            ne: radius,
            sw: 5,
            se: 5,
        },
        top.gamma_multiply(1.0 + hover * 0.12),
    ));
    shapes.push(egui::Shape::rect_stroke(
        rect,
        radius,
        Stroke::new(
            1.5_f32,
            if enabled {
                Color32::from_rgb(218, 242, 133)
            } else {
                top
            },
        ),
        egui::StrokeKind::Inside,
    ));
    let shadow = painter.layout_no_wrap(text.to_owned(), font, rim);
    shapes.push(egui::Shape::galley(
        response.rect.center() - shadow.size() / 2.0 + Vec2::new(0.0, 1.5),
        shadow,
        rim,
    ));
    if response.has_focus() {
        shapes.push(egui::Shape::rect_stroke(
            response.rect,
            radius + 3,
            Stroke::new(3.0_f32, ORANGE),
            egui::StrokeKind::Outside,
        ));
        shapes.push(egui::Shape::rect_stroke(
            response.rect.expand(3.0),
            radius + 6,
            Stroke::new(1.5_f32, PAPER),
            egui::StrokeKind::Outside,
        ));
    }
    painter.set(background, egui::Shape::Vec(shapes));
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skin_survives_theme_changes_and_buttons_keep_keyboard_semantics() {
        let ctx = egui::Context::default();
        install(&ctx);
        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            ctx.set_theme(theme);
            assert_eq!(
                ctx.style().text_styles[&egui::TextStyle::Heading],
                display(30.0)
            );
        }
        let draw = |events: Vec<egui::Event>, enabled| {
            let mut response = None;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(720.0, 620.0),
                    )),
                    events,
                    focused: true,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        response =
                            Some(button(ui, "Select", Vec2::new(180.0, 44.0), enabled, false));
                    });
                },
            );
            (response.unwrap(), output)
        };
        let (initial, _) = draw(vec![], true);
        ctx.memory_mut(|memory| memory.request_focus(initial.id));
        let (focused, output) = draw(vec![], true);
        assert!(focused.has_focus());
        assert!(output.shapes.iter().any(|shape| match &shape.shape {
            egui::Shape::Vec(shapes) => shapes.iter().any(|shape| matches!(shape,
                egui::Shape::Rect(rect) if rect.stroke.color == ORANGE && rect.stroke.width == 3.0)),
            _ => false,
        }), "controller/keyboard focus must paint its orange ring");
        let confirm = || {
            vec![egui::Event::Key {
                key: egui::Key::Space,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }]
        };
        assert!(
            draw(confirm(), true).0.clicked(),
            "controller's Space confirm must activate"
        );
        assert!(
            !draw(confirm(), false).0.clicked(),
            "disabled controls must ignore confirm"
        );
    }
}

pub fn heading(ui: &mut egui::Ui, title: &str, detail: &str) {
    let size = if ui.ctx().content_rect().height() < 650.0 {
        26.0
    } else {
        30.0
    };
    ui.heading(RichText::new(title).font(display(size)).color(INK));
    ui.label(detail);
    ui.add_space(4.0);
}
