use eframe::egui;

use crate::app::NODE_CORNER_RADIUS;
use crate::core::i18n::Language;
use crate::core::layout::LayoutEngine;
use crate::core::tree::{FamilyTree, Gender, Person, PersonDisplayMode, PersonId};
use crate::infrastructure::PhotoTextureCache;
use crate::ui::NodeColorThemePreset;

const NAME_AREA_HEIGHT: f32 = 30.0;

const GENDER_VARIANT_COUNT: usize = 3;

pub struct NodeColorTheme {
    base_fill: [egui::Color32; GENDER_VARIANT_COUNT],
    selected_fill: [egui::Color32; GENDER_VARIANT_COUNT],
    multi_selected_fill: [egui::Color32; GENDER_VARIANT_COUNT],
    dragging_fill: egui::Color32,
    selected_stroke: egui::Color32,
    multi_selected_stroke: egui::Color32,
    default_stroke: egui::Color32,
    selected_stroke_width: f32,
    default_stroke_width: f32,
}

pub const DEFAULT_NODE_COLOR_THEME: NodeColorTheme = NodeColorTheme {
    base_fill: [
        egui::Color32::from_rgb(173, 216, 230),
        egui::Color32::from_rgb(255, 182, 193),
        egui::Color32::from_rgb(245, 245, 245),
    ],
    selected_fill: [
        egui::Color32::from_rgb(200, 235, 255),
        egui::Color32::from_rgb(255, 220, 230),
        egui::Color32::from_rgb(200, 230, 255),
    ],
    multi_selected_fill: [
        egui::Color32::from_rgb(190, 225, 245),
        egui::Color32::from_rgb(255, 210, 220),
        egui::Color32::from_rgb(225, 240, 255),
    ],
    dragging_fill: egui::Color32::from_rgb(255, 220, 180),
    selected_stroke: egui::Color32::from_rgb(0, 100, 200),
    multi_selected_stroke: egui::Color32::from_rgb(100, 150, 200),
    default_stroke: egui::Color32::GRAY,
    selected_stroke_width: 2.0,
    default_stroke_width: 1.0,
};

pub const HIGH_CONTRAST_NODE_COLOR_THEME: NodeColorTheme = NodeColorTheme {
    base_fill: [
        egui::Color32::from_rgb(140, 200, 255),
        egui::Color32::from_rgb(255, 155, 200),
        egui::Color32::from_rgb(230, 230, 230),
    ],
    selected_fill: [
        egui::Color32::from_rgb(80, 170, 255),
        egui::Color32::from_rgb(255, 100, 170),
        egui::Color32::from_rgb(190, 220, 255),
    ],
    multi_selected_fill: [
        egui::Color32::from_rgb(120, 185, 255),
        egui::Color32::from_rgb(255, 130, 185),
        egui::Color32::from_rgb(210, 235, 255),
    ],
    dragging_fill: egui::Color32::from_rgb(255, 190, 120),
    selected_stroke: egui::Color32::from_rgb(0, 60, 160),
    multi_selected_stroke: egui::Color32::from_rgb(40, 100, 180),
    default_stroke: egui::Color32::from_gray(70),
    selected_stroke_width: 2.0,
    default_stroke_width: 1.0,
};

pub fn node_color_theme_from_preset(preset: NodeColorThemePreset) -> &'static NodeColorTheme {
    match preset {
        NodeColorThemePreset::Default => &DEFAULT_NODE_COLOR_THEME,
        NodeColorThemePreset::HighContrast => &HIGH_CONTRAST_NODE_COLOR_THEME,
    }
}

pub struct NodeRenderInput {
    pub person_id: PersonId,
    pub rect: egui::Rect,
    pub is_selected: bool,
    pub is_multi_selected: bool,
    pub is_dragging: bool,
    pub gender: Gender,
    pub display_mode: Option<PersonDisplayMode>,
    pub photo_path: Option<String>,
}

impl NodeRenderInput {
    pub fn from_person(
        person_id: PersonId,
        rect: egui::Rect,
        is_selected: bool,
        is_multi_selected: bool,
        is_dragging: bool,
        person: Option<&Person>,
    ) -> Self {
        let gender = person.map(|person| person.gender).unwrap_or(Gender::Unknown);
        let display_mode = person.map(|person| person.display_mode);
        let photo_path = person.and_then(|person| person.photo_path.clone());

        Self {
            person_id,
            rect,
            is_selected,
            is_multi_selected,
            is_dragging,
            gender,
            display_mode,
            photo_path,
        }
    }
}

