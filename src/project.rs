use crate::*;

pub struct Project {
    name:Option<String>,
    dimensions:(u8,u8),
    zoom:Vec2,
    init_pos:Option<(f32,f32)>,
    state:DrawState,
    camera:Camera2D,
    ui:Ui,
    layers:Layers,
}

impl Project {

    pub fn new_untitled(dimensions:(u8,u8)) -> Self {
        let camera = Camera2D{
            ..Default::default()
        };
        set_camera(&camera);
        Self {
            name:None,
            dimensions,
            zoom:Vec2::new(0.,0.),
            init_pos:None,
            state:DrawState::default(),
            camera,
            ui:Ui::new(),
            layers:Layers::new(Canvas::new_with_w_h(dimensions.0 as u16, dimensions.1 as u16)), 
        }

    }

    pub fn new_titled(name:&String,dimensions:(u8,u8)) -> Self {
        let mut p = Project::new_untitled(dimensions);
        p.name = Some(name.to_owned());
        return p;
    }
    
    pub fn handle_zoom(&mut self) {
         let (_,new_mouse_wheel_y) = mouse_wheel();
            if new_mouse_wheel_y != 0.0 {

                if new_mouse_wheel_y > 0.0 {
                    if self.zoom.y < MAX_ZOOM_POSITIVE && self.zoom.x < MAX_ZOOM_POSITIVE{
                        self.zoom += ZOOM_ADD;
                        self.camera.zoom = self.zoom;
                    }
                }else {

                    if self.zoom.y > MAX_ZOOM_NEGATIVE && self.zoom.x > MAX_ZOOM_NEGATIVE{
                        self.zoom -= ZOOM_ADD;
                        self.camera.zoom = self.zoom;
                    }
                }
            set_camera(&self.camera);
        }

    }

    pub fn move_camera(&mut self) {
        if is_key_down(KeyCode::LeftControl) {
            if is_mouse_button_down(MouseButton::Left) {
                if let Some(init_pos) = self.init_pos {
                    let cur_pos = mouse_position();
                    let dist_x = cur_pos.0 - init_pos.0;
                    let dist_y = cur_pos.1 - init_pos.1;
                    self.camera.offset=vec2(-dist_x/1000.*self.camera.zoom.x+init_pos.0,dist_y/1000.*self.camera.zoom.y*-1.0);
                    set_camera(&self.camera);
                }else {
                    self.init_pos = Some(mouse_position())
                }
            }else {

                self.init_pos = None;
            }
        }else {
            self.init_pos = None;
        }

    }
    

    pub fn handle_drawing(&mut self) {
        if !self.ui.is_clicked() {

            let (mx,my) = mouse_position();
            let point = self.camera.screen_to_world(vec2(mx,my));
            let state = self.state;

                let i= self.layers.get_current_layer_index().clone();
            if let Some(canvas) = self.layers.get_current_layer_mut() {
                if is_mouse_button_pressed(MouseButton::Left) || is_mouse_button_down(MouseButton::Left){
                    if canvas.init_pos_draw().is_none() {
                        canvas.set_init_pos_draw(Some(point));
                    }else {
                        canvas.draw_to_image_repr(point,&state);
                    }
                }else {
                    canvas.finish_drawing_current(&state); 
                    canvas.set_init_pos_draw(None)
                }   
            }

        }
    }

    pub fn backend(&mut self) {
        self.handle_zoom();
        
        self.move_camera(); 
              self.handle_drawing();

        self.handle_keyboard_shortcuts();
    }



