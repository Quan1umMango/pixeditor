/// Stores icon related consts.


use egui_macroquad::egui;


pub const ICON_SIZE: [f32; 2] = [25., 25.];

pub const ERASER_ICON: egui::widgets::ImageSource = egui::include_image!("../assets/icons/eraser.png");
pub const PAINT_BRUSH_ICON: egui::widgets::ImageSource =
    egui::include_image!("../assets/icons/paintbrush.png");
pub const RECT_BORDER_ICON: egui::widgets::ImageSource = egui::include_image!("../assets/icons/rect-border.png");
pub const RECT_FILLED_ICON: egui::widgets::ImageSource = egui::include_image!("../assets/icons/rect-filled.png");
pub const FILL_ICON: egui::widgets::ImageSource = egui::include_image!("../assets/icons/fill.png");
pub const LINE_ICON: egui::widgets::ImageSource = egui::include_image!("../assets/icons/slant-line.png");

