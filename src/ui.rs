use crate::*;

pub struct Ui {
    is_clicked:bool, // Checks if the ui is being clicked/hovered over
}

impl Ui {

    pub fn new() -> Self {
        Self {
            is_clicked:false
        }
    }
    
    pub fn is_clicked(&self) -> bool {
        self.is_clicked
    }

    pub fn draw(&mut self,name:&mut Option<String>,selected_state: &mut DrawState,canvas:&mut Canvas) {
        

        use native_dialog::*;
        // Top- Bar
        egui_macroquad::ui(|egui_ctx| {
            self.is_clicked = egui_ctx.wants_pointer_input() || egui_ctx.is_pointer_over_area();
            egui_macroquad::egui::TopBottomPanel::top("Menu").show(egui_ctx, |ui| {
                egui_macroquad::egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Save").clicked() {
                            if let Some(name)  = name {
                                canvas.save_image_with_name(name.as_str()); 
                            }else  {
                                let path = FileDialog::new()
                                    .set_location("~/Desktop")
                                    .add_filter("PNG Image", &["png"])
                                    .add_filter("JPEG Image", &["jpg", "jpeg"])
                                    .show_save_single_file()
                                    .unwrap();
                                
                                if let Some(n) = path {
                                    let n = n.as_os_str().to_str().unwrap();
                                    *name = Some(n.to_string());
                                    canvas.save_image_with_name(n);
                                }
                            }
                        }

                        if ui.button("Save As").clicked() {
                            let path =  FileDialog::new()
                                    .set_location("~/Desktop")
                                    .add_filter("PNG Image", &["png"])
                                    .add_filter("JPEG Image", &["jpg", "jpeg"])
                                    .show_save_single_file()
                                    .unwrap();
                            if let Some(new_n) =  path {
                                let n = new_n.as_os_str().to_str().unwrap();
                                *name = Some(n.to_string());
                                canvas.save_image_with_name(n);
                            }
                        }
                        
                        if ui.button("Quit").clicked() {
                            std::process::exit(0);
                        }
                    });

                });


            });

                        

            egui_macroquad::egui::Window::new("Tools").show(egui_ctx, |ui| {

                use DrawState::*;

                let tools = [SinglePixel{color:BLACK}, RectangleBorder { thickness: 1, color: BLACK }, RectangleFill{color:BLACK}, Erase,Fill{color:BLACK}];
                for tool in tools.iter() {
                    if ui.button(tool.to_string()).clicked() {
                        if let Some(saved_tool) = canvas.saved_settings().get(tool) {
                            *selected_state = saved_tool.clone();
                        }else {
                            *selected_state = tool.clone();
                            canvas.saved_settings_mut().add(tool.clone());
                        }
                    }
                }

            });


            egui_macroquad::egui::Window::new("Properties").show(egui_ctx, |ui| {
                use DrawState::*;

                match *selected_state {
                    Erase => {
                        ui.label("Unimplemented Size");
                    }
                    SinglePixel{mut color} => {
                        ui.label("Color:");
                            let mut c = color.to_vec().to_array();
                        ui.color_edit_button_rgba_unmultiplied(&mut c);
                        color = Color::from_vec(Vec4::from_array(c));
                        *selected_state = SinglePixel{ color };                 
                    }
                    RectangleFill {mut color } => {
                        let mut c = color.to_vec().to_array();
                        ui.label("Color:");
                        ui.color_edit_button_rgba_unmultiplied(&mut c);
                        color = Color::from_vec(Vec4::from_array(c));
                        *selected_state = RectangleFill {color };
                    }
                    RectangleBorder { thickness,mut color } => {
                        let mut c = color.to_vec().to_array();
                        ui.label("Color:");
                        ui.color_edit_button_rgba_unmultiplied(&mut c);
                        color = Color::from_vec(Vec4::from_array(c));
                        *selected_state = RectangleBorder{ thickness, color };
                        // Handle RectangleBorder case here if needed
                    }
                    Fill{mut color} => {
                        let mut c = color.to_vec().to_array();
                        ui.label("Color:");
                        ui.color_edit_button_rgba_unmultiplied(&mut c);
                        color = Color::from_vec(Vec4::from_array(c));
                        *selected_state = Fill{color };

                    }
                    _ => unimplemented!()
                }
                canvas.saved_settings_mut().set(selected_state); 
            });
 
        });

    }

}


