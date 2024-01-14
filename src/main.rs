use macroquad::prelude::*;

mod ui;
mod canvas;
mod canvas_misc;
mod project;
mod undo_redo;

use ui::*;
use project::*;
use canvas::*;
use canvas_misc::*;
use undo_redo::*;

pub const MAX_ZOOM_POSITIVE:f32 = 3.0;
pub const MAX_ZOOM_NEGATIVE:f32 = 0.1;
pub const ZOOM_ADD:f32 = 0.2;



#[derive(Copy,Clone,Debug)]
pub enum DrawState {
    SinglePixel{color:Color},
    RectangleBorder{thickness:u8,color:Color},
    RectangleFill{color:Color},
    Erase,
    Fill{color:Color}
}

impl Default for DrawState {
    fn default() -> Self {
        Self::SinglePixel{color:BLACK}
    }
}


impl PartialEq for DrawState {
    fn eq(&self, other: &DrawState) -> bool {

        use std::mem;
        mem::discriminant(self) == mem::discriminant(other)
    }
}


impl std::fmt::Display for DrawState {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DrawState::*;
        match self {
            SinglePixel{..} => write!(f,"Single Pixel"),
            RectangleBorder{..} => write!(f,"Rectangle Border"),
            RectangleFill{..} => write!(f, "Rectangle Fill"),
            Erase => write!(f,"Erase"),
            Fill{..} => write!(f,"Fill"),
        }
    }
}


enum AppState {
    MainMenu,
    InProject(Project),
    MainMenuShowCreatePage,
}

#[macroquad::main("Pixel Art Maker")]
async fn main() {

    let mut app_state = AppState::MainMenu;
    let mut my_string = "".to_string();
    let mut dimension:u8= 16;
    loop {

        match app_state{
            AppState::MainMenu => {
                
                egui_macroquad::ui(|ctx| {
                    egui_macroquad::egui::Area::new("Main Menu")
                        .fixed_pos(egui::pos2(32.0, 32.0))
                        .show(ctx, |ui| {
                            if ui.button("Create new Project").clicked() {
                                app_state = AppState::MainMenuShowCreatePage;
                            }
                        });

                });

                egui_macroquad::draw();
            }

            AppState::MainMenuShowCreatePage => {
               egui_macroquad::ui(|ctx| {
                    egui_macroquad::egui::Area::new("Main Menu")
                        .fixed_pos(egui::pos2(32.0, 32.0))
                        .show(ctx, |ui| {
                            egui_macroquad::egui::Window::new("Create Project").show(ctx,|ui| {
                                ui.label("Project Name:");
                                let _response = ui.add(egui::TextEdit::singleline(&mut my_string));
                                ui.label("Dimensions:");
                                ui.add(egui::Slider::new(&mut dimension,16..=127));
                                if ui.button("Finish Project").clicked() {
                                    
                                    let project = if my_string.is_empty() {
                                        Project::new_untitled((dimension,dimension))
                                    }else {
                                    my_string = format!("{}.png",my_string);
                                     Project::new_titled(&my_string,(dimension,dimension))
                                    };
                                    app_state = AppState::InProject(project);
                                }
                            });
                        });
                });
                egui_macroquad::draw();
            }

            AppState::InProject(ref mut project) => {

                project.backend(); 

                clear_background(DARKGRAY);

                project.frontend();

                    egui_macroquad::draw();

            }
        }
        next_frame().await;
    }
}