struct NodeVisualStyle {
    fill_color: egui::Color32,
    stroke_color: egui::Color32,
    stroke_width: f32,
}

pub struct NodePainter<'a> {
    ui: &'a mut egui::Ui,
    painter: &'a egui::Painter,
    tree: &'a FamilyTree,
    zoom: f32,
    language: Language,
    photo_texture_cache: &'a mut PhotoTextureCache,
    color_theme: &'static NodeColorTheme,
}

impl<'a> NodePainter<'a> {
    pub fn new_with_theme(
        ui: &'a mut egui::Ui,
        painter: &'a egui::Painter,
        tree: &'a FamilyTree,
        zoom: f32,
        language: Language,
        photo_texture_cache: &'a mut PhotoTextureCache,
        color_theme: &'static NodeColorTheme,
    ) -> Self {
        Self {
            ui,
            painter,
            tree,
            zoom,
            language,
            photo_texture_cache,
            color_theme,
        }
    }

    pub fn draw_node(&mut self, input: &NodeRenderInput) {
        let visual_style = self.resolve_node_visual_style(input);

        self.draw_frame(input.rect, &visual_style);
        self.draw_person_content(input);
        self.draw_tooltip(input);
    }

    fn gender_index(gender: Gender) -> usize {
        match gender {
            Gender::Male => 0,
            Gender::Female => 1,
            Gender::Unknown => 2,
        }
    }

    fn resolve_node_visual_style(&self, input: &NodeRenderInput) -> NodeVisualStyle {
        let gender_index = Self::gender_index(input.gender);
        let fill_color = if input.is_dragging {
            self.color_theme.dragging_fill
        } else if input.is_selected {
            self.color_theme.selected_fill[gender_index]
        } else if input.is_multi_selected {
            self.color_theme.multi_selected_fill[gender_index]
        } else {
            self.color_theme.base_fill[gender_index]
        };

        let stroke_width = if input.is_multi_selected {
            self.color_theme.selected_stroke_width
        } else {
            self.color_theme.default_stroke_width
        };
        let stroke_color = if input.is_selected {
            self.color_theme.selected_stroke
        } else if input.is_multi_selected {
            self.color_theme.multi_selected_stroke
        } else {
            self.color_theme.default_stroke
        };

        NodeVisualStyle {
            fill_color,
            stroke_color,
            stroke_width,
        }
    }

    fn draw_frame(&self, rect: egui::Rect, style: &NodeVisualStyle) {
        self.painter
            .rect_filled(rect, NODE_CORNER_RADIUS, style.fill_color);
        self.painter.rect_stroke(
            rect,
            NODE_CORNER_RADIUS,
            egui::Stroke::new(style.stroke_width, style.stroke_color),
            egui::epaint::StrokeKind::Outside,
        );
    }

    fn draw_person_content(&mut self, input: &NodeRenderInput) {
        if input.display_mode == Some(PersonDisplayMode::NameAndPhoto) {
            if let Some(photo_path) = input.photo_path.as_deref() {
                if !photo_path.is_empty() {
                    self.draw_photo_and_name(input.rect, input.person_id, photo_path);
                    return;
                }
            }
        }

        self.draw_person_name(input.rect.center(), input.person_id);
    }

    fn draw_photo_and_name(&mut self, rect: egui::Rect, person_id: PersonId, photo_path: &str) {
        let photo_height = rect.height() - NAME_AREA_HEIGHT;
        let photo_rect =
            egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), photo_height));

        if let Some(texture) = self.photo_texture_cache.get_or_load(self.ui.ctx(), photo_path) {
            self.painter.image(
                texture.id(),
                photo_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }

        let text_center = egui::pos2(
            rect.center().x,
            rect.min.y + photo_height + NAME_AREA_HEIGHT / 2.0,
        );
        self.draw_person_name(text_center, person_id);
    }

    fn draw_person_name(&self, center: egui::Pos2, person_id: PersonId) {
        let text = LayoutEngine::person_label(self.tree, person_id);
        self.painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(14.0 * self.zoom.clamp(0.7, 1.2)),
            egui::Color32::BLACK,
        );
    }

    fn draw_tooltip(&mut self, input: &NodeRenderInput) {
        let node_id = self.ui.id().with(input.person_id);
        let node_response = self.ui.interact(input.rect, node_id, egui::Sense::hover());
        if node_response.hovered() {
            let tooltip_text =
                LayoutEngine::person_tooltip(self.tree, input.person_id, self.language);
            node_response.on_hover_text(tooltip_text);
        }
    }
}