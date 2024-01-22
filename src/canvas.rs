use crate::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct CanvasSavedSettings {
    tool_settings: Vec<DrawState>
}


impl CanvasSavedSettings {
    fn new() -> Self {
        use DrawState::*;
        Self {
            tool_settings:vec![SinglePixel{color:BLACK},RectangleFill{color:BLACK},RectangleBorder{thickness:1,color:BLACK}] 
        }
    }


    pub fn get(&self,ds:&DrawState) -> Option<&DrawState> {
        for e in self.tool_settings.iter() {

            if *e == *ds {
                return Some(e);
            }

        }
        None
    }

    pub fn set(&mut self,ds:&DrawState) {
        for e in self.tool_settings.iter_mut() {
            if *e == *ds {
                *e = ds.clone();
            }

        }
    }

    pub fn add(&mut self,ds:DrawState) {
        self.tool_settings.push(ds);
    }
}

#[derive(Clone)]
pub struct Canvas {
    image:Image,
    image_repr:ImageRepr,
    image_repr_copy:ImageRepr,
    image_repr_copy_buffer:Option<ImageRepr>,
    texture:Texture2D,
    init_pos_draw:Option<Vec2>,
    saved_settings:CanvasSavedSettings,
    undo_redo_manager:UndoRedoManager<ImageRepr>,
    id:Uuid
}

impl Canvas {

    pub fn new_empty() -> Self {
        return Self::new_with_w_h(32,32);
    }

    pub fn empty_from(refcanvas:&Canvas) -> Self {
        let w = refcanvas.image.width() as u16;
        let h = refcanvas.image.height() as u16;
        return Self::new_with_w_h(w,h);
    }

    pub fn new_with_w_h(w:u16,h:u16) -> Self {
        let img = Image::gen_image_color(w,h,Color::from_rgba(0,0,0,0));
        let img_repr = ImageRepr::from_image(&img);
        Self {
            texture:Texture2D::from_image(&img),
            image_repr:img_repr.clone(),
            image_repr_copy:img_repr.clone(),
            image:img,
            image_repr_copy_buffer:None,
            init_pos_draw:None,
            saved_settings:CanvasSavedSettings::new(),
           undo_redo_manager:UndoRedoManager::new(),
            id:Uuid::new_v4()
        }
    } 

    pub fn new_with_image(img:Image) -> Self {
        let img_repr = ImageRepr::from_image(&img);
        return Self {
            texture:Texture2D::from_image(&img),
            image_repr:img_repr.clone(),
            image_repr_copy:img_repr.clone(),
            image:img,
            image_repr_copy_buffer:None,
            init_pos_draw:None,
            saved_settings:CanvasSavedSettings::new(),
           undo_redo_manager:UndoRedoManager::new(), 
            id:Uuid::new_v4()
        }
    }

    pub fn draw_canvas(&self) {
        self.image_repr.draw();
    }

    pub fn update_texture(&mut self) {
        self.texture.update(&self.image);
    }

    pub fn save_image(&mut self) {
        let img_repr_save = self.image_repr.clone();
        self.image_repr.invert_x();
        self.image_repr.invert_y();
        self.draw_to_image();
        self.image.export_png("output.png");
        self.image_repr = img_repr_save;
        println!("Finished Saving as output.png");
    }

    pub fn save_image_with_name(&mut self,name:&str) {
        let img_repr_save = self.image_repr.clone();
        self.image_repr.invert_x();
        self.image_repr.invert_y();
        self.draw_to_image();
        // let name = format!("{}",name);
        self.image.export_png(&name);
        self.image_repr = img_repr_save;
        println!("Finished Saving as {:?}",name);
    }