    pub fn frontend(&mut self) {
        if self.get_selected_canvas_mut().is_none() {
            return;
        }

        let (mx, my) = mouse_position();
        let mcoords = self.camera.screen_to_world(vec2(mx, my));
        let dimensions = (self.dimensions.0 as u32, self.dimensions.1 as u32);
        
        draw_pixel_bg(dimensions);
        let active_layers = self.layers.get_all_active().clone();
        for id in active_layers.iter() {
            if let Some(e) = self.layers.get_active_layer(id) {
                e.draw_canvas();
            }
        }

        {let canvas = self.get_selected_canvas_mut().unwrap();
            

            // draw a gray square in the camera position to the nearest pixel
            // This gives a nice effect
            if !canvas.init_pos_draw().is_some() {
                if !(mcoords.x >= 0.5 || mcoords.x <= -0.5 || mcoords.y >= 0.5 || mcoords.y <= -0.5) {
                    let (px, py) = get_pixel_coords(mcoords, dimensions);
                    if let Some(r) = canvas.image_repr().get_pixel_rect(px,py) {
                        let color = Color::new(0.51, 0.51, 0.51, 0.5); // translucent gray-ish
                        draw_rectangle(r.x, r.y, r.w, r.h, color);

                    }
                }
            }
        }
        {


            let mut lc = self.layers.clone();
            let mut ui_name = self.name.clone();
            let mut ui_state = self.state.clone();
            let mut ui = self.ui_mut();
            ui.draw(&mut ui_name,&mut ui_state,&mut lc);
            self.name = ui_name;
            self.state = ui_state;
            self.layers = lc;
           // *self.layers.get_current_layer_mut().unwrap() = canvas;
        }


    }




    pub fn handle_keyboard_shortcuts(&mut self) {
        if let Some(mut canvas) = self.get_selected_canvas_mut() {
        if !(is_key_down(KeyCode::RightControl) || is_key_down(KeyCode::LeftControl)) {
            return;
        }

        if is_key_pressed(KeyCode::S) {
            self.handle_saving();
        }else if is_key_pressed(KeyCode::Z) {
            canvas.undo();
        }else if is_key_pressed(KeyCode::Y) {
            canvas.redo();
        }else if is_key_down(KeyCode::Z) {
            canvas.undo();
        } else if is_key_down(KeyCode::Y) {
            canvas.redo();
        }
        }


    }

    // Does all the saving stuff
    // checks if there is already a name, asks user a name etc.
    pub fn handle_saving(&mut self) {
        if self.get_selected_canvas_mut().is_none() {
            return;
        }
            if let Some(name)  = self.name.clone() {
               self.get_selected_canvas_mut().unwrap().save_image_with_name(name.as_str()); 
            }else  {
                self.save_as();
            }

    }

    pub fn save_as(&mut self) {
        if self.get_selected_canvas_mut().is_none() {
            return;
        }
            use native_dialog::*;
            let path = FileDialog::new()
                .set_location("~/Desktop")
                .add_filter("PNG Image", &["png"])
                .add_filter("JPEG Image", &["jpg", "jpeg"])
                .show_save_single_file()
                .unwrap();

            if let Some(n) = path {
                let n = n.as_os_str().to_str().unwrap();
                self.set_name(n.to_string());
                self.get_selected_canvas_mut().unwrap().save_image_with_name(n);
            }

    }

   
    pub fn set_draw_state(&mut self,new_state:DrawState) {
        self.state = new_state
    }

    pub fn draw_state(&self) -> &DrawState {
        &self.state
    }

    pub fn set_name(&mut self,name:String) {
        self.name = Some(name);
    } 

    pub fn ui_mut(&mut self) -> &mut Ui {
        &mut self.ui
    }

    pub fn get_selected_canvas(&mut self) -> Option<&Canvas> {
        self.layers.get_current_layer() 

    }

    pub fn get_selected_canvas_mut(&mut self) -> Option<&mut Canvas> {
        self.layers.get_current_layer_mut()
    }

    pub fn ui(&self) -> &Ui {
        &self.ui
    }
    
    pub fn state_mut(&mut self) -> &mut DrawState {
        &mut self.state
    }

    pub fn name_mut(&mut self) -> &mut Option<String> {
        &mut self.name
    }

}


fn draw_pixel_bg(dimensions:(u32,u32)) {
    let (width, height) = dimensions;
    let step_x = 1.0 / width as f32;
    let step_y = 1.0 / height as f32;

    let mut x = -0.5;
    let mut y = -0.5;
    let mut xs:Vec<f32> = Vec::new();
    let mut ys:Vec<f32> = Vec::new();
    for _ in 0..height {
        ys.push(y);
        y += step_y;
    }

    for _ in 0..width {
        xs.push(x);
        x += step_x;
    }

    for (ix,x) in xs.iter().enumerate() {
        for (iy,y) in  ys.iter().enumerate() {
            let color = if (ix+iy) %2==0 {
                DARKGRAY
            }else {
                WHITE
            };
            draw_rectangle(*x,*y,step_x,step_y,color);
        }
    }

    
}



