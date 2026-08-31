use crate::canvas::{CanvasEdge, CanvasNode, CanvasNodeData, EdgeStyle, Position, ViewState};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// Renderer for the canvas using Canvas 2D API
pub struct CanvasRenderer {
    ctx: CanvasRenderingContext2d,
    width: f64,
    height: f64,
}

impl CanvasRenderer {
    pub fn new(canvas: &HtmlCanvasElement) -> Result<Self, String> {
        let ctx = canvas
            .get_context("2d")
            .map_err(|e| format!("Failed to get 2d context: {:?}", e))?
            .ok_or_else(|| "Failed to get 2d context".to_string())?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "Failed to cast to CanvasRenderingContext2d".to_string())?;

        let width = canvas.width() as f64;
        let height = canvas.height() as f64;

        Ok(Self { ctx, width, height })
    }

    pub fn resize(&mut self, width: f64, height: f64) {
        self.width = width;
        self.height = height;
    }

    /// Clear the entire canvas
    pub fn clear(&self) {
        self.ctx.clear_rect(0.0, 0.0, self.width, self.height);
    }

    /// Draw grid background
    pub fn draw_grid(&self, view: &ViewState) {
        let grid_size = 40.0 * view.zoom;
        if grid_size < 8.0 {
            return;
        }

        self.ctx.set_stroke_style_str("#E5E7EB");
        self.ctx.set_line_width(0.5);

        let start_x = (view.offset_x % grid_size + grid_size) % grid_size;
        let start_y = (view.offset_y % grid_size + grid_size) % grid_size;

        let mut x = start_x;
        while x < self.width {
            self.ctx.begin_path();
            self.ctx.move_to(x, 0.0);
            self.ctx.line_to(x, self.height);
            self.ctx.stroke();
            x += grid_size;
        }

        let mut y = start_y;
        while y < self.height {
            self.ctx.begin_path();
            self.ctx.move_to(0.0, y);
            self.ctx.line_to(self.width, y);
            self.ctx.stroke();
            y += grid_size;
        }
    }

    /// Render all nodes and edges
    pub fn render(&self, nodes: &[CanvasNode], edges: &[CanvasEdge], view: &ViewState) {
        self.clear();
        self.draw_grid(view);

        self.ctx.save();
        self.ctx
            .translate(view.offset_x, view.offset_y)
            .unwrap_or(());
        self.ctx.scale(view.zoom, view.zoom).unwrap_or(());

        for edge in edges {
            self.draw_edge(edge, nodes);
        }

        for node in nodes {
            self.draw_node(node);
        }

        self.ctx.restore();
    }

    fn draw_node(&self, node: &CanvasNode) {
        let x = node.position.x;
        let y = node.position.y;
        let w = node.width();
        let h = node.height();

        match &node.data {
            CanvasNodeData::Text(d) => {
                // Background
                self.ctx.set_fill_style_str("#F3F4F6");
                self.ctx.fill_rect(x, y, w, h);
                self.ctx.set_stroke_style_str("#D1D5DB");
                self.ctx.set_line_width(1.0);
                self.ctx.stroke_rect(x, y, w, h);

                // Text
                self.ctx.set_fill_style_str(&d.color);
                self.ctx.set_font(&format!("{}px sans-serif", d.font_size));
                self.ctx.set_text_baseline("middle");
                let _ = self.ctx.fill_text(&d.content, x + 8.0, y + h / 2.0);
            }
            CanvasNodeData::Image(d) => {
                // Placeholder box
                self.ctx.set_fill_style_str("#F9FAFB");
                self.ctx.fill_rect(x, y, w, h);
                self.ctx.set_stroke_style_str("#D1D5DB");
                self.ctx.set_line_width(1.0);
                self.ctx.stroke_rect(x, y, w, h);

                // Image icon placeholder
                self.ctx.set_fill_style_str("#9CA3AF");
                self.ctx.set_font("12px sans-serif");
                self.ctx.set_text_baseline("middle");
                let label = if d.alt.is_empty() { "Image" } else { &d.alt };
                let _ = self.ctx.fill_text(label, x + 8.0, y + h / 2.0);
            }
            CanvasNodeData::Link(d) => {
                self.ctx.set_fill_style_str("#EFF6FF");
                self.ctx.fill_rect(x, y, w, h);
                self.ctx.set_stroke_style_str("#3B82F6");
                self.ctx.set_line_width(1.0);
                self.ctx.stroke_rect(x, y, w, h);

                self.ctx.set_fill_style_str("#1D4ED8");
                self.ctx.set_font("bold 12px sans-serif");
                self.ctx.set_text_baseline("top");
                let title = if d.title.is_empty() { &d.url } else { &d.title };
                let _ = self.ctx.fill_text(title, x + 8.0, y + 8.0);

                if !d.description.is_empty() {
                    self.ctx.set_fill_style_str("#6B7280");
                    self.ctx.set_font("10px sans-serif");
                    let _ = self.ctx.fill_text(&d.description, x + 8.0, y + 28.0);
                }
            }
            CanvasNodeData::Document(d) => {
                self.ctx.set_fill_style_str("#F0FDF4");
                self.ctx.fill_rect(x, y, w, h);
                self.ctx.set_stroke_style_str("#22C55E");
                self.ctx.set_line_width(1.0);
                self.ctx.stroke_rect(x, y, w, h);

                self.ctx.set_fill_style_str("#166534");
                self.ctx.set_font("bold 12px sans-serif");
                self.ctx.set_text_baseline("top");
                let _ = self.ctx.fill_text(&d.title, x + 8.0, y + 8.0);

                self.ctx.set_fill_style_str("#6B7280");
                self.ctx.set_font("10px sans-serif");
                let _ = self.ctx.fill_text("Document", x + 8.0, y + 28.0);
            }
            CanvasNodeData::Shape(d) => match d.shape_type {
                crate::canvas::ShapeType::Rectangle => {
                    self.ctx.set_fill_style_str(&d.fill);
                    self.ctx.fill_rect(x, y, w, h);
                    self.ctx.set_stroke_style_str(&d.stroke);
                    self.ctx.set_line_width(2.0);
                    self.ctx.stroke_rect(x, y, w, h);
                }
                crate::canvas::ShapeType::Circle => {
                    let cx = x + w / 2.0;
                    let cy = y + h / 2.0;
                    let r = w.min(h) / 2.0;
                    self.ctx.begin_path();
                    self.ctx
                        .arc(cx, cy, r, 0.0, std::f64::consts::PI * 2.0)
                        .unwrap_or(());
                    self.ctx.set_fill_style_str(&d.fill);
                    self.ctx.fill();
                    self.ctx.set_stroke_style_str(&d.stroke);
                    self.ctx.set_line_width(2.0);
                    self.ctx.stroke();
                }
                crate::canvas::ShapeType::Diamond => {
                    let cx = x + w / 2.0;
                    let cy = y + h / 2.0;
                    self.ctx.begin_path();
                    self.ctx.move_to(cx, y);
                    self.ctx.line_to(x + w, cy);
                    self.ctx.line_to(cx, y + h);
                    self.ctx.line_to(x, cy);
                    self.ctx.close_path();
                    self.ctx.set_fill_style_str(&d.fill);
                    self.ctx.fill();
                    self.ctx.set_stroke_style_str(&d.stroke);
                    self.ctx.set_line_width(2.0);
                    self.ctx.stroke();
                }
            },
        }
    }

    fn draw_edge(&self, edge: &CanvasEdge, nodes: &[CanvasNode]) {
        let source = match nodes.iter().find(|n| n.id == edge.source_id) {
            Some(n) => n,
            None => return,
        };
        let target = match nodes.iter().find(|n| n.id == edge.target_id) {
            Some(n) => n,
            None => return,
        };

        let sx = source.center().x;
        let sy = source.center().y;
        let tx = target.center().x;
        let ty = target.center().y;

        self.ctx.set_stroke_style_str(edge.color());
        self.ctx.set_line_width(2.0);

        match edge.style() {
            EdgeStyle::Solid => {}
            EdgeStyle::Dotted => {
                let arr = js_sys::Array::new();
                arr.push(&JsValue::from_f64(4.0));
                arr.push(&JsValue::from_f64(4.0));
                self.ctx.set_line_dash(&arr.into()).unwrap_or(());
            }
            EdgeStyle::Dashed => {
                let arr = js_sys::Array::new();
                arr.push(&JsValue::from_f64(8.0));
                arr.push(&JsValue::from_f64(4.0));
                self.ctx.set_line_dash(&arr.into()).unwrap_or(());
            }
        }

        self.ctx.begin_path();
        self.ctx.move_to(sx, sy);
        self.ctx.line_to(tx, ty);
        self.ctx.stroke();
        self.ctx
            .set_line_dash(&js_sys::Array::new().into())
            .unwrap_or(());

        if edge.has_arrowhead() {
            self.draw_arrowhead(sx, sy, tx, ty);
        }
    }

    fn draw_arrowhead(&self, sx: f64, sy: f64, tx: f64, ty: f64) {
        let angle = (ty - sy).atan2(tx - sx);
        let arrow_len = 12.0;
        let arrow_angle = std::f64::consts::PI / 6.0;

        let ax = tx - arrow_len * (angle - arrow_angle).cos();
        let ay = ty - arrow_len * (angle - arrow_angle).sin();
        let bx = tx - arrow_len * (angle + arrow_angle).cos();
        let by = ty - arrow_len * (angle + arrow_angle).sin();

        self.ctx.begin_path();
        self.ctx.move_to(tx, ty);
        self.ctx.line_to(ax, ay);
        self.ctx.move_to(tx, ty);
        self.ctx.line_to(bx, by);
        self.ctx.stroke();
    }

    /// Convert screen coordinates to canvas coordinates
    pub fn screen_to_canvas(&self, sx: f64, sy: f64, view: &ViewState) -> Position {
        Position::new(
            (sx - view.offset_x) / view.zoom,
            (sy - view.offset_y) / view.zoom,
        )
    }

    /// Hit test: find which node is at the given screen coordinates
    pub fn hit_test(
        &self,
        sx: f64,
        sy: f64,
        nodes: &[CanvasNode],
        view: &ViewState,
    ) -> Option<String> {
        let canvas_pos = self.screen_to_canvas(sx, sy, view);
        for node in nodes.iter().rev() {
            if node.contains_point(canvas_pos.x, canvas_pos.y) {
                return Some(node.id.clone());
            }
        }
        None
    }
}