    pub fn draw_to_image_repr(&mut self,point:Vec2,state:&DrawState) {
        if point.x < -0.5 || point.x > 0.5 || point.y < -0.5 || point.y > 0.5 {
            return;
        } 

        use DrawState::*;

        let (mut px, mut py) = get_pixel_coords(point,(self.image.width() as u32,self.image.height()as u32));

        // Ensure px and py are within bounds
        if px >= self.image.width() as u32 {
            px = self.image.width() as u32 - 1;
        }
        if py >= self.image.height() as u32 {
            py = self.image.height() as u32 - 1;
        }
        let dimensions = (self.image.width() as u32,self.image.height() as u32);
        match state {
            Erase => {
                self.undo_redo_manager.push(self.image_repr.clone());
                self.image_repr.set_pixel(px,py,self.image_repr_copy.get_pixel(px,py));

            }
            SinglePixel {color}=> {
                self.undo_redo_manager.push(self.image_repr.clone());
                self.image_repr.set_pixel(px, py,*color);
            }

            RectangleBorder { thickness, color } => {
                if let Some(init_pos) = self.init_pos_draw {
                    if self.image_repr_copy_buffer.is_none() {
                        self.image_repr_copy_buffer = Some(self.image_repr.clone());
                        self.undo_redo_manager.push(self.image_repr.clone());
                    }
                    let mut buffer = self.image_repr_copy_buffer.clone().unwrap();

                    // get initial pixel positions
                    let pixels = rect_get_border_pixels(&init_pos, dimensions, px, py, &self.image);
                    for (x, y) in pixels.iter() {
                        buffer.set_pixel(*x, *y, *color);
                    }

                    self.image_repr = buffer;
                } else {
                    self.image_repr_copy_buffer = None;
                    self.init_pos_draw = None;
                    return;
                }
            }

            RectangleFill { color } => {
                if let Some(i) = &self.image_repr_copy_buffer {
                    self.undo_redo_manager.push(self.image_repr.clone());
                }
                if let Some(init_pos) = self.init_pos_draw {
                    if self.image_repr_copy_buffer.is_none() {
                        self.image_repr_copy_buffer = Some(self.image_repr.clone());
                    }
                    let mut buffer = self.image_repr_copy_buffer.clone().unwrap();

                    let pixels = rect_get_border_pixels(&init_pos, dimensions, px, py, &self.image);
                    for (x, y) in pixels.iter() {
                        buffer.set_pixel(*x, *y, *color);
                    }

                    // Filling part
                    let (mut ipx, mut ipy) = get_pixel_coords(init_pos, dimensions);
                    if ipx == px || ipy == py {
                        return;
                    }

                    if ipx >= self.image.width() as u32 {
                        ipx = self.image.width() as u32 - 1;
                    }
                    if ipy >= self.image.height() as u32 {
                        ipy = self.image.height() as u32 - 1;
                    }

                    for x in ipx.min(px)..ipx.max(px) {
                        for y in ipy.min(py)..ipy.max(py) {
                            buffer.set_pixel(x, y, *color);
                        }
                    }

                    self.image_repr = buffer;
                } else {
                    self.image_repr_copy_buffer = None;
                    self.init_pos_draw = None;
                    return;
                }
            }


            Fill {color} => {
                if let Some((_rect,wanted_color)) = self.image_repr().get_pixel_data(px,py) {
                    self.undo_redo_manager.push(self.image_repr.clone());
                    let mut neighbours:Vec<usize> = Vec::new();
                    if let Some(pixel_index) = self.image_repr().get_pixel_index(px,py) {
                        fill_get_all_pixels(pixel_index,self.image_repr(),&mut neighbours,wanted_color);
                        for i in neighbours.iter() {
                            self.image_repr.data[*i].1 = *color
                        }
                    }

                }

            }
            _ => unimplemented!(),
        }


    }
    
    pub fn draw_to_image(&mut self) {
        let new_img = self.image_repr.to_image();
        self.image = new_img;
    }

    pub fn finish_drawing_current(&mut self, state:&DrawState) {
        use DrawState::*;
        match state {
            RectangleBorder{..} | RectangleFill{..} => {
                self.image_repr_copy_buffer = None;
                self.init_pos_draw = None;
                return;
            }
            _ => return
        }
    }


    pub fn init_pos_draw(&self) -> Option<Vec2> {
        self.init_pos_draw
    }

    pub fn set_init_pos_draw(&mut self, n:Option<Vec2>) {
        self.init_pos_draw =n;
    }

    pub fn image_repr(&self) -> &ImageRepr {
        &self.image_repr
    }

    pub fn image_repr_mut(&mut self) -> &mut ImageRepr {
        &mut self.image_repr
    }


    pub fn image(&self) -> &Image {
        &self.image
    }

    pub fn saved_settings(&self) -> &CanvasSavedSettings {
        &self.saved_settings
    }

    pub fn saved_settings_mut(&mut self) -> &mut CanvasSavedSettings {
        &mut self.saved_settings
    }

    pub fn undo(&mut self) {
        if let Some(r) = self.undo_redo_manager.undo() {
            self.image_repr = r.clone();
        }
        
    }

    pub fn redo(&mut self) {
         if let Some(r) = self.undo_redo_manager.redo() {
            self.image_repr = r.clone();
        } 
    }

    pub fn get_id(&self) -> &Uuid {
        &self.id
    }

}


// UNREALIABLE: this sometimes gives more than the allowed height is the coords are more to the
// invalid pixel
// Will make a better one sometime in the future

