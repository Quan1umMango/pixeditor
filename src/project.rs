use crate::{Canvas, FillType, MoveCameraData, Tool};
use crate::{
    ERASER_ICON, ICON_SIZE, MAX_SCROLL, MAX_SCROLL_NEG, MOUSE_MOVE_BY, MOVE_CAMERA_KEY,
    PAINT_BRUSH_ICON, SCROLL_BY,
};
use egui_macroquad::egui;
use macroquad::prelude::*;
use macroquad::ui::root_ui;

/// Overall management of the project, such as undo-redo, saving, opening new files etc.
#[derive(Default)]
pub struct Project {
    project_name: String,
    num_pixels: usize,
    /// If the user input (like mouse-click) was processed by the ui.
    ui_processed: bool,

    canvas: Canvas,
    camera: Camera2D,
    move_camera_data: Option<MoveCameraData>,

    save_options:SaveOptions
}

impl Project {
    pub fn new(project_name: String, num_pixels: usize) -> Self {
        Self {
            project_name:project_name.clone(),
            num_pixels,
            canvas: Canvas::new(num_pixels),
            save_options:SaveOptions::with_name(project_name),
            ..Default::default()
        }
    }

    pub fn handle_input(&mut self) {
        if self.ui_processed {
            return;
        }

        self.handle_scroll();

        if is_key_down(MOVE_CAMERA_KEY) {
            if self.move_camera_data.is_none() {
                self.move_camera_data = Some(MoveCameraData {
                    move_start: mouse_position().into(),
                    move_by: (0., 0.).into(),
                });
            } else {
                let new_pos: Vec2 = mouse_position().into();
                let delta = self.move_camera_data.as_ref().unwrap().move_start - new_pos;
                self.move_camera_data.as_mut().unwrap().move_by = delta;
                self.move_camera_data.as_mut().unwrap().move_start = new_pos;

                self.camera.offset += delta * MOUSE_MOVE_BY;
            }
            return;
        } else {
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
        let (_, sy) = mouse_wheel();
        if sy == 0.0 {
            return;
        }

        if sy > 0.0 {
            if !(self.camera.zoom.x < MAX_SCROLL.x && self.camera.zoom.y < MAX_SCROLL.y) {
                self.camera.zoom = MAX_SCROLL;
                return;
            }
        } else if !(self.camera.zoom.x > MAX_SCROLL_NEG.x && self.camera.zoom.y > MAX_SCROLL_NEG.y)
        {
            self.camera.zoom = MAX_SCROLL_NEG;
            return;
        }
        self.camera.zoom += sy * SCROLL_BY;
    }

    pub fn draw_ui(&mut self) {
        egui_macroquad::ui(|egui_ctx| {
            egui_extras::install_image_loaders(egui_ctx);

            self.ui_processed = egui_ctx.wants_pointer_input() || egui_ctx.is_pointer_over_area();


            // Top Menu Bar
            egui::TopBottomPanel::top("top bar").show(egui_ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Save").clicked() {
                            self.handle_save();
                        }
                        if ui.button("Save As").clicked() {
                            self.handle_save_as();
                        }
                    });                 
                });

            });

            // Tools
            egui::Window::new("Tools").show(egui_ctx, |ui| {
                let single_pixel_btn = egui::ImageButton::new(PAINT_BRUSH_ICON);
                let eraser_btn = egui::ImageButton::new(ERASER_ICON);
                let single_pixel_resp = ui.add_sized(ICON_SIZE, single_pixel_btn);
                if single_pixel_resp.clicked() {
                    self.canvas.set_tool(Tool::pixel());
                }
                let eraser_resp = ui.add_sized(ICON_SIZE, eraser_btn);
                if eraser_resp.clicked() {
                    self.canvas.set_tool(Tool::eraser());
                }

                if ui.button("rect borders").clicked() {
                    self.canvas.set_tool(Tool::rect(FillType::NoFill));
                }

                if ui.button("filled rect").clicked() {
                    self.canvas.set_tool(Tool::rect(FillType::SolidFill));
                }

                if ui.button("flood fill").clicked() {
                    self.canvas.set_tool(Tool::fill());
                }
                if ui.button("line").clicked() {
                    self.canvas.set_tool(Tool::line());
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

    pub fn handle_save(&mut self) {
       if self.save_options.name.is_none() || self.save_options.path.is_none() {
            self.handle_save_as();
            return;
       }
       self.save();
    }

    pub fn handle_save_as(&mut self) {
        use native_dialog::DialogBuilder;
        use std::path::Path;

        let path_res = DialogBuilder::file()
            .set_location("~/Desktop")
            // Currently using macroquad::texture::Image which only supports PNG
            // TODO: Switch to use Image crate and add other extension support
            .add_filter("PNG Image", &["png"])
            .set_filename(self.save_options.name.as_ref().unwrap_or(&String::new()))
            .save_single_file()
            .show()
            .unwrap();

        // User clicked close on save window
        if path_res.is_none() { return; }

        let path_v = path_res.unwrap();
        let path = Path::new(path_v.as_os_str());
        let file_name = path.file_name().unwrap().to_os_string().into_string().unwrap();
        let path_string = path.as_os_str().to_os_string().into_string().unwrap();
        self.save_options.set_name(file_name);
        self.save_options.set_path(path_string);

        self.save();
    }

    pub fn save(&mut self) {
        let img = self.canvas.to_image();
        // TODO: Handle these errors
        img.export_png(self.save_options.path.as_ref().unwrap());
    }
}

#[derive(Default)]
pub struct SaveOptions {
    name:Option<String>,
    path:Option<String>
    //ext:Option<String>,
}

impl SaveOptions {
    pub fn with_name(name:String) -> Self {
        Self{ 
            name: if name.is_empty() { None } else { Some(name) },
            ..Default::default() 
        }
    }

    pub fn set_name(&mut self, name:String) {
        self.name = if name.is_empty() { None } else { Some(name) };
    }

    pub fn set_path(&mut self, path:String) {
        self.path = if path.is_empty() { None } else { Some(path) };
    }

}
