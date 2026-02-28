use std::collections::HashMap;

use crate::app::App;
use crate::core::tree::PersonId;
use crate::ui::NodeRenderer;

use super::node_painter::{node_color_theme_from_preset, NodePainter, NodeRenderInput};

impl App {
    fn build_node_render_input(
        &self,
        node: &crate::core::layout::LayoutNode,
        screen_rects: &HashMap<PersonId, egui::Rect>,
    ) -> Option<NodeRenderInput> {
        let rect = screen_rects.get(&node.id).copied()?;
        let is_selected = self.person_editor.selected == Some(node.id);
        let is_multi_selected = self.person_editor.selected_ids.contains(&node.id);
        let is_dragging = self.canvas.dragging_node == Some(node.id);

        let person = self.tree.persons.get(&node.id);

        Some(NodeRenderInput::from_person(
            node.id,
            rect,
            is_selected,
            is_multi_selected,
            is_dragging,
            person,
        ))
    }
}

impl NodeRenderer for App {
    fn render_canvas_nodes(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        nodes: &[crate::core::layout::LayoutNode],
        screen_rects: &HashMap<PersonId, egui::Rect>,
    ) {
        let render_inputs: Vec<NodeRenderInput> = nodes
            .iter()
            .filter_map(|node| self.build_node_render_input(node, screen_rects))
            .collect();

        let node_color_theme = node_color_theme_from_preset(self.ui.node_color_theme);
        let mut node_painter = NodePainter::new_with_theme(
            ui,
            painter,
            &self.tree,
            self.canvas.zoom,
            self.ui.language,
            &mut self.canvas.photo_texture_cache,
            node_color_theme,
        );

        for input in &render_inputs {
            node_painter.draw_node(input);
        }
    }
}