pub fn get_pixel_coords(coords:Vec2,dimensions:(u32,u32)) -> (u32,u32) {
    let (width, height) = dimensions;
    let x_pixel = ((coords.x + 0.5) * width as f32) as u32;
    let y_pixel = ((coords.y + 0.5) * height as f32) as u32;

    return (x_pixel,y_pixel);
}


// Representation of image pixels in rect form.
#[derive(Clone,PartialEq)]
pub struct ImageRepr {
    data:Vec<(Rect,Color)>,
    height:u32,
    width:u32
}

impl ImageRepr {
    pub fn from_image(image:&Image) -> Self {
        let mut data =Vec::new();
        let (width, height) = (image.width(),image.height());
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
        for y in ys.iter() {
            for x in xs.iter().rev() {
                let (mut px,mut py) = get_pixel_coords(vec2(*x,*y),(width as u32,height as u32));
                if px >= image.width() as u32{
                    px = image.width() as u32- 1;
                }
                if py >= image.height() as u32{
                    py = image.height() as u32 - 1;
                }

                //data.push((Rect::new(*x,*y,step_x,step_y),RED));
                data.push((Rect::new(*x,*y,step_x,step_y),image.get_pixel(px,py)));

            }
        } 

        data.reverse();
        return Self {
            data,
            height:image.height() as u32,
            width:image.width() as u32
        }
    }


    pub fn to_image(&self) -> Image {
        let mut image = Image::gen_image_color(self.width() as u16,self.height() as u16,Color::from_rgba(0,0,0,0));
        for i in 0..self.data.len() {
            
            let x = i as u32%self.width() as u32;
            let y = i as u32/ self.width() as u32;
            let c = self.get_pixel(x,y);

            image.set_pixel(x,y,c);
        } 
        return image;
    }

    pub fn draw(&self) {
        for pixel in self.data.iter() {
            let r = pixel.0;
            draw_rectangle(r.x,r.y,r.w,r.h,pixel.1);
        }
    }


    pub fn draw_to_image(&self, image: &mut Image) {
        let mut repr = self.clone(); 
        repr.invert_x();
        repr.invert_y();

        for i in 0..repr.data().len() {
            let x = i as u32 % repr.width() as u32;
            let y = i as u32 / repr.width() as u32;
            
              let c = repr.get_pixel(x, y);
            if c.a == 0.0 {
                continue;
            }
            image.set_pixel(x, y, c);
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        let i = ((self.width()-1)-y) * self.width() + x;
        self.data[i as usize].1 = color;
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        let i = y * self.width() + x;
        self.data[i as usize].1
    }

    
    pub fn get_pixel_data(&self,x:u32,y:u32) -> Option<(Rect,Color)> {
        if x >= self.width() || y >= self.height() {
            return None
        }
       let i = (self.width()-1-y) * self.width() + x;
        
        self.data.get(i as usize).copied()
    }

    pub fn get_pixel_rect(&self,x:u32,y:u32) -> Option<Rect> {
        if let Some(d) = self.get_pixel_data(x,y) {
            return Some(d.0.clone())
        }
        None    
    }

    pub fn get_pixel_index(&self,x:u32,y:u32) -> Option<usize> {
        let i = ((self.width()-1)-y)*self.width()+x;
        Some(i as usize)
    }



    pub fn get_neighbours(&self,i:usize) -> Vec<usize> {
        if i >= self.data.len() {
            return Vec::new();
        }
        let mut v = Vec::new();

        // top row

        if (i as u32) < self.width() {
            v.push(i+self.width() as usize);
        }
        // bottom row
        else if (i as u32) > (self.width()*self.height()-2)-self.width() {
            v.push(i-self.width() as usize);
        }else {

            v.push(i+self.width() as usize);
            v.push(i-self.width() as usize);
        }

        // left and right
        if (i as u32)%self.width() == 0 {
            v.push(i+1);
        }else if (i as u32+1)%self.width() == 0 {
            v.push(i-1);
        }else {
            v.push(i+1);
            v.push(i-1);
        }

        v
    }

    pub fn invert_y(&mut self) {
        self.data.reverse();
    }


  
pub fn invert_x(&mut self) {
    for y in 0..self.height() {
        for x in 0..self.width() / 2 {
            let cur_index = (y * self.width() + x) as usize;
            let mirror_index = (y * self.width() + self.width() - x - 1) as usize;

            self.data.swap(cur_index, mirror_index);
        }
    }
}

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn data(&self) -> &Vec<(Rect,Color)> {
        &self.data
    }

}
