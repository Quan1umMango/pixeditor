use egui_macroquad::egui;
use macroquad::prelude::*;
use macroquad::ui::{Skin, root_ui, widgets};

mod undo_redo;
mod canvas;
mod project;
mod tool_helper;

use canvas::Canvas;
use project::Project;

const MAX_PIXELS: usize = 64;
const BG_COLOR: Color = Color::from_rgba(90, 90, 90, 255);

const MOVE_CAMERA_KEY: KeyCode = KeyCode::LeftControl;

const RECT_DIMS: Vec2 = vec2(0.1, 0.1);

const MAX_SCROLL: Vec2 = vec2(3., 3.);
const MAX_SCROLL_NEG: Vec2 = vec2(0.1, 0.1);
const SCROLL_BY: f32 = 0.1;
const MOUSE_MOVE_BY: f32 = 0.01;

const ICON_SIZE: [f32; 2] = [25., 25.];

const ERASER_ICON: egui::widgets::ImageSource = egui::include_image!("../assets/icons/eraser.png");
const PAINT_BRUSH_ICON: egui::widgets::ImageSource =
    egui::include_image!("../assets/icons/paintbrush.png");

#[derive(Default)]
pub enum AppState {
    #[default]
    MainMenu,
    NewMenu(NewMenuState),
    Drawing(Box<Project>),
}

#[derive(Default, Clone)]
pub struct NewMenuState {
    file_name: String,
    num_pixels_string: String,
}

#[derive(Default)]
pub struct App {
    state: AppState,
}

impl App {
    /// Handles only KEYBOARD AND MOUSE inputs and not ui.
    pub fn handle_input(&mut self) {
        match &mut self.state {
            AppState::Drawing(proj) => proj.handle_input(),
            _ => {}
        }
    }

    // deceptive name, since it not only draws but updates `state`
    pub fn draw(&mut self) {
        let window_size = vec2(screen_width(), screen_height());

        let btn_size = vec2(260., 55.);
        let mut new_state: Option<AppState> = None;

        match self.state {
            AppState::MainMenu => {
                // TODO: Make this cleaner, add title maybe
                let padding = 50.;

                let new_btn = widgets::Button::new("New Project")
                    .size(btn_size)
                    .position(window_size / 2.0 - btn_size / 2.0 - vec2(0., padding));
                let open_old = widgets::Button::new("Open New")
                    .size(btn_size)
                    .position(vec2(
                        window_size.x / 2.0 - btn_size.x / 2.0,
                        window_size.y / 2.0 + btn_size.x / 4.0 - padding,
                    ));

                if new_btn.ui(&mut root_ui()) {
                    new_state = Some(AppState::NewMenu(NewMenuState::default()));
                }
                if open_old.ui(&mut root_ui()) {
                    // TODO: Handle this.
                }
            }

            AppState::NewMenu(ref mut st) => {
                egui_macroquad::ui(|egui_ctx| {
                    egui::Window::new("Create New Project")
                        .movable(false)
                        .resizable(false)
                        .collapsible(false)
                        .anchor(egui::Align2::CENTER_CENTER, (-25., -50.))
                        .show(egui_ctx, |ui| {
                            let file_name_input = egui::TextEdit::singleline(&mut st.file_name);
                            let num_pixels_input =
                                egui::TextEdit::singleline(&mut st.num_pixels_string);

                            ui.label("Enter project name:");
                            let _ = ui.add(file_name_input);

                            ui.label("Enter project dimensions");
                            if ui.add(num_pixels_input).changed() {
                                // HACK
                                if let Ok(v) = st.num_pixels_string.parse::<usize>() {
                                    if v > MAX_PIXELS {
                                        st.num_pixels_string = MAX_PIXELS.to_string();
                                    }
                                } else {
                                    st.num_pixels_string.clear();
                                }
                            }

                            if ui.button("Done!").clicked() {
                                if let Ok(v) = st.num_pixels_string.parse::<usize>() {
                                    new_state = Some(AppState::Drawing(Box::new(Project::new(
                                        st.file_name.clone(),
                                        v,
                                    ))));
                                }
                            }
                        });
                });
                egui_macroquad::draw();
            }

            AppState::Drawing(ref mut proj) => {
                proj.draw();
            }
        }
        if let Some(ns) = new_state {
            self.state = ns;
        }
    }

    pub fn set_state(&mut self, new_state: AppState) {
        self.state = new_state;
    }
}

#[derive(Default)]
pub struct MoveCameraData {
    move_start: Vec2,
    move_by: Vec2,
}

#[derive(Debug, Default, Clone)]
pub struct Tool {
    kind: ToolKind,
    // Some tools will use this completely (such as line), others partially or none at all
    info: ToolInfo,
}

#[derive(Debug, Default, Copy, Clone)]
pub enum ToolKind {
    #[default]
    Pixel,
    Eraser,
    Rect(FillType),
    Fill,
    Line,
}

