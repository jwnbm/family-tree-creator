use crate::core::tree::PersonId;
use std::collections::HashMap;

// モジュール宣言
mod renderer;
mod node;
mod node_painter;
mod node_interaction;
mod pan_zoom;
mod edge;
mod family_box;
mod event_node;
mod event_relation;

/// キャンバスのメイン描画トレイト
pub trait CanvasRenderer {
    fn render_canvas(&mut self, ctx: &egui::Context);
}

/// ノード描画トレイト
pub trait NodeRenderer {
    fn render_canvas_nodes(
        &mut self,
        _ui: &mut egui::Ui,
        painter: &egui::Painter,
        nodes: &[crate::core::layout::LayoutNode],
        screen_rects: &HashMap<PersonId, egui::Rect>,
    );
}

/// ノードインタラクショントレイト
pub trait NodeInteractionHandler {
    fn handle_node_interactions(
        &mut self,
        ui: &mut egui::Ui,
        nodes: &[crate::core::layout::LayoutNode],
        screen_rects: &HashMap<PersonId, egui::Rect>,
        pointer_pos: Option<egui::Pos2>,
        origin: egui::Pos2,
    ) -> (bool, bool);
}

/// パン・ズーム処理トレイト
pub trait PanZoomHandler {
    fn handle_pan_zoom(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        pointer_pos: Option<egui::Pos2>,
        node_hovered: bool,
        any_node_dragged: bool,
        event_hovered: bool,
        any_event_dragged: bool,
    );
}

/// エッジ描画トレイト
pub trait EdgeRenderer {
    fn render_canvas_edges(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        screen_rects: &HashMap<PersonId, egui::Rect>,
    );
}

/// 家族の枠描画トレイト
pub trait FamilyBoxRenderer {
    fn render_family_boxes(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        screen_rects: &HashMap<PersonId, egui::Rect>,
    );
}

/// イベントノード描画トレイト
pub trait EventNodeRenderer {
    fn render_event_nodes(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        screen_rects: &HashMap<PersonId, egui::Rect>,
        pointer_pos: Option<egui::Pos2>,
    ) -> (bool, bool); // (event_hovered, any_event_dragged)
}

/// イベント関係線描画トレイト
pub trait EventRelationRenderer {
    fn render_event_relations(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        screen_rects: &HashMap<PersonId, egui::Rect>,
    );
}
