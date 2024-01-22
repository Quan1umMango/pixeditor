use crate::*;

pub struct Ui {
    is_clicked: bool, // Checks if the ui is being clicked/hovered over
}

impl Ui {
    pub fn new() -> Self {
        Self {
            is_clicked: false,
        }
    }

    pub fn is_clicked(&self) -> bool {
        self.is_clicked
    }

    pub fn draw(&mut self, name: &mut Option<String>, selected_state: &mut DrawState, layers: &mut Layers) {
        use native_dialog::*;

        // Top- Bar
        egui_macroquad::ui(|egui_ctx| {
            self.is_clicked = egui_ctx.wants_pointer_input() || egui_ctx.is_pointer_over_area();
            egui_macroquad::egui::TopBottomPanel::top("Menu").show(egui_ctx, |ui| {
                egui_macroquad::egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New").clicked() {
                            println!("unimplemented pop for new");
                        }
                        if ui.button("Save").clicked() {
                            if let Some(canvas) = layers.get_current_layer_mut() {
                                if let Some(name) = name {
                                    canvas.save_image_with_name(name.as_str());
                                } else {
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
                        }

                        if ui.button("Save As").clicked() {
                            if let Some(canvas) = layers.get_current_layer_mut() {
                                let path = FileDialog::new()
                                    .set_location("~/Desktop")
                                    .add_filter("PNG Image", &["png"])
                                    .add_filter("JPEG Image", &["jpg", "jpeg"])
                                    .show_save_single_file()
                                    .unwrap();
                                if let Some(new_n) = path {
                                    let n = new_n.as_os_str().to_str().unwrap();
                                    *name = Some(n.to_string());
                                    canvas.save_image_with_name(n);
                                }
                            }
                        }

                        if ui.button("Save Selected Layers").clicked() {
                            if let Some(canvas) = layers.get_current_layer_mut() {
                               layers.save_combined_layers(); 
                            }
                        }

                        if ui.button("Quit").clicked() {
                            std::process::exit(0);
                        }
                    });
                });
            });

            if let Some(canvas) = layers.get_current_layer_mut() {
                egui_macroquad::egui::Window::new("Tools").show(egui_ctx, |ui| {
                    use DrawState::*;

                    let tools = [
                        SinglePixel { color: BLACK },
                        RectangleBorder { thickness: 1, color: BLACK },
                        RectangleFill { color: BLACK },
                        Erase,
                        Fill { color: BLACK },
                    ];

                    for tool in tools.iter() {
                        if ui.button(tool.to_string()).clicked() {
                            if let Some(saved_tool) = canvas.saved_settings().get(tool) {
                                *selected_state = saved_tool.clone();
                            } else {
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
                        SinglePixel { mut color } => {
                            ui.label("Color:");
                            let mut c = color.to_vec().to_array();
                            ui.color_edit_button_rgba_unmultiplied(&mut c);
                            color = Color::from_vec(Vec4::from_array(c));
                            *selected_state = SinglePixel { color };
                        }
                        RectangleFill { mut color } => {
                            let mut c = color.to_vec().to_array();
                            ui.label("Color:");
                            ui.color_edit_button_rgba_unmultiplied(&mut c);
                            color = Color::from_vec(Vec4::from_array(c));
                            *selected_state = RectangleFill { color };
                        }
                        RectangleBorder {
                            thickness,
                            mut color,
                        } => {
                            let mut c = color.to_vec().to_array();
                            ui.label("Color:");
                            ui.color_edit_button_rgba_unmultiplied(&mut c);
                            color = Color::from_vec(Vec4::from_array(c));
                            *selected_state = RectangleBorder { thickness, color };
                        }
                        Fill { mut color } => {
                            let mut c = color.to_vec().to_array();
                            ui.label("Color:");
                            ui.color_edit_button_rgba_unmultiplied(&mut c);
                            color = Color::from_vec(Vec4::from_array(c));
                            *selected_state = Fill { color };
                        }
                        _ => unimplemented!(),
                    }
                    canvas.saved_settings_mut().set(selected_state);
                });
            }

            egui_macroquad::egui::Window::new("Layers").show(egui_ctx, |ui| {
                if ui.button("Add new layer").clicked() {
                    let img_repr = layers.get_current_layer().unwrap().image_repr().clone();
                    layers.add_layer(Canvas::new_with_w_h(
                        img_repr.width() as u16,
                        img_repr.height() as u16,
                    ));
                    layers.set_current_layer_newest();
                }

                for (i, layer) in layers.get_all_layers().clone().iter().enumerate() {
                    let id = layer.get_id();
                    ui.horizontal(|ui| {
                        let mut cur =  layers.get_layer_mut(&id).unwrap().clone();
                        cur.image_repr_mut().invert_x();
                        cur.image_repr_mut().invert_y();
                        cur.draw_to_image();
                        
                        let loc = format!("res/snapshots/{}.png",id);
                        cur.image().export_png(loc.as_str());
                        let texture_color = load_image_from_path(&std::path::Path::new(&loc)).ok().unwrap();
                        let texture: &egui::TextureHandle =  &ui.ctx().load_texture(
                            loc.as_str(),
                            texture_color,
                            Default::default()
                        );
                        ui.image(texture,(50.0,50.0));
                        
                        ui.label(format!("Layer {}", i));
                        

                        let mut is_active = layers.is_active(&id);

                        if ui.checkbox(&mut is_active, "").clicked() {
                            if is_active {
                                layers.set_active(&id);
                            } else {
                                layers.unactive(&id);
                            }
                        }

                        if ui.small_button("x").clicked() {
                            layers.remove_layer(&id);
                        }
                    });
                }
            });
        });
    }
}


fn load_image_from_path(path: &std::path::Path) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}