impl Tool {
    pub fn new(kind: ToolKind, info: ToolInfo) -> Tool {
        Tool { kind, info }
    }

    pub fn pixel() -> Tool {
        Tool::new(ToolKind::Pixel, ToolInfo::default())
    }
    pub fn eraser() -> Tool {
        Tool::new(ToolKind::Eraser, ToolInfo::default())
    }
    pub fn rect(t: FillType) -> Tool {
        Tool::new(ToolKind::Rect(t), ToolInfo::default())
    }
    pub fn fill() -> Tool {
        Tool::new(ToolKind::Fill, ToolInfo::default())
    }
    pub fn line() -> Tool {
        Tool::new(ToolKind::Line, ToolInfo::default())
    }
}

/*

impl DrawTool {
    pub fn info(&self) -> Option<&ToolInfo> {
        use DrawTool::*;
        match self {
            Rect(info,_) | Line(info) => Some(info),
            _ => None
        }
    }

    pub fn set_info(&mut self, tinfo:ToolInfo) {
        use DrawTool::*;
        match self {
            Rect(info,_) | Line(info) => { *info = tinfo; },
            _ => ()
        }
    }

    /// Returns the previous value of tool info before being swapped (if any, else None)
    pub fn replace_info(&mut self, new_info:ToolInfo) -> Option<ToolInfo> {
        use DrawTool::*;
        match self {
            Rect(info,_) | Line(info) => {
                let mut ni = new_info;
                std::mem::swap(info, &mut ni);
                return Some(ni);
            },
            _ => None
        }
    }
}
*/

#[derive(Debug, Clone, Copy)]
pub enum FillType {
    NoFill,
    SolidFill,
}

/// Information about the initial and current/final location of the mouse click when the tool is
/// being used, and also the list of the indices of the pixel which are to be drawn over.
/// If the initial or the start location is None, then it means that the tool is/was not being used.
#[derive(Debug, Default, Clone)]
pub struct ToolInfo {
    /// Initial position of the mouse-click
    initial_loc: Option<Vec2>,
    /// Final/Current position of the mouse-click
    final_loc: Option<Vec2>,
    /// The indices of the pixel which are to be drawn on.
    pixel_indices: Option<Vec<usize>>,
}

#[derive(Default)]
pub struct Layer {
    num_pixels: usize,
    data: Vec<Option<Color>>,
}

impl Layer {
    pub fn new(num_pixels: usize) -> Self {
        let mut d = Vec::<Option<Color>>::new();
        for _ in 0..(num_pixels * num_pixels) {
            d.push(None);
        }
        Self {
            num_pixels,
            data: d,
        }
    }

    /// Panics: If index >= self.data.len()
    pub fn set_pixel(&mut self, index: usize, c: Option<Color>) {
        self.data[index] = c;
    }

    /// Panics: If index >= self.data.len()
    pub fn set_pixel_color(&mut self, index: usize, c: Color) {
        self.data[index] = Some(c);
    }

    /// Flips the image along y-axis. See unflipped version for this function ot achieve otherwise
    pub fn to_image(&self) -> Image {
        let mut img = Image::gen_image_color(self.num_pixels as u16,self.num_pixels as u16,Color::from_rgba(0,0,0,0)); 
        for (i,p) in self.data.iter().enumerate() {
            let x = (i % self.num_pixels) as u32;
            // We have to do this because otherwise the image is upside down
            let y = (self.num_pixels - 1 - (i / self.num_pixels)) as u32;

            if let Some(c) = p {
                img.set_pixel(x,y,*c);
            }
        }

       img
    }

    pub fn to_image_unflipped(&self) -> Image {
        let mut img = Image::gen_image_color(self.num_pixels as u16,self.num_pixels as u16,Color::from_rgba(0,0,0,0)); 
        for (i,p) in self.data.iter().enumerate() {
            let x = (i % self.num_pixels) as u32;
            let y = (i / self.num_pixels) as u32;

            if let Some(c) = p {
                img.set_pixel(x,y,*c);
            }
        }

       img
    }

}

#[macroquad::main("Pixeditor")]
async fn main() {
    let mut app = App::default();

    let window_style = root_ui()
        .style_builder()
        .text_color(WHITE)
        .color(GRAY)
        .build();

    let label_style = root_ui()
        .style_builder()
        .text_color(WHITE)
        .color(DARKGRAY)
        .build();
    let button_style = root_ui()
        .style_builder()
        .margin(RectOffset::new(-20.0, -30.0, 0.0, 0.0))
        .text_color(WHITE)
        .color(DARKGRAY)
        .build();

    let skin = Skin {
        window_style,
        label_style,
        button_style,
        ..root_ui().default_skin()
    };

    root_ui().push_skin(&skin);

    loop {
        app.handle_input();
        clear_background(BG_COLOR);
        app.draw();
        next_frame().await;
    }
}
