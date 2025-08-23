use macroquad::prelude::*;
use egui_macroquad::egui;
use macroquad::ui::root_ui;
use crate::{ Canvas, MoveCameraData,  DrawTool, ToolInfo, FillType };
use crate::{
    MOVE_CAMERA_KEY, MOUSE_MOVE_BY, ICON_SIZE, ERASER_ICON, PAINT_BRUSH_ICON, SCROLL_BY, MAX_SCROLL, MAX_SCROLL_NEG,
};


/// Overall management of the project, such as undo-redo, saving, opening new files etc.
#[derive(Default)]
pub struct Project {
    project_name:String,
    num_pixels:usize,
    /// If the user input (like mouse-click) was processed by the ui.
    ui_processed:bool,

    canvas:Canvas,
    camera:Camera2D,
    move_camera_data:Option<MoveCameraData>,
}

impl Project {
    pub fn new(project_name:String,num_pixels:usize) -> Self {
        Self {
            project_name,
            num_pixels,
            canvas:Canvas::new(num_pixels),
            ..Default::default()
        }
    }

    pub fn handle_input(&mut self) {
        if self.ui_processed { return; }

        self.handle_scroll();

        if is_key_down(MOVE_CAMERA_KEY) {
           if self.move_camera_data.is_none() { self.move_camera_data = Some(MoveCameraData{ move_start: mouse_position().into(), move_by: (0.,0.).into() }); }
           else {
               let new_pos:Vec2 = mouse_position().into();
               let delta = self.move_camera_data.as_ref().unwrap().move_start - new_pos;
               self.move_camera_data.as_mut().unwrap().move_by = delta;
               self.move_camera_data.as_mut().unwrap().move_start = new_pos;

               self.camera.offset += delta  * MOUSE_MOVE_BY;
           }
           return;
        }else {
            self.move_camera_data = None;
        }

        if !root_ui().active_window_focused() {
        self.use_tool();
        }
    }

    pub fn use_tool(&mut self) {
       self.canvas.use_tool_at_mouse_position(&self.camera); 
    }

    pub fn handle_scroll(&mut self) {
        // mouse_wheel() returns if currently the user is scrolling. If its 0.0, then the user is
        // not scrolling, else (if its -1.0 or 1.0) the user is scrolling
        let (_,sy) = mouse_wheel();
        if sy == 0.0 { return }

        if sy > 0.0 {
            if !(self.camera.zoom.x < MAX_SCROLL.x && self.camera.zoom.y < MAX_SCROLL.y) {
                self.camera.zoom = MAX_SCROLL;
                return;
            }
        }else if !(self.camera.zoom.x > MAX_SCROLL_NEG.x && self.camera.zoom.y > MAX_SCROLL_NEG.y) {
            self.camera.zoom =  MAX_SCROLL_NEG;
            return;
        }
        self.camera.zoom += sy * SCROLL_BY;

    }

    pub fn draw_ui(&mut self) {
        egui_macroquad::ui(|egui_ctx| {

            egui_extras::install_image_loaders(egui_ctx);

            self.ui_processed =  egui_ctx.wants_pointer_input() || egui_ctx.is_pointer_over_area();

            egui::Window::new("Tools")
            .show(egui_ctx, |ui| {
                let single_pixel_btn = egui::ImageButton::new(PAINT_BRUSH_ICON);
                let eraser_btn = egui::ImageButton::new(ERASER_ICON);
                let single_pixel_resp = ui.add_sized(ICON_SIZE,single_pixel_btn);
                if single_pixel_resp.clicked() {
                    self.canvas.set_tool(DrawTool::Pixel);
                }
                let eraser_resp = ui.add_sized(ICON_SIZE,eraser_btn);
                if eraser_resp.clicked() {
                    self.canvas.set_tool(DrawTool::Eraser);
                }

                if ui.button("rect borders").clicked() {
                    self.canvas.set_tool(DrawTool::Rect(ToolInfo::default(),FillType::NoFill));
                }

                if ui.button("filled rect").clicked() {
                    self.canvas.set_tool(DrawTool::Rect(ToolInfo::default(),FillType::SolidFill));
                }

                if ui.button("flood fill").clicked() {
                    self.canvas.set_tool(DrawTool::Fill);
                }
                if ui.button("line").clicked() {
                    self.canvas.set_tool(DrawTool::Line(ToolInfo::default()));
                }
            });

            egui::Window::new("Color").show(egui_ctx, |ui| {
                let mut c = self.canvas.selected_color().to_vec().to_array();
                ui.color_edit_button_rgba_unmultiplied(&mut c);
                self.canvas.set_selected_color(Color::from_vec(c.into()));
            });
        });

        egui_macroquad::draw();

    }

    pub fn draw(&mut self) {
        set_camera(&self.camera);
        self.canvas.draw(&self.camera);
        self.draw_ui();
    }

}
