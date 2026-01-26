use crate::app::App;
use crate::ui::PanZoomHandler;

impl PanZoomHandler for App {
    fn handle_pan_zoom(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        pointer_pos: Option<egui::Pos2>,
        node_hovered: bool,
        any_node_dragged: bool,
        event_hovered: bool,
        any_event_dragged: bool,
    ) {
        let any_hovered = node_hovered || event_hovered;
        let any_dragged = any_node_dragged || any_event_dragged;
        let any_dragging = self.canvas.dragging_node.is_some() || self.canvas.dragging_event.is_some();
        
        if !any_hovered && !any_dragged && !any_dragging {
            if let Some(pos) = pointer_pos {
                let primary_down = ui.input(|i| i.pointer.primary_down());
                let primary_pressed = ui.input(|i| i.pointer.primary_pressed());
                
                if primary_pressed && rect.contains(pos) {
                    self.canvas.dragging_pan = true;
                    self.canvas.last_pointer_pos = Some(pos);
                }
                
                if self.canvas.dragging_pan && primary_down {
                    if let Some(prev) = self.canvas.last_pointer_pos {
                        self.canvas.pan += pos - prev;
                        self.canvas.last_pointer_pos = Some(pos);
                    }
                }
                
                if !primary_down && self.canvas.dragging_pan {
                    self.canvas.dragging_pan = false;
                    self.canvas.last_pointer_pos = None;
                }
            }
        } else if !any_dragged {
            self.canvas.dragging_pan = false;
            self.canvas.last_pointer_pos = None;
        }
    }
}
