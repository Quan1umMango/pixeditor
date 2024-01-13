use crate::*;



pub struct Project {
    name:Option<String>,
    canvas:Canvas,
    dimensions:(u8,u8),
    zoom:Vec2,
    init_pos:Option<(f32,f32)>,
    state:DrawState,
    camera:Camera2D,
    ui:Ui,
}

impl Project {

    pub fn new_untitled(dimensions:(u8,u8)) -> Self {
        let camera = Camera2D{
            ..Default::default()
        };
        set_camera(&camera);
        Self {
            name:None,
            canvas:Canvas::new_with_w_h(dimensions.0 as u16,dimensions.1 as u16),
            dimensions,
            zoom:Vec2::new(0.,0.),
            init_pos:None,
            state:DrawState::default(),
            camera,
            ui:Ui::new(),
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

            if is_mouse_button_pressed(MouseButton::Left) || is_mouse_button_down(MouseButton::Left){
                if self.canvas.init_pos_draw().is_none() {
                    self.canvas.set_init_pos_draw(Some(point));
                }
                self.canvas.draw_to_image_repr(point,&self.state);
            }else {
                self.canvas.finish_drawing_current(&self.state); 
                self.canvas.set_init_pos_draw(None)
            }   
        }
    }

    pub fn backend(&mut self) {
        self.handle_zoom();

        self.move_camera();       
        self.handle_drawing();

        // Saving images
        if is_key_pressed(KeyCode::Space) {
            self.canvas.save_image();
        }
    }



    pub fn frontend(&mut self) {
        self.canvas.draw_canvas();

        // draw a gray square in the camera position to the nearest pixel
        // This gives a nice effect 
        if !self.canvas.init_pos_draw().is_some()  {

            let (mx,my) = mouse_position();
            let mcoords  = self.camera.screen_to_world(vec2(mx,my));
            let dimensions = (self.dimensions.0 as u32, self.dimensions.1 as u32);
            if !(mcoords.x >= 0.5 || mcoords.x <= -0.5 || mcoords.y >= 0.5 || mcoords.y <= -0.5){
                let (px,py) = get_pixel_coords(mcoords,dimensions);
                let r = self.canvas.image_repr().get_pixel_rect(px,py);
                let color = Color::new(0.51, 0.51, 0.51, 0.5); //translucent ray-ish 
                draw_rectangle(r.x ,r.y,r.w,r.h,color);           
            }

        }

        self.ui.draw(&mut self.name,&mut self.state,&mut self.canvas);

    }
}


